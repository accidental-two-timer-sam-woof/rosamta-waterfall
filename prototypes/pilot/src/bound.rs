//! P1: binding the last check to the operation (04 §3.3, A10, A16, A20;
//! DC-002, SEC-002).
//!
//! Every directory from home down to the target's parent is opened one
//! component at a time without following links, and each one's identity is
//! compared with what the preflight saw. The operation then goes through the
//! held parent handle rather than through the path:
//!
//! - Unix: `openat` with `O_NOFOLLOW | O_DIRECTORY` from the home directory
//!   handle, then `openat`/`renameat`/`unlinkat` relative to the parent.
//!   A directory handle does not stop another process from renaming the
//!   directory, so after the operation the path is walked again; if it no
//!   longer leads to the held directory, the change is undone through the
//!   same handle.
//! - Windows: each directory is opened with `FILE_FLAG_OPEN_REPARSE_POINT`
//!   (rejecting reparse points) and without `FILE_SHARE_DELETE`, so no one
//!   can rename or delete it while the handle is held. The final path of
//!   each handle is compared with the expected one. Deletion goes through a
//!   handle whose identity was checked (`FileDispositionInfo`).
//!
//! The test harness injects a parent replacement at the [`Sync`] points.

use std::ffi::OsStr;
use std::io;
use std::path::{Component, Path};

/// Which file system object a handle refers to: device and inode on Unix,
/// volume serial number and 128-bit file id on Windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub volume: u64,
    pub file: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    File,
    Dir,
    /// A symbolic link, or on Windows any reparse point.
    Link,
    Other,
}

/// What a directory entry is, observed without following links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub identity: Identity,
    pub kind: Kind,
    /// Unix permission bits; `None` on Windows.
    pub mode: Option<u32>,
}

/// Points where the test harness lets another process replace a directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sync {
    /// The preflight has finished; nothing is open yet.
    AfterPreflight,
    /// The last check before the operation has passed: every directory is
    /// open and the target matched the preflight.
    AfterOpen,
}

impl Sync {
    pub fn name(self) -> &'static str {
        match self {
            Sync::AfterPreflight => "after-preflight",
            Sync::AfterOpen => "after-open",
        }
    }
}

#[derive(Clone, Debug)]
pub enum Op {
    Create(Vec<u8>),
    Update(Vec<u8>),
    Remove,
    /// Read the target for a rollback snapshot or a takeover backup (A16).
    Snapshot,
}

impl Op {
    pub fn name(&self) -> &'static str {
        match self {
            Op::Create(_) => "create",
            Op::Update(_) => "update",
            Op::Remove => "remove",
            Op::Snapshot => "snapshot",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The operation ran and the recheck passed.
    Done { snapshot: Option<Vec<u8>> },
    /// A check failed before anything changed.
    Refused(String),
    /// The operation ran on the held directory, the recheck failed, and the
    /// change was undone through the same handle.
    RolledBack(String),
    /// The snapshot was read through the held handle, but the path no longer
    /// leads there, so the plan is void.
    Invalidated { snapshot: Vec<u8>, reason: String },
}

impl Outcome {
    pub fn name(&self) -> &'static str {
        match self {
            Outcome::Done { .. } => "done",
            Outcome::Refused(_) => "refused",
            Outcome::RolledBack(_) => "rolled-back",
            Outcome::Invalidated { .. } => "invalidated",
        }
    }
}

/// What the preflight recorded: the identity of home and of every directory
/// below it on the way to the target, and the target if it existed.
#[derive(Clone, Debug)]
pub struct Preflight {
    pub dirs: Vec<Identity>,
    pub target: Option<Entry>,
}

