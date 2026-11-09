//! P2: single-file atomic replacement (REL-001, REL-002, 04 §5 A6).
//!
//! The complete new content goes to a temporary file in the target's
//! directory, is flushed to disk, and then replaces the target in one
//! operation. A process killed at any point leaves either the old or the new
//! complete file at the target path; at worst a temporary file is left over.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Marker in every temporary file name, so leftovers can be found and cleaned.
pub const TEMP_MARKER: &str = ".rosamta-tmp.";

/// How the target is replaced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// Unix `rename(2)`; Windows `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`
    /// and `MOVEFILE_WRITE_THROUGH`.
    Rename,
    /// Windows `ReplaceFileW`, which keeps the replaced file's ACL and
    /// attributes; falls back to `Rename` when the target does not exist.
    /// Rejected: a kill can leave the target neither old nor new.
    #[cfg(windows)]
    ReplaceFile,
    /// Negative control: truncate the target and write it in place. Not
    /// atomic; the kill test must be able to catch it.
    InPlace,
}

impl Method {
    pub fn name(self) -> &'static str {
        match self {
            Method::Rename => "rename",
            #[cfg(windows)]
            Method::ReplaceFile => "replace-file",
            Method::InPlace => "in-place",
        }
    }

    pub fn parse(name: &str) -> Option<Method> {
        match name {
            "rename" => Some(Method::Rename),
            #[cfg(windows)]
            "replace-file" => Some(Method::ReplaceFile),
            "in-place" => Some(Method::InPlace),
            _ => None,
        }
    }
}

/// Replaces `target` with `content`.
///
/// Without a declared mode, an existing regular file keeps its Unix
/// permission bits. The temporary file is created as `0600` before any
/// content is written to it.
pub fn replace(target: &Path, content: &[u8], method: Method) -> io::Result<()> {
    if method == Method::InPlace {
        return write_in_place(target, content);
    }
    let dir = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let existing = match fs::symlink_metadata(target) {
        Ok(meta) if meta.file_type().is_file() => Some(meta),
        Ok(_) => {
            return Err(io::Error::other("target is not a regular file"));
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };

    let (temp_path, mut file) = create_temp(dir, target)?;
    let result = (|| {
        file.write_all(content)?;
        // On Apple platforms std issues F_FULLFSYNC here.
        file.sync_all()?;
        #[cfg(unix)]
        if let Some(meta) = &existing {
            file.set_permissions(meta.permissions())?;
        }
        #[cfg(not(unix))]
        let _ = &existing;
        drop(file);
        swap_into_place(&temp_path, target, method)?;
        sync_dir(dir)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn create_temp(dir: &Path, target: &Path) -> io::Result<(PathBuf, File)> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let stem = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    loop {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = dir.join(format!(".{stem}{TEMP_MARKER}{}.{n}", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
}

#[cfg(unix)]
fn swap_into_place(temp: &Path, target: &Path, _method: Method) -> io::Result<()> {
    fs::rename(temp, target)
}

#[cfg(unix)]
fn sync_dir(dir: &Path) -> io::Result<()> {
    File::open(dir)?.sync_all()
}

#[cfg(windows)]
fn swap_into_place(temp: &Path, target: &Path, method: Method) -> io::Result<()> {
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW, ReplaceFileW,
    };
    let temp_w = wide(temp);
    let target_w = wide(target);
    if method == Method::ReplaceFile && target.exists() {
        // SAFETY: both paths are NUL-terminated UTF-16 buffers that outlive the call.
        let ok = unsafe {
            ReplaceFileW(
                target_w.as_ptr(),
                temp_w.as_ptr(),
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        return if ok != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        };
    }
    // SAFETY: as above.
    let ok = unsafe {
        MoveFileExW(
            temp_w.as_ptr(),
            target_w.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if ok != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(windows)]
fn sync_dir(_dir: &Path) -> io::Result<()> {
    // MOVEFILE_WRITE_THROUGH already waits for the rename to reach the disk;
    // Windows has no directory fsync.
    Ok(())
}

#[cfg(windows)]
fn wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn write_in_place(target: &Path, content: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(target)?;
    for chunk in content.chunks(4096) {
        file.write_all(chunk)?;
    }
    Ok(())
}

/// Temporary files left in `dir` by interrupted replacements.
pub fn leftover_temps(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().contains(TEMP_MARKER) {
            found.push(entry.path());
        }
    }
    Ok(found)
}
