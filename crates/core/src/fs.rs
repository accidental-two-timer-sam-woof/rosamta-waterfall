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
