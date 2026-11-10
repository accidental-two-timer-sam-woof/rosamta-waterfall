//! P6: the platform operations 05 §6 added on top of P1–P3 (ASM-16–ASM-21).
//!
//! - Creating and renaming without replacing what is already there (§6.3
//!   "建立", initialization's directory rename; ASM-16).
//! - Unix: replacing by exchanging the temporary file with the target, and
//!   deleting by moving the target aside first, so a file swapped in after
//!   the last check is found and put back (§6.3; ASM-17).
//! - Windows: holding the target with `FILE_READ_DATA` and only
//!   `FILE_SHARE_READ` while it is checked, so no one can rename, delete or
//!   write it (§6.3; ASM-18).
//! - Windows: creating a directory with its own DACL, and copying a file's
//!   DACL to the temporary file that replaces it (§6.4; ASM-19).
//! - The lock that keeps writing commands apart (§6.7; ASM-20).
//! - Opening without blocking on a FIFO (§6.3 "版本"; ASM-21).

use std::ffi::{OsStr, OsString};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::bound::Identity;

pub use imp::*;

/// What a check expects to find at the target: the observation of 05 §6.3.
pub struct Observed {
    pub identity: Identity,
    pub content: Vec<u8>,
}

/// How a bound replace or delete ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Done {
    /// The operation went through on the observed file.
    Applied,
    /// The file moved out of the way was not the observed one, so it was
    /// put back under the target name and nothing was changed (Unix).
    PutBack,
}

/// A temporary name next to `name`, as in 05 §5.1.
pub fn temp_name(name: &OsStr) -> OsString {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    let mut temp = OsString::from(".");
    temp.push(name);
    temp.push(format!(
        "{}{}.{seq}",
        crate::atomic::TEMP_MARKER,
        std::process::id()
    ));
    temp
}

/// The error another holder of the lock causes: `EWOULDBLOCK` on Unix,
/// `ERROR_SHARING_VIOLATION` on Windows.
pub fn is_busy(e: &io::Error) -> bool {
    #[cfg(unix)]
    {
        e.kind() == io::ErrorKind::WouldBlock
    }
    #[cfg(windows)]
    {
        e.raw_os_error() == Some(32)
    }
}

/// Short name for an OS error, for the result lines.
pub fn error_name(e: &io::Error) -> String {
    match e.raw_os_error() {
        #[cfg(unix)]
        Some(code) => format!("errno:{code}({:?})", e.kind()),
        #[cfg(windows)]
        Some(code) => format!("win32:{code}({:?})", e.kind()),
        None => format!("{:?}", e.kind()),
    }
}

#[cfg(unix)]
mod imp {
    use super::{Done, Observed, temp_name};
    use crate::bound::unix::entry_of;
    use crate::bound::{Dir, Entry, Kind};
    use rustix::fs::{AtFlags, FlockOperation, Mode, OFlags, RenameFlags};
    use std::ffi::OsStr;
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::os::fd::OwnedFd;

    /// Writes a complete `0600` file under a new name (05 §6.4).
    pub fn write_temp(dir: &Dir, name: &OsStr, content: &[u8]) -> io::Result<Entry> {
        let fd = rustix::fs::openat(
            dir.fd(),
            name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )?;
        let entry = entry_of(&rustix::fs::fstat(&fd)?);
        let mut file = File::from(fd);
        file.write_all(content)?;
        file.sync_all()?;
        Ok(entry)
    }

    /// 05 §6.3 "建立": gives the temporary file the target name with
    /// `linkat`, which fails with `EEXIST` if the name exists in any form,
    /// then drops the temporary name (ASM-16).
    pub fn create_no_replace(dir: &Dir, temp: &OsStr, name: &OsStr) -> io::Result<()> {
        rustix::fs::linkat(dir.fd(), temp, dir.fd(), name, AtFlags::empty())?;
        rustix::fs::unlinkat(dir.fd(), temp, AtFlags::empty())?;
        Ok(())
    }

    /// `RENAME_NOREPLACE` on Linux, `RENAME_EXCL` on macOS (ASM-16).
    pub fn rename_no_replace(dir: &Dir, from: &OsStr, to: &OsStr) -> io::Result<()> {
        rustix::fs::renameat_with(dir.fd(), from, dir.fd(), to, RenameFlags::NOREPLACE)?;
        Ok(())
    }

    /// `RENAME_EXCHANGE` on Linux, `RENAME_SWAP` on macOS (ASM-17).
    pub fn exchange(dir: &Dir, a: &OsStr, b: &OsStr) -> io::Result<()> {
        rustix::fs::renameat_with(dir.fd(), a, dir.fd(), b, RenameFlags::EXCHANGE)?;
        Ok(())
    }

