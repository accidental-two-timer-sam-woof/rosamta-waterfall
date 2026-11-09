//! P3: taking over a symbolic link at a target path, and restoring it
//! (SEC-002, DC-001, DC-003; 04 §3.3, §5 A3, A6).
//!
//! Only the link itself is kept; the file it points to is never opened. The
//! link is not copied either: copying means creating a new symbolic link,
//! which a Windows standard user may only do with developer mode on (see
//! [`create`]). Instead:
//!
//! 1. Backup: a hard link to the symbolic link itself is made in the backup
//!    directory. The target path still holds the link.
//! 2. Takeover: the new regular file replaces the link in one rename
//!    ([`crate::bound::Dir::write`], P2), so the target path always holds
//!    either the link or the complete new file. The backup is now the link's
//!    only name.
//! 3. Restore: after checking that the target path still holds the file
//!    this batch wrote, the backup is renamed over it.
//!
//! Each step goes through directory handles held by the caller (P1) and
//! checks the identity of what it is about to move. A hard link and a rename
//! only work within one file system, so the backup directory must be on the
//! same one as the target.

use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

use crate::bound::{Dir, Identity, Kind};

/// Makes `backup_name` in `backup_dir` a second name of the symbolic link
/// `name` in `dir`, which must still be `expected`.
pub fn backup(
    dir: &Dir,
    name: &OsStr,
    expected: Identity,
    backup_dir: &Dir,
    backup_name: &OsStr,
) -> io::Result<()> {
    imp::backup(dir, name, expected, backup_dir, backup_name)
}

/// Renames the backup over `name` in `dir`. The backup must still be the
/// link `link`, and `name` must still be the regular file `written`.
pub fn restore(
    backup_dir: &Dir,
    backup_name: &OsStr,
    link: Identity,
    dir: &Dir,
    name: &OsStr,
    written: Identity,
) -> io::Result<()> {
    match dir.entry(name)? {
        Some(e) if e.kind == Kind::File && e.identity == written => {}
        other => {
            return Err(io::Error::other(format!(
                "target is not the file this batch wrote: {other:?}"
            )));
        }
    }
    imp::restore(backup_dir, backup_name, link, dir, name)
}

/// What the link at `name` in `dir` points to, read from the link itself.
pub fn read_target(dir: &Dir, name: &OsStr) -> io::Result<PathBuf> {
    imp::read_target(dir, name)
}

/// Creates a new symbolic link to a file, the way a copy-based backup or
/// restore would have to. On Windows this needs `SeCreateSymbolicLinkPrivilege`
/// or developer mode (std passes `SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE`).
pub fn create(link: &Path, target: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link)
    }
}

#[cfg(unix)]
mod imp {
    use super::{Dir, Identity, Kind};
    use rustix::fs::AtFlags;
    use std::ffi::{OsStr, OsString};
    use std::io;
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;

    fn check_link(dir: &Dir, name: &OsStr, expected: Identity) -> io::Result<()> {
        match dir.entry(name)? {
            Some(e) if e.kind == Kind::Link && e.identity == expected => Ok(()),
            other => Err(io::Error::other(format!(
                "not the expected link: {other:?}"
            ))),
        }
    }

    /// `linkat` without `AT_SYMLINK_FOLLOW` links the symbolic link itself
    /// (POSIX.1-2008; Linux and macOS both follow it). A replacement between
    /// the check and the call is caught afterwards and undone.
    pub fn backup(
        dir: &Dir,
        name: &OsStr,
        expected: Identity,
        backup_dir: &Dir,
        backup_name: &OsStr,
    ) -> io::Result<()> {
        check_link(dir, name, expected)?;
        rustix::fs::linkat(
            dir.fd(),
            name,
            backup_dir.fd(),
            backup_name,
            AtFlags::empty(),
        )?;
        if let Err(e) = check_link(backup_dir, backup_name, expected) {
            let _ = rustix::fs::unlinkat(backup_dir.fd(), backup_name, AtFlags::empty());
            return Err(e);
        }
        rustix::fs::fsync(backup_dir.fd())?;
        Ok(())
    }

    /// The window between the checks and `renameat` is the same as for
    /// `Dir::remove` (05 §6).
    pub fn restore(
        backup_dir: &Dir,
        backup_name: &OsStr,
        link: Identity,
        dir: &Dir,
        name: &OsStr,
    ) -> io::Result<()> {
        check_link(backup_dir, backup_name, link)?;
        rustix::fs::renameat(backup_dir.fd(), backup_name, dir.fd(), name)?;
        rustix::fs::fsync(dir.fd())?;
        rustix::fs::fsync(backup_dir.fd())?;
        Ok(())
    }

    pub fn read_target(dir: &Dir, name: &OsStr) -> io::Result<PathBuf> {
        let target = rustix::fs::readlinkat(dir.fd(), name, Vec::new())?;
        Ok(PathBuf::from(OsString::from_vec(target.into_bytes())))
    }
}

