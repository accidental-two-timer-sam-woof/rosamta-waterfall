//! Handle-relative filesystem operations for Linux and macOS (05 §7.3).

use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Read},
    os::fd::OwnedFd,
    path::Path,
};

use rosamta_core::fs::{Entry, Kind};
use rustix::fs::{self, AtFlags, CWD, FileType, Mode, OFlags, Stat};

pub(super) struct Dir(OwnedFd);

pub(super) fn creation_mode() -> u32 {
    let current = rustix::process::umask(Mode::from_raw_mode(0o777));
    rustix::process::umask(current);
    0o666 & !current.as_raw_mode()
}

impl Dir {
    pub(super) fn enter_child_process(&self, command: &mut std::process::Command) {
        use std::os::{
            fd::{AsRawFd, BorrowedFd},
            unix::process::CommandExt,
        };
        let fd = self.0.as_raw_fd();
        // fchdir is async-signal-safe. The caller keeps the directory open until Git exits.
        unsafe {
            command.pre_exec(move || {
                rustix::process::fchdir(BorrowedFd::borrow_raw(fd)).map_err(Into::into)
            });
        }
    }

    pub(super) fn publish(
        &self,
        staging: &OsStr,
        destination: &OsStr,
        identity: &str,
    ) -> io::Result<()> {
        if self.child(staging)?.id()? != identity {
            return Err(io::Error::other("staging directory changed"));
        }
        self.rename(staging, destination, fs::RenameFlags::NOREPLACE)?;
        if self.child(destination)?.id()? != identity {
            self.rename(destination, staging, fs::RenameFlags::NOREPLACE)?;
            return Err(io::Error::other(
                "staging directory changed while publishing",
            ));
        }
        Ok(())
    }
    pub(super) fn home(path: &Path) -> io::Result<Self> {
        Ok(Self(fs::openat(
            CWD,
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )?))
    }

    pub(super) fn child(&self, name: &OsStr) -> io::Result<Self> {
        Ok(Self(fs::openat(
            &self.0,
            name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::empty(),
        )?))
    }

    pub(super) fn id(&self) -> io::Result<String> {
        Ok(identity(&fs::fstat(&self.0)?))
    }

    #[cfg_attr(target_os = "linux", allow(clippy::useless_conversion))] // macOS st_mode is u16.
    pub(super) fn entry(&self, name: &OsStr) -> io::Result<Option<Entry>> {
        let before = match fs::statat(&self.0, name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(stat) => stat,
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        let kind = kind(&before);
        let mut bytes = Vec::new();
        if kind == Kind::Regular {
            let fd = fs::openat(
                &self.0,
                name,
                OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
                Mode::empty(),
            )?;
            let opened = fs::fstat(&fd)?;
            if FileType::from_raw_mode(opened.st_mode) != FileType::RegularFile
                || version(&opened) != version(&before)
            {
                return Err(io::Error::other("file changed while opening it"));
            }
            let mut file = File::from(fd);
            file.read_to_end(&mut bytes)?;
            if version(&fs::fstat(&file)?) != version(&opened) {
                return Err(io::Error::other("file changed while reading it"));
            }
        }
        Ok(Some(Entry {
            kind,
            identity: identity(&before),
            parent: self.id()?,
            bytes,
            mode: Some(u32::from(before.st_mode) & 0o7777),
        }))
    }

    pub(super) fn names(&self) -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::Dir::read_from(&self.0)? {
            let entry = entry?;
            let name = entry.file_name().to_str().map_err(io::Error::other)?;
            if !matches!(name, "." | "..") {
                names.push(name.to_owned());
            }
        }
        Ok(names)
    }
}

fn kind(stat: &Stat) -> Kind {
    match FileType::from_raw_mode(stat.st_mode) {
        FileType::RegularFile => Kind::Regular,
        FileType::Directory => Kind::Directory,
        FileType::Symlink => Kind::Symlink,
        _ => Kind::Other,
    }
}

fn identity(stat: &Stat) -> String {
    format!("{:016x}:{:016x}", stat.st_dev, stat.st_ino)
}

fn version(stat: &Stat) -> (String, i128, i128, i128, i128, i128) {
    (
        identity(stat),
        stat.st_size.into(),
        stat.st_mtime.into(),
        stat.st_mtime_nsec.into(),
        stat.st_ctime.into(),
        stat.st_ctime_nsec.into(),
    )
}

