//! Snapshots and batch rollback (05 §4.13).

use crate::{
    fs::{Entry, FileSystem, Replacement},
    path::HomePath,
};
use std::io;

pub struct Journal<'a> {
    fs: &'a dyn FileSystem,
    dir: HomePath,
    snapshots: Vec<(HomePath, Entry)>,
    changes: Vec<Change>,
}

struct Change {
    index: usize,
    target: HomePath,
    before: Option<Entry>,
    after: Option<Entry>,
    link_backup: Option<HomePath>,
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
        self.save_file(path, &entry.bytes, entry.mode)
    }

    pub fn index<'p>(
        &mut self,
        targets: impl Iterator<Item = (usize, &'p HomePath)>,
    ) -> Result<(), String> {
        let targets: Vec<_> = targets
            .map(|(index, path)| serde_json::json!({"index": index, "target": path.as_str()}))
            .collect();
        let bytes =
            serde_json::to_vec_pretty(&serde_json::json!({"version": 1, "targets": targets}))
                .map_err(|error| error.to_string())?;
        self.save_file(self.dir.join("targets.json")?, &bytes, Some(0o600))
    }

    fn save_file(&mut self, path: HomePath, bytes: &[u8], mode: Option<u32>) -> Result<(), String> {
        let saved = self
            .fs
            .change(&path, None, Some(Replacement { bytes, mode }))
            .map_err(|error| format!("{path}: {error}"))?
            .ok_or_else(|| format!("{path}: snapshot was not created"))?;
        self.snapshots.push((path, saved));
        Ok(())
    }

    pub fn record(
        &mut self,
        index: usize,
        target: HomePath,
        before: Option<Entry>,
        after: Option<Entry>,
        link_backup: Option<HomePath>,
    ) {
        self.changes.push(Change {
            index,
            target,
            before,
            after,
            link_backup,
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
            let result = match (&change.link_backup, &change.before, &change.after) {
                (Some(backup), Some(original), Some(expected)) => {
                    self.fs
                        .restore_link(backup, original, &change.target, expected)
                }
                (Some(_), _, _) => Err(io::Error::other("missing observed file for link recovery")),
                (None, _, _) => self
                    .fs
                    .change(&change.target, change.after.as_ref(), replacement)
                    .map(|_| ()),
            };
            match result {
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
        for (path, snapshot) in self.snapshots.into_iter().rev() {
            self.fs
                .change(&path, Some(&snapshot), None)
                .map_err(|error| format!("{path}: {error}"))?;
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