    pub fn rename(dir: &Dir, from: &OsStr, to: &OsStr) -> io::Result<()> {
        rustix::fs::renameat(dir.fd(), from, dir.fd(), to)?;
        Ok(())
    }

    pub fn unlink(dir: &Dir, name: &OsStr) -> io::Result<()> {
        rustix::fs::unlinkat(dir.fd(), name, AtFlags::empty())?;
        Ok(())
    }

    /// Opens `name` with `O_NOFOLLOW | O_NONBLOCK` and reports what was
    /// actually opened (ASM-21). The caller decides from the kind whether
    /// to read it.
    pub fn open_nonblocking(dir: &Dir, name: &OsStr) -> io::Result<(Entry, File)> {
        let fd = rustix::fs::openat(
            dir.fd(),
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )?;
        let entry = entry_of(&rustix::fs::fstat(&fd)?);
        Ok((entry, File::from(fd)))
    }

    /// Whether `name` is still the observed regular file with the observed
    /// content. A link, a FIFO or anything else is simply "not the same".
    pub fn matches(dir: &Dir, name: &OsStr, observed: &Observed) -> io::Result<bool> {
        match dir.entry(name)? {
            Some(e) if e.kind == Kind::File && e.identity == observed.identity => {}
            _ => return Ok(false),
        }
        let (entry, mut file) = match open_nonblocking(dir, name) {
            Ok(opened) => opened,
            Err(e) if e.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()) => {
                return Ok(false);
            }
            Err(e) => return Err(e),
        };
        if entry.kind != Kind::File || entry.identity != observed.identity {
            return Ok(false);
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(bytes == observed.content)
    }

    /// 05 §6.3 "取代": the last check, then `hook` (the `BeforeOperation`
    /// sync point), then one exchange. The old name now holds the file that
    /// was replaced; if it is not the observed one, exchange back.
    pub fn replace(
        dir: &Dir,
        name: &OsStr,
        content: &[u8],
        observed: &Observed,
        hook: &mut dyn FnMut(),
    ) -> io::Result<Done> {
        let temp = temp_name(name);
        let written = write_temp(dir, &temp, content)?;
        let result = (|| {
            if !matches(dir, name, observed)? {
                return Err(io::Error::other("target differs at the last check"));
            }
            hook();
            exchange(dir, &temp, name)?;
            if matches(dir, &temp, observed)? {
                unlink(dir, &temp)?;
                return Ok(Done::Applied);
            }
            exchange(dir, &temp, name)?;
            Ok(Done::PutBack)
        })();
        // Whatever happened, the temporary name ends up holding our file
        // (or nothing, after a successful replace); remove only that.
        if let Ok(Some(e)) = dir.entry(&temp)
            && e.identity == written.identity
        {
            unlink(dir, &temp)?;
        }
        result
    }

    /// 05 §6.3 "刪除": the last check, then `hook`, then move the target to
    /// a temporary name and check it there. Delete it only if it is the
    /// observed file; otherwise move it back without replacing anything.
    pub fn remove(
        dir: &Dir,
        name: &OsStr,
        observed: &Observed,
        hook: &mut dyn FnMut(),
    ) -> io::Result<Done> {
        if !matches(dir, name, observed)? {
            return Err(io::Error::other("target differs at the last check"));
        }
        hook();
        let aside = temp_name(name);
        rename(dir, name, &aside)?;
        if matches(dir, &aside, observed)? {
            unlink(dir, &aside)?;
            return Ok(Done::Applied);
        }
        rename_no_replace(dir, &aside, name)?;
        Ok(Done::PutBack)
    }

