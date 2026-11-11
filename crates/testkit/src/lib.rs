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

use rosamta_core::{
    fs::{Entry, FileSystem, Replacement, WriteLock},
    path::HomePath,
};

/// Delegates to the real platform while allowing a test to fail an individual write.
pub struct FaultFs<'a, F> {
    pub inner: &'a dyn FileSystem,
    pub before_change: F,
}

impl<F: Fn(&HomePath) -> io::Result<()>> FileSystem for FaultFs<'_, F> {
    fn lock(&self, path: &HomePath) -> io::Result<Box<dyn WriteLock>> {
        self.inner.lock(path)
    }

    fn change(
        &self,
        path: &HomePath,
        expected: Option<&Entry>,
        replacement: Option<Replacement<'_>>,
    ) -> io::Result<Option<Entry>> {
        (self.before_change)(path)?;
        self.inner.change(path, expected, replacement)
    }

    fn inspect(&self, path: &HomePath) -> io::Result<Option<Entry>> {
        self.inner.inspect(path)
    }

    fn children(&self, path: &HomePath) -> io::Result<Vec<String>> {
        self.inner.children(path)
    }

    fn create_directory(&self, path: &HomePath) -> io::Result<()> {
        self.inner.create_directory(path)
    }

    fn remove_empty_directory(&self, path: &HomePath) -> io::Result<()> {
        self.inner.remove_empty_directory(path)
    }

    fn publish_directory(
        &self,
        staging: &HomePath,
        identity: &str,
        destination: &HomePath,
    ) -> io::Result<()> {
        self.inner.publish_directory(staging, identity, destination)
    }

    fn backup_link(
        &self,
        source: &HomePath,
        expected: &Entry,
        backup: &HomePath,
    ) -> io::Result<()> {
        self.inner.backup_link(source, expected, backup)
    }

    fn restore_link(
        &self,
        backup: &HomePath,
        original: &Entry,
        target: &HomePath,
        expected: &Entry,
    ) -> io::Result<()> {
        self.inner.restore_link(backup, original, target, expected)
    }
}
