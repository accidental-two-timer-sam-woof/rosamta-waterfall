//! Snapshots and batch rollback (05 §4.13).

use crate::{
    fs::{Entry, FileSystem, Replacement},
    path::HomePath,
};
use std::io;

pub struct Journal<'a> {
    fs: &'a dyn FileSystem,
    dir: HomePath,
    snapshots: Vec<HomePath>,
    changes: Vec<Change>,
}

struct Change {
    index: usize,
    target: HomePath,
    before: Option<Entry>,
    after: Option<Entry>,
}

pub struct RecoveryFailure {
    pub index: usize,
    pub path: HomePath,
    pub reason: String,
}

impl<'a> Journal<'a> {
    pub fn new(fs: &'a dyn FileSystem, dir: HomePath) -> Self {
        Self {
            fs,
            dir,
            snapshots: Vec::new(),
            changes: Vec::new(),
        }
    }

    pub fn save(&mut self, index: usize, entry: &Entry) -> Result<(), String> {
        let path = self.dir.join(&format!("before-{index}"))?;
        self.fs
            .change(
                &path,
                None,
                Some(Replacement {
                    bytes: &entry.bytes,
                    mode: entry.mode,
                }),
            )
            .map_err(|error| format!("{path}: {error}"))?;
        self.snapshots.push(path);
        Ok(())
    }

    pub fn record(
        &mut self,
        index: usize,
        target: HomePath,
        before: Option<Entry>,
        after: Option<Entry>,
    ) {
        self.changes.push(Change {
            index,
            target,
            before,
            after,
        });
    }

    pub fn rollback(&self) -> (Vec<usize>, Vec<RecoveryFailure>) {
        let mut reverted = Vec::new();
        let mut failures = Vec::new();
        for change in self.changes.iter().rev() {
            let replacement = change.before.as_ref().map(|entry| Replacement {
                bytes: &entry.bytes,
                mode: entry.mode,
            });
            match self
                .fs
                .change(&change.target, change.after.as_ref(), replacement)
            {
                Ok(_) => reverted.push(change.index),
                Err(error) => failures.push(RecoveryFailure {
                    index: change.index,
                    path: change.target.clone(),
                    reason: error.to_string(),
                }),
            }
        }
        (reverted, failures)
    }

    pub fn finish(self) -> Result<(), String> {
        for path in self.snapshots {
            if let Some(entry) = self
                .fs
                .inspect(&path)
                .map_err(|error| format!("{path}: {error}"))?
            {
                self.fs
                    .change(&path, Some(&entry), None)
                    .map_err(|error| format!("{path}: {error}"))?;
            }
        }
        self.fs
            .remove_empty_directory(&self.dir)
            .map_err(|error| format!("{}: {error}", self.dir))
    }
}

pub fn unique_directory(
    fs: &dyn FileSystem,
    base: &HomePath,
    name: &str,
) -> Result<(String, HomePath), String> {
    for index in 1..=999 {
        let id = if index == 1 {
            name.to_owned()
        } else {
            format!("{name}-{index}")
        };
        let path = base.join(&id)?;
        match fs.create_directory(&path) {
            Ok(()) => return Ok((id, path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("{path}: {error}")),
        }
    }
    Err(format!("{base}: no available batch directory"))
}
