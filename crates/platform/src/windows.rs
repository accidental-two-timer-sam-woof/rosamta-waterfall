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
use windows_sys::Wdk::Storage::FileSystem::{
    FILE_INFORMATION_CLASS, FILE_LINK_INFORMATION, FILE_RENAME_INFORMATION, FileLinkInformation,
    FileRenameInformation, NtSetInformationFile,
};
use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, DELETE, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_ATTRIBUTE_TAG_INFO, FILE_DISPOSITION_INFO, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_READ_DATA,
    FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES,
    FileAttributeTagInfo, FileDispositionInfo, GetFileInformationByHandle,
    GetFileInformationByHandleEx, MOVEFILE_WRITE_THROUGH, MoveFileExW, READ_CONTROL, SYNCHRONIZE,
    SetFileInformationByHandle, WRITE_DAC,
};
use windows_sys::Win32::{
    Foundation::{GENERIC_WRITE, RtlNtStatusToDosError},
    System::IO::IO_STATUS_BLOCK,
};

pub(super) struct Dir {
    file: File,
    path: PathBuf,
}

fn create_private_directory(path: &Path) -> io::Result<()> {
    use windows_sys::Win32::{
        Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
            },
            PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
        },
        Storage::FileSystem::CreateDirectoryW,
    };
    let sid = current_user_sid()?;
    let sddl = format!("D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;{sid})");
    let sddl: Vec<_> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    // The SDDL is NUL-terminated. On success the API allocates the descriptor.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let _descriptor = LocalMemory(descriptor);
    let attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    let path = wide(path);
    // The descriptor, attributes and NUL-terminated path outlive the call.
    if unsafe { CreateDirectoryW(path.as_ptr(), &attributes) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

struct LocalMemory(*mut std::ffi::c_void);

impl Drop for LocalMemory {
    fn drop(&mut self) {
        // The wrapped pointer was returned by a Windows API using LocalAlloc.
        unsafe {
            windows_sys::Win32::Foundation::LocalFree(self.0);
        }
    }
}

fn current_user_sid() -> io::Result<String> {
    use std::os::windows::io::{FromRawHandle, OwnedHandle};
    use windows_sys::Win32::{
        Foundation::HANDLE,
        Security::{
            Authorization::ConvertSidToStringSidW, GetTokenInformation, TOKEN_QUERY, TOKEN_USER,
            TokenUser,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken},
    };
    let mut token: HANDLE = std::ptr::null_mut();
    // The pseudo-handle refers to this process. A successful call owns a token handle.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let mut size = 0;
    // A null buffer with zero length reports the allocation size.
    unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            std::ptr::null_mut(),
            0,
            &mut size,
        );
    }
    if size < size_of::<TOKEN_USER>() as u32 {
        return Err(io::Error::last_os_error());
    }
    let mut buffer = vec![0u64; (size as usize).div_ceil(size_of::<u64>())];
    // The aligned buffer has enough bytes for TOKEN_USER and its SID.
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            buffer.as_mut_ptr().cast(),
            size,
            &mut size,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut sid = std::ptr::null_mut();
    // The SID points into the initialized buffer. The result is a LocalAlloc string.
    if unsafe { ConvertSidToStringSidW((*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid, &mut sid) }
        == 0
    {
        return Err(io::Error::last_os_error());
    }
    let _sid = LocalMemory(sid.cast());
    let mut len = 0;
    // ConvertSidToStringSidW guarantees a NUL-terminated UTF-16 string.
    while unsafe { *sid.add(len) } != 0 {
        len += 1;
    }
    let sid = unsafe { std::slice::from_raw_parts(sid, len) };
    String::from_utf16(sid).map_err(io::Error::other)
}

impl Dir {
    pub(super) fn enter_child_process(&self, command: &mut std::process::Command) {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
        command
            .current_dir(&self.path)
            .creation_flags(CREATE_NO_WINDOW);
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
            if is_symlink(&file)? {
                Kind::Symlink
            } else {
                Kind::Other
            }
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
        create_private_directory(&self.path.join(name))
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
        let Some(replacement) = replacement else {
            if let Some(expected) = expected {
                let file = self.hold_checked(name, expected, DELETE)?;
                delete(&file)?;
            }
            return Ok(None);
        };
        use std::io::Write;
        let temp = super::temp_name();
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .access_mode(
                GENERIC_WRITE
                    | DELETE
                    | FILE_READ_ATTRIBUTES
                    | READ_CONTROL
                    | WRITE_DAC
                    | SYNCHRONIZE,
            )
            .share_mode(0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(self.path.join(temp))?;
        let result = (|| {
            file.write_all(replacement.bytes)?;
            let written = Entry {
                kind: Kind::Regular,
                identity: identity(&file)?,
                parent: self.id()?,
                bytes: replacement.bytes.to_vec(),
                mode: None,
            };
            if let Some(expected) = expected {
                let held = self.hold_checked(name, expected, READ_CONTROL)?;
                if expected.kind == Kind::Regular {
                    copy_dacl(&held, &file)?;
                }
                file.sync_all()?;
                // The observed target excludes writers and delete access until this final step.
                // NT rename addresses the directory entry relative to the held parent.
                drop(held);
                set_name(&file, FileRenameInformation, true, &self.file, name)?;
            } else {
                file.sync_all()?;
                set_name(&file, FileRenameInformation, false, &self.file, name)?;
            }
            Ok(Some(written))
        })();
        if result.is_err() {
            let _ = delete(&file);
        }
        result
    }

    fn hold_checked(&self, name: &OsStr, expected: &Entry, access: u32) -> io::Result<File> {
        let mut file = OpenOptions::new()
            .access_mode(FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE | access)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(self.path.join(name))?;
        let attributes = file.metadata()?.file_attributes();
        let same_kind = if expected.kind == Kind::Symlink {
            is_symlink(&file)?
        } else {
            attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) == 0
        };
        if !same_kind || identity(&file)? != expected.identity || self.id()? != expected.parent {
            return Err(io::Error::other("target changed before file operation"));
        }
        if expected.kind == Kind::Regular {
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            if bytes != expected.bytes {
                return Err(io::Error::other("target changed before file operation"));
            }
        }
        Ok(file)
    }
}

