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

impl Dir {
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