#[cfg(windows)]
mod imp {
    use super::{Dir, Identity, Kind};
    use crate::bound::windows::{NO_FOLLOW, entry_of, info, open};
    use std::ffi::OsStr;
    use std::io;
    use std::mem::offset_of;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{AsRawHandle, OwnedHandle};
    use std::path::PathBuf;
    use windows_sys::Wdk::Storage::FileSystem::{
        FILE_INFORMATION_CLASS, FILE_LINK_INFORMATION, FILE_RENAME_INFORMATION,
        FileLinkInformation, FileRenameInformation, NtSetInformationFile,
    };
    use windows_sys::Win32::Foundation::{HANDLE, RtlNtStatusToDosError};
    use windows_sys::Win32::Storage::FileSystem::{
        DELETE, FILE_ATTRIBUTE_TAG_INFO, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES, FileAttributeTagInfo, OPEN_EXISTING, SYNCHRONIZE,
    };
    use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;

    /// `IO_REPARSE_TAG_SYMLINK`. Junctions and other reparse points are not
    /// symbolic links and are not taken over.
    const SYMLINK_TAG: u32 = 0xA000_000C;

    // Both structures have the same layout; `set_name` fills either.
    const _: () = assert!(
        offset_of!(FILE_LINK_INFORMATION, FileName)
            == offset_of!(FILE_RENAME_INFORMATION, FileName)
    );

    /// Opens the reparse point itself (not what it points to) and checks it
    /// is the expected symbolic link.
    fn open_link(
        dir: &Dir,
        name: &OsStr,
        access: u32,
        expected: Identity,
    ) -> io::Result<OwnedHandle> {
        let share = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
        let handle = open(
            &dir.path().join(name),
            access,
            share,
            OPEN_EXISTING,
            NO_FOLLOW,
        )?;
        let entry = entry_of(&handle)?;
        let tag: FILE_ATTRIBUTE_TAG_INFO = info(&handle, FileAttributeTagInfo)?;
        if entry.kind != Kind::Link || tag.ReparseTag != SYMLINK_TAG || entry.identity != expected {
            return Err(io::Error::other(format!(
                "not the expected symbolic link: {entry:?}, tag {:#x}",
                tag.ReparseTag
            )));
        }
        Ok(handle)
    }

    /// Gives the handle's file the name `name` under the directory `root`
    /// (`FileLinkInformation` adds a name, `FileRenameInformation` moves it).
    /// A relative name under a held directory handle keeps the operation
    /// bound to that directory.
    fn set_name(
        handle: &OwnedHandle,
        class: FILE_INFORMATION_CLASS,
        replace: bool,
        root: HANDLE,
        name: &OsStr,
    ) -> io::Result<()> {
        let name: Vec<u16> = name.encode_wide().collect();
        let offset = offset_of!(FILE_RENAME_INFORMATION, FileName);
        let size = (offset + name.len() * 2).max(size_of::<FILE_RENAME_INFORMATION>());
        let mut buf = vec![0u64; size.div_ceil(8)];
        let info = buf.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
        // SAFETY: buf is zeroed, 8-byte aligned, and large enough for the
        // header plus the name.
        unsafe {
            (*info).Anonymous.ReplaceIfExists = replace;
            (*info).RootDirectory = root;
            (*info).FileNameLength = (name.len() * 2) as u32;
            std::ptr::copy_nonoverlapping(
                name.as_ptr(),
                (&raw mut (*info).FileName).cast::<u16>(),
                name.len(),
            );
        }
        // SAFETY: an all-zero IO_STATUS_BLOCK is valid.
        let mut status_block: IO_STATUS_BLOCK = unsafe { std::mem::zeroed() };
        // SAFETY: the handle is open, buf holds a valid structure of `size`
        // bytes, and both outlive the call.
        let status = unsafe {
            NtSetInformationFile(
                handle.as_raw_handle(),
                &mut status_block,
                buf.as_ptr().cast(),
                size as u32,
                class,
            )
        };
        if status < 0 {
            // SAFETY: plain conversion of a status code.
            let code = unsafe { RtlNtStatusToDosError(status) };
            return Err(io::Error::from_raw_os_error(code as i32));
        }
        Ok(())
    }

    pub fn backup(
        dir: &Dir,
        name: &OsStr,
        expected: Identity,
        backup_dir: &Dir,
        backup_name: &OsStr,
    ) -> io::Result<()> {
        let access = FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES | SYNCHRONIZE;
        let handle = open_link(dir, name, access, expected)?;
        set_name(
            &handle,
            FileLinkInformation,
            false,
            backup_dir.handle().as_raw_handle(),
            backup_name,
        )
    }

    pub fn restore(
        backup_dir: &Dir,
        backup_name: &OsStr,
        link: Identity,
        dir: &Dir,
        name: &OsStr,
    ) -> io::Result<()> {
        let access = DELETE | FILE_READ_ATTRIBUTES | SYNCHRONIZE;
        let handle = open_link(backup_dir, backup_name, access, link)?;
        set_name(
            &handle,
            FileRenameInformation,
            true,
            dir.handle().as_raw_handle(),
            name,
        )
    }

    pub fn read_target(dir: &Dir, name: &OsStr) -> io::Result<PathBuf> {
        std::fs::read_link(dir.path().join(name))
    }
}