    /// The `lock` file of 05 §6.7: `flock(LOCK_EX | LOCK_NB)` on a file
    /// opened without following links (ASM-20).
    pub struct Lock(#[allow(dead_code)] OwnedFd);

    pub fn lock(dir: &Dir, name: &OsStr) -> io::Result<Lock> {
        let fd = rustix::fs::openat(
            dir.fd(),
            name,
            OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )?;
        rustix::fs::flock(&fd, FlockOperation::NonBlockingLockExclusive)?;
        Ok(Lock(fd))
    }

    /// What another program does to a file, for the child process.
    pub fn external(op: &str, path: &std::path::Path, other: &std::path::Path) -> io::Result<()> {
        match op {
            "rename" => std::fs::rename(path, other),
            "delete" => std::fs::remove_file(path),
            "write" => std::fs::OpenOptions::new()
                .write(true)
                .open(path)?
                .write_all(b"external\n"),
            "replace" => std::fs::rename(other, path),
            "read" => std::fs::read(path).map(drop),
            _ => Err(io::Error::other(format!("unknown op {op}"))),
        }
    }
}

#[cfg(windows)]
mod imp {
    use super::{Done, Observed, temp_name};
    use crate::bound::windows::{NO_FOLLOW, delete_on_close, entry_of, open};
    use crate::bound::{Dir, Entry, Kind};
    use std::ffi::OsStr;
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{AsRawHandle, OwnedHandle};
    use std::path::Path;
    use windows_sys::Win32::Foundation::{
        CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE, LocalFree,
    };
    use windows_sys::Win32::Security::Authorization::{
        ConvertSecurityDescriptorToStringSecurityDescriptorW, ConvertSidToStringSidW,
        ConvertStringSecurityDescriptorToSecurityDescriptorW, GetSecurityInfo, SDDL_REVISION_1,
        SE_FILE_OBJECT, SetSecurityInfo,
    };
    use windows_sys::Win32::Security::{
        ACL, DACL_SECURITY_INFORMATION, GetSecurityDescriptorControl, GetSecurityDescriptorDacl,
        GetTokenInformation, PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR,
        SE_DACL_PROTECTED, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER, TokenUser,
        UNPROTECTED_DACL_SECURITY_INFORMATION,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CREATE_NEW, CreateDirectoryW, DELETE, DeleteFileW, FILE_ATTRIBUTE_NORMAL,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        MoveFileExW, OPEN_ALWAYS, OPEN_EXISTING, READ_CONTROL, SYNCHRONIZE, WRITE_DAC,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows_sys::core::PWSTR;

    fn wide(s: &OsStr) -> Vec<u16> {
        s.encode_wide().chain(std::iter::once(0)).collect()
    }

    fn move_file(from: &Path, to: &Path, flags: u32) -> io::Result<()> {
        let (from, to) = (wide(from.as_os_str()), wide(to.as_os_str()));
        // SAFETY: both are NUL-terminated UTF-16 buffers that outlive the call.
        if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), flags) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Writes a complete file under a new name (05 §6.4).
    pub fn write_temp(dir: &Dir, name: &OsStr, content: &[u8]) -> io::Result<Entry> {
        let handle = open(
            &dir.path().join(name),
            GENERIC_WRITE | GENERIC_READ,
            0,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
        )?;
        let mut file = File::from(handle);
        file.write_all(content)?;
        file.sync_all()?;
        entry_of(&file)
    }

    /// 05 §6.3 "建立": `MoveFileExW` without `MOVEFILE_REPLACE_EXISTING`
    /// (ASM-16).
    pub fn create_no_replace(dir: &Dir, temp: &OsStr, name: &OsStr) -> io::Result<()> {
        move_file(
            &dir.path().join(temp),
            &dir.path().join(name),
            MOVEFILE_WRITE_THROUGH,
        )
    }

    /// The same call renames a directory without replacing (ASM-16).
    pub fn rename_no_replace(dir: &Dir, from: &OsStr, to: &OsStr) -> io::Result<()> {
        create_no_replace(dir, from, to)
    }

