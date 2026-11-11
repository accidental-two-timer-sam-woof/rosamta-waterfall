//! Handle-relative filesystem operations for Windows (05 §7.3).

use std::{
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, Read},
    os::windows::{
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    },
    path::{Path, PathBuf},
};

use rosamta_core::fs::{Entry, Kind};
use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
    FILE_SHARE_READ, FILE_SHARE_WRITE, GetFileInformationByHandle,
};

pub(super) struct Dir {
    file: File,
    path: PathBuf,
}

impl Dir {
    pub(super) fn home(path: &Path) -> io::Result<Self> {
        Self::open_dir(path, false)
    }

    pub(super) fn child(&self, name: &OsStr) -> io::Result<Self> {
        Self::open_dir(&self.path.join(name), true)
    }

    fn open_dir(path: &Path, reject_reparse: bool) -> io::Result<Self> {
        let flags = FILE_FLAG_BACKUP_SEMANTICS
            | if reject_reparse {
                FILE_FLAG_OPEN_REPARSE_POINT
            } else {
                0
            };
        let file = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(flags)
            .open(path)?;
        let attributes = file.metadata()?.file_attributes();
        if attributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || (reject_reparse && attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0)
        {
            return Err(io::Error::other(format!(
                "{} is not a plain directory",
                path.display()
            )));
        }
        // Use the resolved path of the trusted home for all descendant opens.
        let path = if reject_reparse {
            path.to_owned()
        } else {
            fs::canonicalize(path)?
        };
        Ok(Self { file, path })
    }

    pub(super) fn id(&self) -> io::Result<String> {
        identity(&self.file)
    }

    pub(super) fn entry(&self, name: &OsStr) -> io::Result<Option<Entry>> {
        let path = self.path.join(name);
        let flags = FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT;
        let file = match OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(flags)
            .open(&path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let attributes = file.metadata()?.file_attributes();
        let kind = if attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            Kind::Symlink
        } else if attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            Kind::Directory
        } else {
            Kind::Regular
        };
        let id = identity(&file)?;
        let mut bytes = Vec::new();
        if kind == Kind::Regular {
            let mut opened = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(flags)
                .open(&path)?;
            if identity(&opened)? != id {
                return Err(io::Error::other("file changed while opening it"));
            }
            opened.read_to_end(&mut bytes)?;
        }
        Ok(Some(Entry {
            kind,
            identity: id,
            parent: self.id()?,
            bytes,
            mode: None,
        }))
    }

    pub(super) fn names(&self) -> io::Result<Vec<String>> {
        fs::read_dir(&self.path)?
            .map(|entry| {
                entry?
                    .file_name()
                    .into_string()
                    .map_err(|_| io::Error::other("directory entry name is not UTF-8"))
            })
            .collect()
    }
}

fn identity(file: &File) -> io::Result<String> {
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // The handle is valid for the lifetime of the borrowed File.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(format!(
        "{:08x}:{:08x}{:08x}",
        info.dwVolumeSerialNumber, info.nFileIndexHigh, info.nFileIndexLow
    ))
}
