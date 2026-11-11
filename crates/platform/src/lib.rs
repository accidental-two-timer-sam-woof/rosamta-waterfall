//! Operating-system adapters for the core interfaces.

pub mod git;
pub mod host;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix;

#[cfg(windows)]
mod windows;