    /// Replaces `to` the way P2 settled on (ASM-02).
    pub fn move_replace(dir: &Dir, from: &OsStr, to: &OsStr) -> io::Result<()> {
        move_file(
            &dir.path().join(from),
            &dir.path().join(to),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    }

    /// The target held for its last check: data access, so that the share
    /// mode counts (ASM-08), and only `FILE_SHARE_READ`, so that no one can
    /// rename, delete or write it (ASM-18).
    pub struct Held(File);

    pub fn hold(dir: &Dir, name: &OsStr, delete: bool) -> io::Result<(Held, Entry, Vec<u8>)> {
        let access =
            FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE | if delete { DELETE } else { 0 };
        let handle = open(
            &dir.path().join(name),
            access,
            FILE_SHARE_READ,
            OPEN_EXISTING,
            NO_FOLLOW,
        )?;
        let file = File::from(handle);
        let entry = entry_of(&file)?;
        let mut bytes = Vec::new();
        if entry.kind == Kind::File {
            (&file).read_to_end(&mut bytes)?;
        }
        Ok((Held(file), entry, bytes))
    }

    /// Negative control for ASM-18: a handle with attribute access only,
    /// the kind ASM-08 found takes no part in sharing checks.
    pub fn hold_attributes_only(dir: &Dir, name: &OsStr) -> io::Result<OwnedHandle> {
        open(
            &dir.path().join(name),
            FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            FILE_SHARE_READ,
            OPEN_EXISTING,
            NO_FOLLOW,
        )
    }

    fn check(entry: &Entry, bytes: &[u8], observed: &Observed) -> io::Result<()> {
        if entry.kind != Kind::File
            || entry.identity != observed.identity
            || bytes != observed.content
        {
            return Err(io::Error::other("target differs at the last check"));
        }
        Ok(())
    }

    /// 05 §6.3 "取代": hold and check the target, run `hook` while it is
    /// held, close it and replace it with `MoveFileExW`.
    pub fn replace(
        dir: &Dir,
        name: &OsStr,
        content: &[u8],
        observed: &Observed,
        hook: &mut dyn FnMut(),
    ) -> io::Result<Done> {
        let temp = temp_name(name);
        write_temp(dir, &temp, content)?;
        let result = (|| {
            let (held, entry, bytes) = hold(dir, name, false)?;
            check(&entry, &bytes, observed)?;
            hook();
            drop(held);
            move_replace(dir, &temp, name)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(dir.path().join(&temp));
        }
        result.map(|()| Done::Applied)
    }

    /// 05 §6.3 "刪除": hold the target with `DELETE`, check it, run `hook`
    /// while it is held, and delete that handle's file (ASM-10).
    pub fn remove(
        dir: &Dir,
        name: &OsStr,
        observed: &Observed,
        hook: &mut dyn FnMut(),
    ) -> io::Result<Done> {
        let (held, entry, bytes) = hold(dir, name, true)?;
        check(&entry, &bytes, observed)?;
        hook();
        delete_on_close(&held.0)?;
        drop(held);
        Ok(Done::Applied)
    }

    /// The `lock` file of 05 §6.7: opened with share mode `0` (ASM-20).
    pub struct Lock(#[allow(dead_code)] OwnedHandle);

    pub fn lock(dir: &Dir, name: &OsStr) -> io::Result<Lock> {
        let handle = open(
            &dir.path().join(name),
            GENERIC_READ | GENERIC_WRITE,
            0,
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
        )?;
        Ok(Lock(handle))
    }

    /// What another program does to a file, for the child process.
    pub fn external(op: &str, path: &Path, other: &Path) -> io::Result<()> {
        match op {
            "rename" => move_file(path, other, 0),
            "delete" => {
                let p = wide(path.as_os_str());
                // SAFETY: p is a NUL-terminated UTF-16 buffer.
                if unsafe { DeleteFileW(p.as_ptr()) } == 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            }
            "write" => std::fs::OpenOptions::new()
                .write(true)
                .open(path)?
                .write_all(b"external\n"),
            "replace" => move_file(other, path, MOVEFILE_REPLACE_EXISTING),
            "read" => std::fs::read(path).map(drop),
            _ => Err(io::Error::other(format!("unknown op {op}"))),
        }
    }

    /// Memory the security functions allocate with `LocalAlloc`.
    struct Local(*mut core::ffi::c_void);

    impl Drop for Local {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: the pointer came from a function documented to
                // allocate it with LocalAlloc.
                unsafe { LocalFree(self.0) };
            }
        }
    }

    fn take_wide(p: PWSTR) -> String {
        let _free = Local(p.cast());
        let mut len = 0;
        // SAFETY: p is a NUL-terminated UTF-16 string from the system.
        while unsafe { *p.add(len) } != 0 {
            len += 1;
        }
        // SAFETY: the first len units were just read.
        String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(p, len) })
    }

    fn check_win32(code: u32) -> io::Result<()> {
        if code != 0 {
            return Err(io::Error::from_raw_os_error(code as i32));
        }
        Ok(())
    }

    /// The SID of the user the process runs as, in SDDL form.
    pub fn user_sid() -> io::Result<String> {
        let mut token: HANDLE = std::ptr::null_mut();
        // SAFETY: the token handle is closed below.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut buf = vec![0u64; 64];
        let mut len = 0u32;
        // SAFETY: buf is writable for the size passed in and aligned for
        // TOKEN_USER.
        let ok = unsafe {
            GetTokenInformation(
                token,
                TokenUser,
                buf.as_mut_ptr().cast(),
                (buf.len() * 8) as u32,
                &mut len,
            )
        };
        let err = io::Error::last_os_error();
        // SAFETY: token came from OpenProcessToken.
        unsafe { CloseHandle(token) };
        if ok == 0 {
            return Err(err);
        }
        // SAFETY: the call filled buf with a TOKEN_USER.
        let user = unsafe { &*(buf.as_ptr() as *const TOKEN_USER) };
        let mut s: PWSTR = std::ptr::null_mut();
        // SAFETY: the SID points into buf, which is alive.
        if unsafe { ConvertSidToStringSidW(user.User.Sid, &mut s) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(take_wide(s))
    }

    fn parse_sddl(sddl: &str) -> io::Result<Local> {
        let w: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
        let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        // SAFETY: w is NUL-terminated; sd receives a LocalAlloc'd descriptor.
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                w.as_ptr(),
                SDDL_REVISION_1,
                &mut sd,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Local(sd))
    }

