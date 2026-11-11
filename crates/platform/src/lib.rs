//! Operating-system adapters for the core interfaces.

pub mod git;
pub mod host;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix;

#[cfg(windows)]
mod windows;

use rosamta_core::{
    fs::{Entry, FileSystem, Replacement, WriteLock},
    path::HomePath,
};
use std::{ffi::OsStr, io, path::Path};

#[cfg(unix)]
use unix::Dir;
#[cfg(windows)]
use windows::Dir;

pub struct SystemFs {
    root: Dir,
}

impl SystemFs {
    pub fn open(path: &Path) -> io::Result<Self> {
        Ok(Self {
            root: Dir::home(path)?,
        })
    }

    fn chain(&self, parts: &str) -> io::Result<Vec<Dir>> {
        self.walk(parts, false)
    }

    fn walk(&self, parts: &str, create: bool) -> io::Result<Vec<Dir>> {
        let mut chain = Vec::new();
        for part in parts.split('/').filter(|part| !part.is_empty()) {
            let directory = chain.last().unwrap_or(&self.root);
            let part = OsStr::new(part);
            let child = match directory.child(part) {
                Err(error) if create && error.kind() == io::ErrorKind::NotFound => {
                    match directory.mkdir(part) {
                        Ok(()) => {}
                        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                        Err(error) => return Err(error),
                    }
                    directory.child(part)?
                }
                result => result?,
            };
            chain.push(child);
        }
        Ok(chain)
    }
}

impl FileSystem for SystemFs {
    fn lock(&self, path: &HomePath) -> io::Result<Box<dyn WriteLock>> {
        let (parts, name) = path
            .as_str()
            .rsplit_once('/')
            .unwrap_or(("", path.as_str()));
        let chain = self.walk(parts, true)?;
        let handle = chain.last().unwrap_or(&self.root).lock(OsStr::new(name))?;
        Ok(Box::new(SystemLock {
            _handle: handle,
            _chain: chain,
        }))
    }

    fn change(
        &self,
        path: &HomePath,
        expected: Option<&Entry>,
        replacement: Option<Replacement<'_>>,
    ) -> io::Result<Option<Entry>> {
        let (parts, name) = path
            .as_str()
            .rsplit_once('/')
            .unwrap_or(("", path.as_str()));
        let chain = self.walk(parts, expected.is_none() && replacement.is_some())?;
        let parent = chain.last().unwrap_or(&self.root);
        let fresh = self.chain(parts)?;
        if fresh.last().unwrap_or(&self.root).id()? != parent.id()? {
            return Err(io::Error::other("target parent changed after opening it"));
        }
        parent.change(OsStr::new(name), expected, replacement)
    }

    fn create_directory(&self, path: &HomePath) -> io::Result<()> {
        let (parts, name) = path
            .as_str()
            .rsplit_once('/')
            .unwrap_or(("", path.as_str()));
        let chain = self.walk(parts, true)?;
        chain.last().unwrap_or(&self.root).mkdir(OsStr::new(name))
    }

    fn publish_directory(
        &self,
        staging: &HomePath,
        identity: &str,
        destination: &HomePath,
    ) -> io::Result<()> {
        fn split(path: &str) -> (&str, &str) {
            path.rsplit_once('/').unwrap_or(("", path))
        }
        let (staging_parent, staging_name) = split(staging.as_str());
        let (destination_parent, destination_name) = split(destination.as_str());
        if staging_parent != destination_parent {
            return Err(io::Error::other(
                "staging and destination must share a directory",
            ));
        }
        let chain = self.chain(staging_parent)?;
        let parent = chain.last().unwrap_or(&self.root);
        parent.publish(
            OsStr::new(staging_name),
            OsStr::new(destination_name),
            identity,
        )
    }

    fn remove_empty_directory(&self, path: &HomePath) -> io::Result<()> {
        let (parts, name) = path
            .as_str()
            .rsplit_once('/')
            .unwrap_or(("", path.as_str()));
        let chain = self.chain(parts)?;
        chain.last().unwrap_or(&self.root).rmdir(OsStr::new(name))
    }
    fn inspect(&self, path: &HomePath) -> io::Result<Option<Entry>> {
        let (parts, name) = path
            .as_str()
            .rsplit_once('/')
            .unwrap_or(("", path.as_str()));
        match self.chain(parts) {
            Ok(chain) => chain.last().unwrap_or(&self.root).entry(OsStr::new(name)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn children(&self, path: &HomePath) -> io::Result<Vec<String>> {
        match self.chain(path.as_str()) {
            Ok(chain) => chain.last().unwrap_or(&self.root).names(),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error),
        }
    }
}

struct SystemLock {
    _handle: std::fs::File,
    _chain: Vec<Dir>,
}

impl WriteLock for SystemLock {}

fn temp_name() -> std::ffi::OsString {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!(
        ".rosamta-tmp.{}.{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
    .into()
}
