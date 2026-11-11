//! Fixtures and injection points for integration tests (05 §8).

use std::{io, process::Command};

pub use tempfile::TempDir as TempHome;

/// Creates a temporary home for a child process and removes it when dropped.
pub fn temporary_home() -> io::Result<TempHome> {
    tempfile::Builder::new().prefix("rosamta-home-").tempdir()
}

/// Runs a program in the test home with no inherited XDG directory overrides.
pub fn command_in_home(program: impl AsRef<std::ffi::OsStr>, home: &TempHome) -> Command {
    let mut command = Command::new(program);
    command
        .current_dir(home.path())
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .env_remove("XDG_STATE_HOME");
    command
}