    /// The DACL and whether it is protected from inheritance.
    fn dacl_of(sd: PSECURITY_DESCRIPTOR) -> io::Result<(*mut ACL, bool)> {
        let (mut present, mut defaulted) = (0, 0);
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let (mut control, mut revision) = (0u16, 0u32);
        // SAFETY: sd is a valid security descriptor; the out pointers are
        // valid for writes.
        unsafe {
            if GetSecurityDescriptorDacl(sd, &mut present, &mut dacl, &mut defaulted) == 0
                || GetSecurityDescriptorControl(sd, &mut control, &mut revision) == 0
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok((dacl, control & SE_DACL_PROTECTED != 0))
    }

    fn set_dacl_on(handle: &impl AsRawHandle, dacl: *const ACL, protected: bool) -> io::Result<()> {
        let flags = DACL_SECURITY_INFORMATION
            | if protected {
                PROTECTED_DACL_SECURITY_INFORMATION
            } else {
                UNPROTECTED_DACL_SECURITY_INFORMATION
            };
        // SAFETY: the handle has WRITE_DAC; dacl is valid or null.
        check_win32(unsafe {
            SetSecurityInfo(
                handle.as_raw_handle(),
                SE_FILE_OBJECT,
                flags,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                dacl,
                std::ptr::null(),
            )
        })
    }

    fn open_security(path: &Path, access: u32) -> io::Result<OwnedHandle> {
        open(
            path,
            access,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            OPEN_EXISTING,
            NO_FOLLOW,
        )
    }

    /// Reads the security descriptor's DACL part through a handle.
    fn read_dacl(handle: &impl AsRawHandle) -> io::Result<Local> {
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        // SAFETY: the handle has READ_CONTROL; sd receives a LocalAlloc'd
        // descriptor that dacl points into.
        check_win32(unsafe {
            GetSecurityInfo(
                handle.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut dacl,
                std::ptr::null_mut(),
                &mut sd,
            )
        })?;
        Ok(Local(sd))
    }

    /// The DACL of `path` in SDDL form, for comparing and reporting.
    pub fn dacl_sddl(path: &Path) -> io::Result<String> {
        let handle = open_security(path, READ_CONTROL)?;
        let sd = read_dacl(&handle)?;
        let mut s: PWSTR = std::ptr::null_mut();
        // SAFETY: sd is a valid descriptor; s receives a LocalAlloc'd string.
        let ok = unsafe {
            ConvertSecurityDescriptorToStringSecurityDescriptorW(
                sd.0,
                SDDL_REVISION_1,
                DACL_SECURITY_INFORMATION,
                &mut s,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(take_wide(s))
    }

    /// `CreateDirectoryW` with the DACL given in SDDL (ASM-19).
    pub fn create_dir_with_dacl(path: &Path, sddl: &str) -> io::Result<()> {
        let sd = parse_sddl(sddl)?;
        let sa = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd.0,
            bInheritHandle: 0,
        };
        let p = wide(path.as_os_str());
        // SAFETY: p is NUL-terminated; sa and the descriptor outlive the call.
        if unsafe { CreateDirectoryW(p.as_ptr(), &sa) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Sets the DACL of an existing file from SDDL; `D:P` makes it
    /// protected from inheritance.
    pub fn set_dacl(path: &Path, sddl: &str) -> io::Result<()> {
        let sd = parse_sddl(sddl)?;
        let (dacl, protected) = dacl_of(sd.0)?;
        let handle = open_security(path, WRITE_DAC | READ_CONTROL)?;
        set_dacl_on(&handle, dacl, protected)
    }

    /// 05 §6.4: copies the DACL of `from` to `to` through handles,
    /// keeping whether it is protected (ASM-19).
    pub fn copy_dacl(from: &Path, to: &Path) -> io::Result<()> {
        let source = open_security(from, READ_CONTROL)?;
        let sd = read_dacl(&source)?;
        let (dacl, protected) = dacl_of(sd.0)?;
        let target = open_security(to, WRITE_DAC | READ_CONTROL)?;
        set_dacl_on(&target, dacl, protected)
    }
}
