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
    #[cfg(unix)]
    home: std::path::PathBuf,
    #[cfg(unix)]
    creation_mode: u32,
}

impl SystemFs {
    pub fn open(path: &Path) -> io::Result<Self> {
        Ok(Self {
            root: Dir::home(path)?,
            #[cfg(unix)]
            home: path.to_owned(),
            #[cfg(unix)]
            creation_mode: unix::creation_mode(),
        })
    }

    fn chain(&self, parts: &str) -> io::Result<Vec<Dir>> {
        self.walk(parts, false)
    }

    fn verify_chain(&self, parts: &str, parent: &Dir) -> io::Result<()> {
        #[cfg(unix)]
        if Dir::home(&self.home)?.id()? != self.root.id()? {
            return Err(io::Error::other("home moved after opening it"));
        }
        let fresh = self.chain(parts)?;
        if fresh.last().unwrap_or(&self.root).id()? != parent.id()? {
            return Err(io::Error::other("directory changed after opening it"));
        }
        Ok(())
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

    fn backup_link(
        &self,
        source: &HomePath,
        expected: &Entry,
        backup: &HomePath,
    ) -> io::Result<()> {
        let (source_parent, source_name) = split_path(source);
        let (backup_parent, backup_name) = split_path(backup);
        let original = self.chain(source_parent)?;
        let backup_chain = self.walk(backup_parent, true)?;
        original.last().unwrap_or(&self.root).link_to(
            source_name,
            expected,
            backup_chain.last().unwrap_or(&self.root),
            backup_name,
        )
    }

    fn restore_link(
        &self,
        backup: &HomePath,
        original: &Entry,
        target: &HomePath,
        expected: &Entry,
    ) -> io::Result<()> {
        let (backup_parent, backup_name) = split_path(backup);
        let (target_parent, target_name) = split_path(target);
        let backup_chain = self.chain(backup_parent)?;
        let target_chain = self.chain(target_parent)?;
        let from = backup_chain.last().unwrap_or(&self.root);
        let to = target_chain.last().unwrap_or(&self.root);
        to.restore_link(target_name, expected, from, backup_name, original)
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
        let verify = || self.verify_chain(parts, parent);
        #[cfg(unix)]
        {
            parent.change(
                OsStr::new(name),
                expected,
                replacement,
                self.creation_mode,
                verify,
            )
        }
        #[cfg(windows)]
        {
            verify()?;
            parent.change(OsStr::new(name), expected, replacement)
        }
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
            Ok(chain) => {
                let parent = chain.last().unwrap_or(&self.root);
                let entry = parent.entry(OsStr::new(name))?;
                #[cfg(unix)]
                self.verify_chain(parts, parent)?;
                Ok(entry)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn children(&self, path: &HomePath) -> io::Result<Vec<String>> {
        match self.chain(path.as_str()) {
            Ok(chain) => {
                let directory = chain.last().unwrap_or(&self.root);
                let names = directory.names()?;
                #[cfg(unix)]
                self.verify_chain(path.as_str(), directory)?;
                Ok(names)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error),
        }
    }
}

struct SystemLock {
    _handle: std::fs::File,
    _chain: Vec<Dir>,
}

fn split_path(path: &HomePath) -> (&str, &OsStr) {
    let (parent, name) = path
        .as_str()
        .rsplit_once('/')
        .unwrap_or(("", path.as_str()));
    (parent, OsStr::new(name))
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
