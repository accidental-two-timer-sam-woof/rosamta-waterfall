//! Boundary for filesystem operations, locking and synchronization hooks (05 §4.10).

use crate::path::HomePath;
use std::io;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Kind {
    Regular,
    Directory,
    Symlink,
    Other,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::Directory => "directory",
            Self::Symlink => "symlink",
            Self::Other => "other",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub kind: Kind,
    pub identity: String,
    pub parent: String,
    pub bytes: Vec<u8>,
    pub mode: Option<u32>,
}

pub trait FileSystem {
    fn lock(&self, path: &HomePath) -> io::Result<Box<dyn WriteLock>>;

    /// Change a regular file only if its observed state still matches.
    /// A missing replacement deletes the observed file.
    fn change(
        &self,
        path: &HomePath,
        expected: Option<&Entry>,
        replacement: Option<Replacement<'_>>,
    ) -> io::Result<Option<Entry>>;

    fn create_directory(&self, path: &HomePath) -> io::Result<()>;
    fn remove_empty_directory(&self, path: &HomePath) -> io::Result<()>;
    fn inspect(&self, path: &HomePath) -> io::Result<Option<Entry>>;
    fn children(&self, path: &HomePath) -> io::Result<Vec<String>>;

    fn read(&self, path: &HomePath) -> io::Result<Option<Vec<u8>>> {
        match self.inspect(path)? {
            None => Ok(None),
            Some(entry) if entry.kind == Kind::Regular => Ok(Some(entry.bytes)),
            Some(entry) => Err(io::Error::other(format!(
                "{path} is {}, expected a regular file",
                entry.kind.name()
            ))),
        }
    }
}

pub struct Replacement<'a> {
    pub bytes: &'a [u8],
    pub mode: Option<u32>,
}

pub trait WriteLock {}