impl Preflight {
    pub fn take(home: &Path, rel: &Path) -> io::Result<Preflight> {
        let (parents, name) = split(rel)?;
        let chain = Chain::open(home, &parents, None).map_err(io::Error::other)?;
        let dirs = chain
            .dirs
            .iter()
            .map(Dir::identity)
            .collect::<io::Result<_>>()?;
        let target = chain.parent().entry(name)?;
        if let Some(entry) = target
            && entry.kind != Kind::File
        {
            return Err(io::Error::other(format!("target is {:?}", entry.kind)));
        }
        Ok(Preflight { dirs, target })
    }
}

/// Runs one operation on `home/rel`, calling `hook` at each [`Sync`] point.
pub fn run(
    home: &Path,
    rel: &Path,
    pre: &Preflight,
    op: &Op,
    hook: &mut dyn FnMut(Sync),
) -> io::Result<Outcome> {
    let (parents, name) = split(rel)?;
    hook(Sync::AfterPreflight);

    let chain = match Chain::open(home, &parents, Some(&pre.dirs)) {
        Ok(chain) => chain,
        Err(reason) => return Ok(Outcome::Refused(reason)),
    };
    let dir = chain.parent();
    let current = dir.entry(name)?;
    let expected = match (op, current, pre.target) {
        (Op::Create(_), None, None) => None,
        (Op::Create(_), Some(_), _) => return Ok(Outcome::Refused("target appeared".into())),
        (_, Some(now), Some(then)) if now == then => Some(then),
        _ => return Ok(Outcome::Refused("target differs from the preflight".into())),
    };
    hook(Sync::AfterOpen);

    let recheck = |written: Option<Identity>| -> Result<(), String> {
        let again = Chain::open(home, &parents, Some(&pre.dirs))?;
        let held = dir.identity().map_err(|e| e.to_string())?;
        let found = again.parent().identity().map_err(|e| e.to_string())?;
        if held != found {
            return Err("path leads to another directory".into());
        }
        let entry = again.parent().entry(name).map_err(|e| e.to_string())?;
        match (written, entry) {
            (Some(id), Some(e)) if e.identity == id => Ok(()),
            (None, None) => Ok(()),
            (_, e) => Err(format!("target is now {e:?}")),
        }
    };

    Ok(match op {
        Op::Snapshot => {
            let expected = expected.expect("snapshot needs a target");
            let bytes = match dir.read_file(name, expected.identity) {
                Ok(bytes) => bytes,
                Err(e) => return Ok(Outcome::Refused(e.to_string())),
            };
            match recheck(Some(expected.identity)) {
                Ok(()) => Outcome::Done {
                    snapshot: Some(bytes),
                },
                Err(reason) => Outcome::Invalidated {
                    snapshot: bytes,
                    reason,
                },
            }
        }
        Op::Create(content) => {
            let written = dir.write(name, content, None)?;
            match recheck(Some(written)) {
                Ok(()) => Outcome::Done { snapshot: None },
                Err(reason) => {
                    dir.remove(name, written)?;
                    Outcome::RolledBack(reason)
                }
            }
        }
        Op::Update(content) => {
            let expected = expected.expect("update needs a target");
            let saved = match dir.read_file(name, expected.identity) {
                Ok(bytes) => bytes,
                Err(e) => return Ok(Outcome::Refused(e.to_string())),
            };
            let written = dir.write(name, content, expected.mode)?;
            match recheck(Some(written)) {
                Ok(()) => Outcome::Done { snapshot: None },
                Err(reason) => {
                    dir.write(name, &saved, expected.mode)?;
                    Outcome::RolledBack(reason)
                }
            }
        }
        Op::Remove => {
            let expected = expected.expect("remove needs a target");
            let saved = match dir.read_file(name, expected.identity) {
                Ok(bytes) => bytes,
                Err(e) => return Ok(Outcome::Refused(e.to_string())),
            };
            dir.remove(name, expected.identity)?;
            match recheck(None) {
                Ok(()) => Outcome::Done { snapshot: None },
                Err(reason) => {
                    dir.write(name, &saved, expected.mode)?;
                    Outcome::RolledBack(reason)
                }
            }
        }
    })
}