impl Dir {
    pub(super) fn mkdir(&self, name: &OsStr) -> io::Result<()> {
        Ok(fs::mkdirat(&self.0, name, Mode::from_raw_mode(0o700))?)
    }

    pub(super) fn rmdir(&self, name: &OsStr) -> io::Result<()> {
        Ok(fs::unlinkat(&self.0, name, AtFlags::REMOVEDIR)?)
    }

    pub(super) fn lock(&self, name: &OsStr) -> io::Result<File> {
        let fd = fs::openat(
            &self.0,
            name,
            OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
            Mode::from_raw_mode(0o600),
        )?;
        if kind(&fs::fstat(&fd)?) != Kind::Regular {
            return Err(io::Error::other("lock is not a regular file"));
        }
        fs::flock(&fd, fs::FlockOperation::NonBlockingLockExclusive)?;
        Ok(File::from(fd))
    }

    pub(super) fn change(
        &self,
        name: &OsStr,
        expected: Option<&Entry>,
        replacement: Option<rosamta_core::fs::Replacement<'_>>,
        creation_mode: u32,
        verify: impl Fn() -> io::Result<()>,
    ) -> io::Result<Option<Entry>> {
        let actual = self.entry(name)?;
        if expected.is_none() && actual.is_some() {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        if actual.as_ref() != expected
            || expected.is_some_and(|entry| {
                entry.kind != Kind::Regular
                    && (entry.kind != Kind::Symlink || replacement.is_none())
            })
        {
            return Err(io::Error::other("target changed after planning"));
        }
        if let (Some(expected), Some(replacement)) = (expected, &replacement)
            && expected.kind == Kind::Regular
            && expected.bytes == replacement.bytes
            && let Some(mode) = replacement.mode
            && expected.mode != Some(mode)
        {
            return self.change_mode(name, expected, mode, verify).map(Some);
        }
        let temp = super::temp_name();
        if let Some(replacement) = replacement {
            use std::io::Write;
            let fd = fs::openat(
                &self.0,
                &temp,
                OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::from_raw_mode(0o600),
            )?;
            let mut file = File::from(fd);
            let mode = replacement
                .mode
                .or_else(|| {
                    expected
                        .filter(|entry| entry.kind == Kind::Regular)
                        .and_then(|entry| entry.mode)
                })
                .unwrap_or(creation_mode);
            let prepared = (|| -> io::Result<Entry> {
                file.write_all(replacement.bytes)?;
                fs::fchmod(&file, Mode::from_raw_mode(mode))?;
                file.sync_all()?;
                Ok(Entry {
                    kind: Kind::Regular,
                    identity: identity(&fs::fstat(&file)?),
                    parent: self.id()?,
                    bytes: replacement.bytes.to_vec(),
                    mode: Some(mode),
                })
            })();
            let written = match prepared {
                Ok(written) => written,
                Err(error) => {
                    let _ = self.unlink(&temp);
                    return Err(error);
                }
            };
            if let Err(error) = verify() {
                let _ = self.unlink(&temp);
                return Err(error);
            }
            if expected.is_none() {
                if let Err(error) = self.rename(&temp, name, fs::RenameFlags::NOREPLACE) {
                    let _ = self.unlink(&temp);
                    return Err(error);
                }
                if let Err(error) = verify() {
                    let current = fs::statat(&self.0, name, AtFlags::SYMLINK_NOFOLLOW);
                    if matches!(current, Ok(stat) if identity(&stat) == written.identity) {
                        self.rename(name, &temp, fs::RenameFlags::NOREPLACE)?;
                        self.unlink(&temp)?;
                    }
                    return Err(error);
                }
            } else {
                if let Err(error) = self.rename(&temp, name, fs::RenameFlags::EXCHANGE) {
                    let _ = self.unlink(&temp);
                    return Err(error);
                }
                if !matches!(self.entry(&temp), Ok(ref old) if old.as_ref() == expected)
                    || verify().is_err()
                {
                    let current = fs::statat(&self.0, name, AtFlags::SYMLINK_NOFOLLOW);
                    if !matches!(current, Ok(stat) if identity(&stat) == written.identity) {
                        return Err(io::Error::other(format!(
                            "target changed again during replacement; old entry remains at {temp:?}"
                        )));
                    }
                    self.rename(&temp, name, fs::RenameFlags::EXCHANGE).map_err(|error| {
                        io::Error::other(format!("replacement changed and could not be reversed; recover {:?}: {error}", temp))
                    })?;
                    let _ = self.unlink(&temp);
                    return Err(io::Error::other("target changed during replacement"));
                }
                let _ = self.unlink(&temp);
            }
            Ok(Some(written))
        } else {
            if expected.is_none() {
                return Ok(None);
            }
            verify()?;
            self.rename(name, &temp, fs::RenameFlags::NOREPLACE)?;
            if !matches!(self.entry(&temp), Ok(ref old) if old.as_ref() == expected)
                || verify().is_err()
            {
                self.rename(&temp, name, fs::RenameFlags::NOREPLACE)
                    .map_err(|error| {
                        io::Error::other(format!(
                            "target changed and could not be put back; recover {:?}: {error}",
                            temp
                        ))
                    })?;
                return Err(io::Error::other("target changed during deletion"));
            }
            if let Err(error) = self.unlink(&temp) {
                self.rename(&temp, name, fs::RenameFlags::NOREPLACE)?;
                return Err(error);
            }
            Ok(None)
        }
    }

    fn unlink(&self, name: &OsStr) -> io::Result<()> {
        Ok(fs::unlinkat(&self.0, name, AtFlags::empty())?)
    }

    fn change_mode(
        &self,
        name: &OsStr,
        expected: &Entry,
        mode: u32,
        verify: impl Fn() -> io::Result<()>,
    ) -> io::Result<Entry> {
        let fd = fs::openat(
            &self.0,
            name,
            OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW | OFlags::NONBLOCK,
            Mode::empty(),
        )?;
        let stat = fs::fstat(&fd)?;
        if kind(&stat) != Kind::Regular
            || identity(&stat) != expected.identity
            || self.entry(name)?.as_ref() != Some(expected)
        {
            return Err(io::Error::other("target changed before changing its mode"));
        }
        verify()?;
        fs::fchmod(&fd, Mode::from_raw_mode(mode))?;
        let check = fs::statat(&self.0, name, AtFlags::SYMLINK_NOFOLLOW);
        if !matches!(check, Ok(stat) if identity(&stat) == expected.identity) || verify().is_err() {
            if let Some(previous) = expected.mode {
                fs::fchmod(&fd, Mode::from_raw_mode(previous))?;
            }
            return Err(io::Error::other("target moved during mode change"));
        }
        let mut updated = expected.clone();
        updated.mode = Some(mode);
        Ok(updated)
    }

    fn rename(&self, from: &OsStr, to: &OsStr, flags: fs::RenameFlags) -> io::Result<()> {
        Ok(fs::renameat_with(&self.0, from, &self.0, to, flags)?)
    }

    pub(super) fn link_to(
        &self,
        name: &OsStr,
        expected: &Entry,
        target: &Self,
        destination: &OsStr,
    ) -> io::Result<()> {
        if !self.same_link(name, expected)? {
            return Err(io::Error::other("symbolic link changed"));
        }
        fs::linkat(&self.0, name, &target.0, destination, AtFlags::empty())?;
        if !target.same_link(destination, expected)? {
            target.unlink(destination)?;
            return Err(io::Error::other(
                "symbolic link changed while backing it up",
            ));
        }
        Ok(())
    }

    pub(super) fn restore_link(
        &self,
        name: &OsStr,
        expected: &Entry,
        backup: &Self,
        saved: &OsStr,
        original: &Entry,
    ) -> io::Result<()> {
        if self.entry(name)?.as_ref() != Some(expected) {
            return Err(io::Error::other("target changed before link restoration"));
        }
        let temp = super::temp_name();
        backup.link_to(saved, original, self, &temp)?;
        if let Err(error) = self.rename(&temp, name, fs::RenameFlags::EXCHANGE) {
            let _ = self.unlink(&temp);
            return Err(error);
        }
        if !matches!(self.entry(&temp), Ok(Some(ref current)) if current == expected) {
            if !self.same_link(name, original)? {
                return Err(io::Error::other(format!(
                    "target changed during recovery; displaced file remains at {temp:?}"
                )));
            }
            self.rename(&temp, name, fs::RenameFlags::EXCHANGE)?;
            self.unlink(&temp)?;
            return Err(io::Error::other("target changed during link restoration"));
        }
        self.unlink(&temp)
    }

    fn same_link(&self, name: &OsStr, expected: &Entry) -> io::Result<bool> {
        Ok(self.entry(name)?.is_some_and(|entry| {
            entry.kind == Kind::Symlink && entry.identity == expected.identity
        }))
    }
}
