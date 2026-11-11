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
    BY_HANDLE_FILE_INFORMATION, DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_DISPOSITION_INFO, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_SHARE_READ, FILE_SHARE_WRITE,
    FileDispositionInfo, GetFileInformationByHandle, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    ReplaceFileW, SetFileInformationByHandle,
};

pub(super) struct Dir {
    file: File,
    path: PathBuf,
}

impl Dir {
    pub(super) fn enter_child_process(&self, command: &mut std::process::Command) {
        command.current_dir(&self.path);
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
        move_no_replace(&self.path.join(staging), &self.path.join(destination))?;
        if self.child(destination)?.id()? != identity {
            move_no_replace(&self.path.join(destination), &self.path.join(staging))?;
            return Err(io::Error::other(
                "staging directory changed while publishing",
            ));
        }
        Ok(())
    }
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
            .access_mode(FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES)
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

impl Dir {
    pub(super) fn mkdir(&self, name: &OsStr) -> io::Result<()> {
        fs::create_dir(self.path.join(name))
    }

    pub(super) fn rmdir(&self, name: &OsStr) -> io::Result<()> {
        let directory = self.child(name)?;
        let path = directory.path.clone();
        drop(directory);
        fs::remove_dir(path)
    }

    pub(super) fn lock(&self, name: &OsStr) -> io::Result<File> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .share_mode(0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(self.path.join(name))?;
        if file.metadata()?.file_attributes()
            & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)
            != 0
        {
            return Err(io::Error::other("lock is not a regular file"));
        }
        Ok(file)
    }

    pub(super) fn change(
        &self,
        name: &OsStr,
        expected: Option<&Entry>,
        replacement: Option<rosamta_core::fs::Replacement<'_>>,
    ) -> io::Result<Option<Entry>> {
        let path = self.path.join(name);
        if self.entry(name)?.as_ref() != expected
            || expected.is_some_and(|entry| entry.kind != Kind::Regular)
        {
            return Err(io::Error::other("target changed after planning"));
        }
        if let Some(replacement) = replacement {
            use std::io::Write;
            let temp = self.path.join(super::temp_name());
            let prepared = OpenOptions::new()
                .write(true)
                .create_new(true)
                .share_mode(0)
                .open(&temp)
                .and_then(|mut file| {
                    file.write_all(replacement.bytes)?;
                    file.sync_all()
                });
            if let Err(error) = prepared {
                if error.kind() != io::ErrorKind::AlreadyExists {
                    let _ = fs::remove_file(&temp);
                }
                return Err(error);
            }
            let result = (|| {
                if self.entry(name)?.as_ref() != expected {
                    return Err(io::Error::other("target changed before replacement"));
                }
                if expected.is_none() {
                    return move_no_replace(&temp, &path);
                }
                let aside = super::temp_name();
                let aside_path = self.path.join(&aside);
                if let Err(error) = replace_file(&path, &temp, &aside_path) {
                    if error.raw_os_error() == Some(1177) {
                        move_no_replace(&aside_path, &path).map_err(|restore| {
                            io::Error::other(format!(
                                "{error}; original remains at {}: {restore}",
                                aside_path.display()
                            ))
                        })?;
                    }
                    return Err(error);
                }
                if !matches!(self.entry(&aside), Ok(ref old) if old.as_ref() == expected) {
                    replace_file(&path, &aside_path, &temp).map_err(|error| {
                        io::Error::other(format!(
                            "target changed during replacement; original remains at {}: {error}",
                            aside_path.display()
                        ))
                    })?;
                    return Err(io::Error::other("target changed during replacement"));
                }
                let _ = fs::remove_file(&aside_path);
                Ok(())
            })();
            let _ = fs::remove_file(&temp);
            result?;
            self.entry(name)
        } else if let Some(expected) = expected {
            let mut file = OpenOptions::new()
                .access_mode(FILE_READ_DATA | FILE_READ_ATTRIBUTES | DELETE)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path)?;
            let attributes = file.metadata()?.file_attributes();
            let mut bytes = Vec::new();
            if attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
                || identity(&file)? != expected.identity
            {
                return Err(io::Error::other("target changed before deletion"));
            }
            file.read_to_end(&mut bytes)?;
            if bytes != expected.bytes {
                return Err(io::Error::other("target changed before deletion"));
            }
            let info = FILE_DISPOSITION_INFO { DeleteFile: true };
            // The handle grants DELETE, and the buffer is valid for this call.
            if unsafe {
                SetFileInformationByHandle(
                    file.as_raw_handle(),
                    FileDispositionInfo,
                    (&info as *const FILE_DISPOSITION_INFO).cast(),
                    size_of::<FILE_DISPOSITION_INFO>() as u32,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(None)
        } else {
            Ok(None)
        }
    }
}

fn wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn move_no_replace(source: &Path, dest: &Path) -> io::Result<()> {
    let (source, dest) = (wide(source), wide(dest));
    // Both paths are NUL-terminated UTF-16 and outlive the call.
    if unsafe { MoveFileExW(source.as_ptr(), dest.as_ptr(), MOVEFILE_WRITE_THROUGH) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn replace_file(target: &Path, replacement: &Path, backup: &Path) -> io::Result<()> {
    let (target, replacement, backup) = (wide(target), wide(replacement), wide(backup));
    // Both reserved arguments are NULL. The paths outlive the call.
    if unsafe {
        ReplaceFileW(
            target.as_ptr(),
            replacement.as_ptr(),
            backup.as_ptr(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
