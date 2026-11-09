//! Facts about the process that ran a pilot, recorded next to each result.

use std::fmt::Display;
use std::io;
use std::path::{Path, PathBuf};

/// Prints one result line. CI collects these lines into the job summary.
pub fn report(id: &str, fields: &[(&str, &dyn Display)]) {
    let mut line = format!("PILOT-RESULT {id}");
    for (key, value) in fields {
        line.push_str(&format!(" {key}={value}"));
    }
    println!("{line}");
}

/// A fresh directory under the system temp directory, removed on drop.
pub struct ScratchDir(PathBuf);

impl ScratchDir {
    pub fn new(label: &str) -> io::Result<Self> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let path = std::env::temp_dir().join(format!(
            "rosamta-pilot-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir(&path)?;
        Ok(ScratchDir(path))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Who the process runs as: the uid on Unix, token elevation on Windows.
pub fn identity() -> io::Result<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let dir = ScratchDir::new("identity")?;
        let probe = dir.path().join("probe");
        std::fs::write(&probe, b"")?;
        let uid = std::fs::metadata(&probe)?.uid();
        Ok(format!("uid={uid}"))
    }
    #[cfg(windows)]
    {
        let user = std::env::var("USERNAME").unwrap_or_default();
        Ok(format!("user={user} elevated={}", is_elevated()?))
    }
}

#[cfg(windows)]
pub fn is_elevated() -> io::Result<bool> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token: HANDLE = std::ptr::null_mut();
    // SAFETY: the pseudo handle from GetCurrentProcess needs no cleanup; the
    // token handle is closed below.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut len = 0u32;
    // SAFETY: the buffer is a TOKEN_ELEVATION of the size passed in.
    let ok = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast(),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        )
    };
    let err = io::Error::last_os_error();
    // SAFETY: token came from OpenProcessToken.
    unsafe { CloseHandle(token) };
    if ok == 0 {
        return Err(err);
    }
    Ok(elevation.TokenIsElevated != 0)
}
