//! Host context collection (05 §3.3).

use rosamta_core::host::{HostContext, Platform};
use std::{env, io};

pub fn collect() -> io::Result<HostContext> {
    #[cfg(target_os = "linux")]
    let platform = Platform::Linux;
    #[cfg(target_os = "macos")]
    let platform = Platform::Macos;
    #[cfg(windows)]
    let platform = Platform::Windows;
    let path = env::home_dir()
        .filter(|path| path.is_absolute())
        .ok_or_else(|| io::Error::other("home must be an absolute directory"))?;
    if !path.is_dir() {
        return Err(io::Error::other(format!(
            "{} is not a home directory",
            path.display()
        )));
    }
    let hostname = hostname::get()?
        .into_string()
        .map_err(|_| io::Error::other("hostname is not UTF-8"))?;
    let actual = std::fs::canonicalize(&path)?;
    #[cfg(windows)]
    let actual = actual
        .to_str()
        .and_then(|path| path.strip_prefix(r"\\?\"))
        .map_or(actual.clone(), std::path::PathBuf::from);
    let mut host = HostContext::new(platform, hostname, path, env::vars_os());
    host.actual_home = Some(actual);
    Ok(host)
}