fn split(rel: &Path) -> io::Result<(Vec<&OsStr>, &OsStr)> {
    let mut parts = Vec::new();
    for component in rel.components() {
        match component {
            Component::Normal(part) => parts.push(part),
            _ => return Err(io::Error::other("path must be relative and normalized")),
        }
    }
    let name = parts.pop().ok_or_else(|| io::Error::other("empty path"))?;
    Ok((parts, name))
}

/// Home and every directory below it on the way to the target, all held
/// open for as long as the chain lives.
struct Chain {
    dirs: Vec<Dir>,
}

impl Chain {
    /// Opens each component without following links. With `expected`, every
    /// directory's identity must match the preflight.
    fn open(
        home: &Path,
        parents: &[&OsStr],
        expected: Option<&[Identity]>,
    ) -> Result<Chain, String> {
        let check = |i: usize, dir: &Dir| -> Result<(), String> {
            if let Some(expected) = expected {
                let id = dir.identity().map_err(|e| e.to_string())?;
                if expected.get(i) != Some(&id) {
                    return Err(format!("directory {i} differs from the preflight"));
                }
            }
            Ok(())
        };
        let root = Dir::open_root(home).map_err(|e| format!("home: {e}"))?;
        check(0, &root)?;
        let mut dirs = vec![root];
        for (i, name) in parents.iter().enumerate() {
            let next = dirs[i]
                .open_child(name)
                .map_err(|e| format!("{}: {e}", name.to_string_lossy()))?;
            check(i + 1, &next)?;
            dirs.push(next);
        }
        Ok(Chain { dirs })
    }

    fn parent(&self) -> &Dir {
        self.dirs.last().expect("chain holds at least home")
    }
}

#[cfg(unix)]
pub use unix::Dir;
#[cfg(windows)]
pub use windows::Dir;

#[cfg(unix)]
pub(crate) mod unix {
    use super::{Entry, Identity, Kind};
    use rustix::fs::{AtFlags, CWD, FileType, Mode, OFlags, Stat};
    use std::ffi::OsStr;
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
    use std::path::Path;

    pub struct Dir {
        fd: OwnedFd,
    }

    // The casts are no-ops on Linux but not on macOS (`st_dev` is `i32`,
    // `st_mode` is `u16`).
    #[allow(clippy::unnecessary_cast)]
    pub(crate) fn entry_of(st: &Stat) -> Entry {
        let kind = match FileType::from_raw_mode(st.st_mode) {
            FileType::RegularFile => Kind::File,
            FileType::Directory => Kind::Dir,
            FileType::Symlink => Kind::Link,
            _ => Kind::Other,
        };
        Entry {
            identity: Identity {
                volume: st.st_dev as u64,
                file: st.st_ino as u128,
            },
            kind,
            mode: Some(st.st_mode as u32 & 0o7777),
        }
    }

    impl Dir {
        /// Home itself is trusted as given; links in its own path are the
        /// subject of the home-normalization rules (05 §6), not of P1.
        pub fn open_root(path: &Path) -> io::Result<Dir> {
            let fd = rustix::fs::openat(
                CWD,
                path,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
            )?;
            Ok(Dir { fd })
        }

        /// A symlink component fails with `ELOOP` (or `ENOTDIR`).
        pub fn open_child(&self, name: &OsStr) -> io::Result<Dir> {
            let fd = rustix::fs::openat(
                &self.fd,
                name,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )?;
            Ok(Dir { fd })
        }

        pub fn identity(&self) -> io::Result<Identity> {
            Ok(entry_of(&rustix::fs::fstat(&self.fd)?).identity)
        }

        pub fn fd(&self) -> BorrowedFd<'_> {
            self.fd.as_fd()
        }

        pub fn entry(&self, name: &OsStr) -> io::Result<Option<Entry>> {
            match rustix::fs::statat(&self.fd, name, AtFlags::SYMLINK_NOFOLLOW) {
                Ok(st) => Ok(Some(entry_of(&st))),
                Err(rustix::io::Errno::NOENT) => Ok(None),
                Err(e) => Err(e.into()),
            }
        }

