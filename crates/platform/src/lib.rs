//! Operating-system adapters for the core interfaces.

pub mod git;
pub mod host;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix;

#[cfg(windows)]
mod windows;

use rosamta_core::{
    fs::{Entry, FileSystem},
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
        let mut chain = Vec::new();
        for part in parts.split('/').filter(|part| !part.is_empty()) {
            let directory = chain.last().unwrap_or(&self.root);
            let child = directory.child(OsStr::new(part))?;
            chain.push(child);
        }
        Ok(chain)
    }
}

impl FileSystem for SystemFs {
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