fn delete(file: &File) -> io::Result<()> {
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
    Ok(())
}

fn copy_dacl(source: &File, destination: &File) -> io::Result<()> {
    use std::ptr::null_mut;
    use windows_sys::Win32::Security::{
        ACL,
        Authorization::{GetSecurityInfo, SE_FILE_OBJECT, SetSecurityInfo},
        DACL_SECURITY_INFORMATION, GetSecurityDescriptorControl,
        PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SE_DACL_PROTECTED,
        UNPROTECTED_DACL_SECURITY_INFORMATION,
    };
    let mut dacl: *mut ACL = null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = null_mut();
    // The source is held with READ_CONTROL. The returned DACL points into the allocated descriptor.
    let status = unsafe {
        GetSecurityInfo(
            source.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut dacl,
            null_mut(),
            &mut descriptor,
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let _descriptor = LocalMemory(descriptor);
    let (mut flags, mut version) = (0, 0);
    // The descriptor is live and valid until _descriptor is dropped.
    if unsafe { GetSecurityDescriptorControl(descriptor, &mut flags, &mut version) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let inheritance = if flags & SE_DACL_PROTECTED != 0 {
        PROTECTED_DACL_SECURITY_INFORMATION
    } else {
        UNPROTECTED_DACL_SECURITY_INFORMATION
    };
    // The destination is opened with WRITE_DAC and the descriptor outlives this call.
    let status = unsafe {
        SetSecurityInfo(
            destination.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | inheritance,
            null_mut(),
            null_mut(),
            dacl,
            null_mut(),
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    Ok(())
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

impl Dir {
    pub(super) fn link_to(
        &self,
        name: &OsStr,
        expected: &Entry,
        target: &Self,
        destination: &OsStr,
    ) -> io::Result<()> {
        if expected.kind != Kind::Symlink {
            return Err(io::Error::other("expected a symbolic link"));
        }
        let file = OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES | SYNCHRONIZE)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(self.path.join(name))?;
        if identity(&file)? != expected.identity || !is_symlink(&file)? {
            return Err(io::Error::other("symbolic link changed before backup"));
        }
        set_name(&file, FileLinkInformation, false, &target.file, destination)?;
        if !target
            .entry(destination)?
            .is_some_and(|entry| entry.kind == Kind::Symlink && entry.identity == expected.identity)
        {
            return Err(io::Error::other(
                "backup does not match the observed symbolic link",
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
        let temp = super::temp_name();
        backup.link_to(saved, original, self, &temp)?;
        let result = if self.entry(name)?.as_ref() == Some(expected) {
            self.rename_bound(&temp, name, true)
        } else {
            Err(io::Error::other("target changed before link restoration"))
        };
        if result.is_err() {
            let _ = fs::remove_file(self.path.join(&temp));
        }
        result
    }

    fn rename_bound(&self, source: &OsStr, target: &OsStr, replace: bool) -> io::Result<()> {
        let file = OpenOptions::new()
            .access_mode(DELETE | FILE_READ_ATTRIBUTES | SYNCHRONIZE)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(self.path.join(source))?;
        set_name(&file, FileRenameInformation, replace, &self.file, target)
    }
}

fn is_symlink(file: &File) -> io::Result<bool> {
    const SYMLINK_TAG: u32 = 0xa000_000c;
    let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
    // The buffer matches FileAttributeTagInfo and is valid for this call.
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(info.ReparseTag == SYMLINK_TAG)
}

fn set_name(
    file: &File,
    class: FILE_INFORMATION_CLASS,
    replace: bool,
    directory: &File,
    name: &OsStr,
) -> io::Result<()> {
    use std::{mem::offset_of, os::windows::ffi::OsStrExt};
    const _: () = assert!(
        offset_of!(FILE_LINK_INFORMATION, FileName)
            == offset_of!(FILE_RENAME_INFORMATION, FileName)
    );
    let name: Vec<u16> = name.encode_wide().collect();
    let offset = offset_of!(FILE_RENAME_INFORMATION, FileName);
    let size = (offset + name.len() * size_of::<u16>()).max(size_of::<FILE_RENAME_INFORMATION>());
    let mut buffer = vec![0u64; size.div_ceil(size_of::<u64>())];
    let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFORMATION>();
    // The aligned buffer has space for the header and the entire UTF-16 name.
    unsafe {
        (*info).Anonymous.ReplaceIfExists = replace;
        (*info).RootDirectory = directory.as_raw_handle();
        (*info).FileNameLength = (name.len() * size_of::<u16>()) as u32;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            (&raw mut (*info).FileName).cast(),
            name.len(),
        );
    }
    // Both File values and the name buffer remain live until the NT call returns.
    let status = unsafe {
        let mut result: IO_STATUS_BLOCK = std::mem::zeroed();
        NtSetInformationFile(
            file.as_raw_handle(),
            &mut result,
            buffer.as_ptr().cast(),
            size as u32,
            class,
        )
    };
    if status < 0 {
        // The argument is an NTSTATUS returned by NtSetInformationFile.
        let code = unsafe { RtlNtStatusToDosError(status) };
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    Ok(())
}