        /// Reads the file that was actually opened, after checking it is the
        /// expected regular file (A16).
        pub fn read_file(&self, name: &OsStr, expected: Identity) -> io::Result<Vec<u8>> {
            let fd = rustix::fs::openat(
                &self.fd,
                name,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )?;
            let opened = entry_of(&rustix::fs::fstat(&fd)?);
            if opened.kind != Kind::File || opened.identity != expected {
                return Err(io::Error::other("opened file differs from the preflight"));
            }
            let mut bytes = Vec::new();
            File::from(fd).read_to_end(&mut bytes)?;
            Ok(bytes)
        }

        /// Writes a `0600` temporary file in this directory, then renames it
        /// over `name`. Returns the identity of the written file.
        pub fn write(
            &self,
            name: &OsStr,
            content: &[u8],
            mode: Option<u32>,
        ) -> io::Result<Identity> {
            let temp = format!(
                ".{}{}{}",
                name.to_string_lossy(),
                crate::atomic::TEMP_MARKER,
                std::process::id()
            );
            let fd = rustix::fs::openat(
                &self.fd,
                &temp,
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o600),
            )?;
            let result = (|| {
                if let Some(mode) = mode {
                    rustix::fs::fchmod(&fd, Mode::from_raw_mode(mode as _))?;
                }
                let identity = entry_of(&rustix::fs::fstat(&fd)?).identity;
                let mut file = File::from(fd);
                file.write_all(content)?;
                file.sync_all()?;
                drop(file);
                rustix::fs::renameat(&self.fd, &temp, &self.fd, name)?;
                rustix::fs::fsync(&self.fd)?;
                Ok(identity)
            })();
            if result.is_err() {
                let _ = rustix::fs::unlinkat(&self.fd, &temp, AtFlags::empty());
            }
            result
        }

        /// Unlinks `name` if it is still the expected file. Unix has no
        /// unlink-by-handle, so a window remains between the check and the
        /// unlink (05 §6).
        pub fn remove(&self, name: &OsStr, expected: Identity) -> io::Result<()> {
            match self.entry(name)? {
                Some(e) if e.identity == expected => {}
                _ => return Err(io::Error::other("target differs before unlink")),
            }
            rustix::fs::unlinkat(&self.fd, name, AtFlags::empty())?;
            rustix::fs::fsync(&self.fd)?;
            Ok(())
        }
    }
}

#[cfg(windows)]
pub(crate) mod windows {
    use super::{Entry, Identity, Kind};
    use std::ffi::{OsStr, OsString};
    use std::fs::File;
    use std::io::{self, Read, Write};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::os::windows::io::{AsHandle, AsRawHandle, BorrowedHandle, FromRawHandle, OwnedHandle};
    use std::path::{Path, PathBuf};
    use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CREATE_NEW, CreateFileW, DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_TAG_INFO, FILE_DISPOSITION_INFO,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_ID_INFO,
        FILE_LIST_DIRECTORY, FILE_NAME_NORMALIZED, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE,
        FILE_SHARE_READ, FILE_SHARE_WRITE, FileAttributeTagInfo, FileDispositionInfo, FileIdInfo,
        GetFileInformationByHandleEx, GetFinalPathNameByHandleW, MOVEFILE_REPLACE_EXISTING,
        MOVEFILE_WRITE_THROUGH, MoveFileExW, OPEN_EXISTING, SYNCHRONIZE,
        SetFileInformationByHandle, VOLUME_NAME_DOS,
    };

    pub struct Dir {
        handle: OwnedHandle,
        path: PathBuf,
    }

    fn wide(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub(crate) fn open(
        path: &Path,
        access: u32,
        share: u32,
        disposition: u32,
        flags: u32,
    ) -> io::Result<OwnedHandle> {
        let path = wide(path);
        // SAFETY: path is a NUL-terminated UTF-16 buffer that outlives the call.
        let handle = unsafe {
            CreateFileW(
                path.as_ptr(),
                access,
                share,
                std::ptr::null(),
                disposition,
                flags,
                std::ptr::null_mut(),
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: CreateFileW returned a valid handle that we now own.
        Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
    }

    pub(crate) fn info<T>(handle: &impl AsRawHandle, class: i32) -> io::Result<T> {
        let mut value = std::mem::MaybeUninit::<T>::zeroed();
        // SAFETY: the buffer is a T of the size passed in, matching `class`.
        let ok = unsafe {
            GetFileInformationByHandleEx(
                handle.as_raw_handle(),
                class,
                value.as_mut_ptr().cast(),
                size_of::<T>() as u32,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: the call succeeded and filled the buffer.
        Ok(unsafe { value.assume_init() })
    }

    pub(crate) fn entry_of(handle: &impl AsRawHandle) -> io::Result<Entry> {
        let tag: FILE_ATTRIBUTE_TAG_INFO = info(handle, FileAttributeTagInfo)?;
        let id: FILE_ID_INFO = info(handle, FileIdInfo)?;
        let kind = if tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            Kind::Link
        } else if tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            Kind::Dir
        } else {
            Kind::File
        };
        Ok(Entry {
            identity: Identity {
                volume: id.VolumeSerialNumber,
                file: u128::from_le_bytes(id.FileId.Identifier),
            },
            kind,
            mode: None,
        })
    }

    fn final_path(handle: &impl AsRawHandle) -> io::Result<OsString> {
        let mut buf = vec![0u16; 1024];
        // SAFETY: buf has the length passed in.
        let len = unsafe {
            GetFinalPathNameByHandleW(
                handle.as_raw_handle(),
                buf.as_mut_ptr(),
                buf.len() as u32,
                FILE_NAME_NORMALIZED | VOLUME_NAME_DOS,
            )
        };
        if len == 0 || len as usize >= buf.len() {
            return Err(io::Error::last_os_error());
        }
        Ok(OsString::from_wide(&buf[..len as usize]))
    }

    /// Directory handles leave out `FILE_SHARE_DELETE`, so the directory
    /// cannot be renamed or deleted while held. The share mode only counts
    /// when the handle has data access: a handle opened with nothing but
    /// `FILE_READ_ATTRIBUTES | SYNCHRONIZE` takes no part in sharing checks,
    /// and the windows-2025 runner let another process rename the directory.
    /// `FILE_LIST_DIRECTORY` is the smallest access that counts; it shares
    /// read and write, so our own work inside the directory is not blocked.
    const DIR_ACCESS: u32 = FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES | SYNCHRONIZE;
    const DIR_SHARE: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE;
    pub(crate) const NO_FOLLOW: u32 = FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT;

    impl Dir {
        pub fn open_root(path: &Path) -> io::Result<Dir> {
            let handle = open(
                path,
                DIR_ACCESS,
                DIR_SHARE,
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS,
            )?;
            Ok(Dir {
                handle,
                path: path.to_path_buf(),
            })
        }

        pub fn open_child(&self, name: &OsStr) -> io::Result<Dir> {
            let path = self.path.join(name);
            let handle = open(&path, DIR_ACCESS, DIR_SHARE, OPEN_EXISTING, NO_FOLLOW)?;
            match entry_of(&handle)?.kind {
                Kind::Dir => {}
                kind => return Err(io::Error::other(format!("not a plain directory: {kind:?}"))),
            }
            let child = Dir { handle, path };
            child.check_final_path(&self.handle, name)?;
            Ok(child)
        }

        /// The final path of this handle must be the parent's final path
        /// plus `name`.
        fn check_final_path(&self, parent: &OwnedHandle, name: &OsStr) -> io::Result<()> {
            let mut expected = final_path(parent)?;
            expected.push("\\");
            expected.push(name);
            let actual = final_path(&self.handle)?;
            if actual != expected {
                return Err(io::Error::other(format!(
                    "final path {actual:?}, expected {expected:?}"
                )));
            }
            Ok(())
        }

        pub fn identity(&self) -> io::Result<Identity> {
            Ok(entry_of(&self.handle)?.identity)
        }

        pub fn handle(&self) -> BorrowedHandle<'_> {
            self.handle.as_handle()
        }

        pub fn path(&self) -> &Path {
            &self.path
        }

        pub fn entry(&self, name: &OsStr) -> io::Result<Option<Entry>> {
            let share = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
            match open(
                &self.path.join(name),
                FILE_READ_ATTRIBUTES,
                share,
                OPEN_EXISTING,
                NO_FOLLOW,
            ) {
                Ok(handle) => Ok(Some(entry_of(&handle)?)),
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e),
            }
        }

        pub fn read_file(&self, name: &OsStr, expected: Identity) -> io::Result<Vec<u8>> {
            let handle = open(
                &self.path.join(name),
                GENERIC_READ,
                FILE_SHARE_READ,
                OPEN_EXISTING,
                NO_FOLLOW,
            )?;
            let opened = entry_of(&handle)?;
            if opened.kind != Kind::File || opened.identity != expected {
                return Err(io::Error::other("opened file differs from the preflight"));
            }
            let mut bytes = Vec::new();
            File::from(handle).read_to_end(&mut bytes)?;
            Ok(bytes)
        }

        /// Path-based, which is safe only because every directory on the way
        /// is held without `FILE_SHARE_DELETE`. The temporary file's final
        /// path is checked as well, and a misplaced one is deleted through
        /// its handle.
        pub fn write(
            &self,
            name: &OsStr,
            content: &[u8],
            _mode: Option<u32>,
        ) -> io::Result<Identity> {
            let temp_name = format!(
                ".{}{}{}",
                name.to_string_lossy(),
                crate::atomic::TEMP_MARKER,
                std::process::id()
            );
            let temp = self.path.join(&temp_name);
            let handle = open(
                &temp,
                GENERIC_WRITE | GENERIC_READ | DELETE,
                0,
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL,
            )?;
            let mut file = File::from(handle);
            let prepared = (|| {
                let mut expected = final_path(&self.handle)?;
                expected.push("\\");
                expected.push(&temp_name);
                let actual = final_path(&file)?;
                if actual != expected {
                    return Err(io::Error::other(format!(
                        "temporary file landed at {actual:?}, expected {expected:?}"
                    )));
                }
                let identity = entry_of(&file)?.identity;
                file.write_all(content)?;
                file.sync_all()?;
                Ok(identity)
            })();
            let identity = match prepared {
                Ok(identity) => identity,
                Err(e) => {
                    // Delete through the handle: the path may lead elsewhere.
                    let _ = delete_on_close(&file);
                    return Err(e);
                }
            };
            drop(file);
            let (from, to) = (wide(&temp), wide(&self.path.join(name)));
            // SAFETY: both are NUL-terminated UTF-16 buffers that outlive the call.
            let ok = unsafe {
                MoveFileExW(
                    from.as_ptr(),
                    to.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            };
            if ok == 0 {
                let err = io::Error::last_os_error();
                let _ = std::fs::remove_file(&temp);
                return Err(err);
            }
            Ok(identity)
        }

        /// Opens the target with `DELETE`, checks its identity, and deletes
        /// that handle's file.
        pub fn remove(&self, name: &OsStr, expected: Identity) -> io::Result<()> {
            let handle = open(
                &self.path.join(name),
                DELETE | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                FILE_SHARE_READ,
                OPEN_EXISTING,
                NO_FOLLOW,
            )?;
            if entry_of(&handle)?.identity != expected {
                return Err(io::Error::other("target differs before delete"));
            }
            delete_on_close(&handle)
        }
    }

    /// Marks the handle's file for deletion when the handle closes. The
    /// handle needs `DELETE` access.
    pub(crate) fn delete_on_close(handle: &impl AsRawHandle) -> io::Result<()> {
        let info = FILE_DISPOSITION_INFO { DeleteFile: true };
        // SAFETY: info is a FILE_DISPOSITION_INFO of the size passed in.
        let ok = unsafe {
            SetFileInformationByHandle(
                handle.as_raw_handle(),
                FileDispositionInfo,
                (&info as *const FILE_DISPOSITION_INFO).cast(),
                size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        };
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

/// How the injected process replaces a directory (04 §5 A10, A16, A20).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swap {
    /// Move the directory aside (inside home) and put a link to a directory
    /// outside home in its place. Windows uses a junction, which a standard
    /// user can create without developer mode.
    LinkOut,
    /// Move the directory aside and move another directory inside home,
    /// holding a same-named file, into its place.
    DirIn,
    /// Move the directory itself outside home and leave an empty directory
    /// in its place.
    MoveOut,
}

impl Swap {
    pub fn name(self) -> &'static str {
        match self {
            Swap::LinkOut => "link-out",
            Swap::DirIn => "dir-in",
            Swap::MoveOut => "move-out",
        }
    }

    pub fn parse(name: &str) -> Option<Swap> {
        [Swap::LinkOut, Swap::DirIn, Swap::MoveOut]
            .into_iter()
            .find(|s| s.name() == name)
    }

    /// Replaces `victim`. `aside` is where it goes for `LinkOut` and `DirIn`;
    /// `other` is the link target, the replacement directory, or the
    /// destination outside home.
    pub fn apply(self, victim: &Path, aside: &Path, other: &Path) -> io::Result<()> {
        match self {
            Swap::LinkOut => {
                std::fs::rename(victim, aside)?;
                make_dir_link(victim, other)
            }
            Swap::DirIn => {
                std::fs::rename(victim, aside)?;
                std::fs::rename(other, victim)
            }
            Swap::MoveOut => {
                std::fs::rename(victim, other)?;
                std::fs::create_dir(victim)
            }
        }
    }
}

#[cfg(unix)]
pub fn make_dir_link(link: &Path, target: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
pub fn make_dir_link(link: &Path, target: &Path) -> io::Result<()> {
    let status = std::process::Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(link)
        .arg(target)
        .stdout(std::process::Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("mklink /J failed: {status}")))
    }
}

/// Keeps replacing `victim` and putting it back until `stop` exists; used
/// by the concurrent test. Each step is retried until it succeeds, so a
/// blocked step (Windows) only delays the cycle. Returns (cycles, blocked).
pub fn churn(swap: Swap, victim: &Path, aside: &Path, other: &Path, stop: &Path) -> (u64, u64) {
    let steps: [&dyn Fn() -> io::Result<()>; 4] = match swap {
        Swap::DirIn => [
            &|| std::fs::rename(victim, aside),
            &|| std::fs::rename(other, victim),
            &|| std::fs::rename(victim, other),
            &|| std::fs::rename(aside, victim),
        ],
        Swap::LinkOut => [
            &|| std::fs::rename(victim, aside),
            &|| make_dir_link(victim, other),
            &|| remove_dir_link(victim),
            &|| std::fs::rename(aside, victim),
        ],
        Swap::MoveOut => panic!("churn does not support move-out"),
    };
    let (mut cycles, mut blocked) = (0, 0);
    let mut state = 0;
    loop {
        if state == 0 && stop.exists() {
            return (cycles, blocked);
        }
        match steps[state]() {
            Ok(()) => {
                state = (state + 1) % 4;
                if state == 0 {
                    cycles += 1;
                }
            }
            Err(_) => blocked += 1,
        }
    }
}

#[cfg(unix)]
fn remove_dir_link(link: &Path) -> io::Result<()> {
    std::fs::remove_file(link)
}

#[cfg(windows)]
fn remove_dir_link(link: &Path) -> io::Result<()> {
    std::fs::remove_dir(link)
}
