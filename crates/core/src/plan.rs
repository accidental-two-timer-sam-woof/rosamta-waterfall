//! Apply plans and drift comparison (05 §4.12).

use crate::{
    config::Ownership,
    fs::{Entry, FileSystem, Kind},
    host::{HostContext, Platform},
    layout::Layout,
    path::HomePath,
    state::{Fingerprint, Parent, Record, State},
};
use std::collections::{BTreeMap, BTreeSet};

pub struct Desired {
    pub target: HomePath,
    pub bytes: Vec<u8>,
    pub mode: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Create,
    Update,
    Delete,
    Unchanged,
}
impl Action {
    pub fn name(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Unchanged => "unchanged",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    Proceed,
    Takeover,
    Overwrite,
    Accept,
    Delete,
    Retain,
}
impl Disposition {
    pub fn name(self) -> &'static str {
        match self {
            Self::Proceed => "proceed",
            Self::Takeover => "takeover",
            Self::Overwrite => "overwrite",
            Self::Accept => "accept",
            Self::Delete => "delete",
            Self::Retain => "retain",
        }
    }
}

#[derive(Default)]
pub struct Choices {
    pub strict: bool,
    pub overwrite_all: bool,
    pub accept_all: bool,
    pub delete_modified_all: bool,
    pub overwrite: BTreeSet<String>,
    pub accept: BTreeSet<String>,
    pub delete_modified: BTreeSet<String>,
}

impl Choices {
    fn drift(&self, key: &str) -> Disposition {
        if self.strict {
            Disposition::Retain
        } else if self.overwrite.contains(key) {
            Disposition::Overwrite
        } else if self.accept.contains(key) || self.accept_all {
            Disposition::Accept
        } else if self.overwrite_all {
            Disposition::Overwrite
        } else {
            Disposition::Retain
        }
    }
}

pub struct Plan {
    pub items: Vec<Item>,
    pub warnings: Vec<String>,
    pub strict: bool,
    pub blocked: bool,
}

pub struct Item {
    pub output: String,
    pub target: HomePath,
    pub before: Option<Entry>,
    pub desired: Option<Desired>,
    pub action: Action,
    pub disposition: Disposition,
    pub reason: Option<&'static str>,
    pub drift: bool,
    pub cleanup: bool,
    pub record_changed: bool,
}

impl Item {
    pub fn writes_target(&self) -> bool {
        !matches!(self.disposition, Disposition::Retain | Disposition::Accept)
            && self.action != Action::Unchanged
    }
}

pub struct DesiredSet(pub Vec<(crate::config::Output, Desired)>);

pub fn build(
    fs: &dyn FileSystem,
    layout: &Layout,
    host: &HostContext,
    state: &State,
    desired: DesiredSet,
    choices: &Choices,
) -> Result<Plan, Vec<String>> {
    let mut errors = Vec::new();
    let mut items = Vec::new();
    let mut enabled = BTreeSet::new();
    for (output, content) in desired.0 {
        let key = content.target.key(host.platform);
        enabled.insert(key.clone());
        let before = match fs.inspect(&content.target) {
            Ok(entry) => entry,
            Err(error) => {
                errors.push(format!("{}: {}: {error}", output.name, content.target));
                continue;
            }
        };
        let mut item = Item {
            output: output.name.to_string(),
            target: content.target.clone(),
            before,
            desired: Some(content),
            action: Action::Unchanged,
            disposition: Disposition::Proceed,
            reason: None,
            drift: false,
            cleanup: false,
            record_changed: false,
        };
        let record = state.records.get(&key);
        classify(
            &mut item,
            record,
            output.ownership,
            output.takeover,
            host.platform,
            choices,
            &key,
        );
        items.push(item);
    }
    for (key, record) in &state.records {
        if enabled.contains(key) {
            continue;
        }
        let target = HomePath::relative(&record.target).map_err(|e| vec![e])?;
        let inspection = fs.inspect(&target);
        let (before, inaccessible) = match inspection {
            Ok(entry) => (entry, false),
            Err(_) => (None, true),
        };
        let mut item = Item {
            output: record.name.clone(),
            target,
            before,
            desired: None,
            action: Action::Delete,
            disposition: Disposition::Proceed,
            reason: None,
            drift: false,
            cleanup: true,
            record_changed: true,
        };
        let reason = if record.forget_pending {
            Some("forget-pending")
        } else if layout.reserved(&item.target, host) {
            Some("reserved")
        } else if inaccessible {
            Some("location-unconfirmed")
        } else if let Some(entry) = &item.before {
            if entry.kind != Kind::Regular {
                Some("unsupported-type")
            } else if !record.parent.matches(entry, host.platform) {
                Some("location-unconfirmed")
            } else if record.observed.matches(entry) {
                None
            } else if choices.delete_modified_all || choices.delete_modified.contains(key) {
                item.disposition = Disposition::Delete;
                None
            } else {
                Some("modified")
            }
        } else {
            item.action = Action::Unchanged;
            None
        };
        if let Some(reason) = reason {
            retain(&mut item, reason);
        }
        items.push(item);
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut warnings = Vec::new();
    for (paths, option, cleanup) in [
        (&choices.overwrite, "--overwrite", false),
        (&choices.accept, "--accept", false),
        (&choices.delete_modified, "--delete-modified", true),
    ] {
        for path in paths {
            let matched = items.iter().any(|item| {
                item.target.key(host.platform) == *path
                    && if cleanup { item.cleanup } else { item.drift }
            });
            if !matched {
                warnings.push(format!("{option} did not match a conflict at ~/{path}"));
            }
        }
    }
    items.sort_by_key(|item| {
        (
            !item.cleanup,
            item.target.key(host.platform),
            item.output.clone(),
        )
    });
    let blocked = choices.strict && items.iter().any(|item| item.drift);
    Ok(Plan {
        items,
        warnings,
        strict: choices.strict,
        blocked,
    })
}

fn retain(item: &mut Item, reason: &'static str) {
    item.disposition = Disposition::Retain;
    item.reason = Some(reason);
    item.record_changed = false;
}

fn classify(
    item: &mut Item,
    record: Option<&Record>,
    ownership: Ownership,
    takeover: bool,
    platform: Platform,
    choices: &Choices,
    key: &str,
) {
    if record.is_some_and(|record| record.forget_pending) {
        retain(item, "forget-pending");
        return;
    }
    let Some(desired) = item.desired.as_ref() else {
        return;
    };
    let desired = Fingerprint::new(&desired.bytes, desired.mode);
    let Some(before) = item.before.as_ref() else {
        item.action = Action::Create;
        item.record_changed = true;
        return;
    };
    let record = record.filter(|record| record.parent.matches(before, platform));
    if ownership == Ownership::CreateOnly {
        item.record_changed = record.is_some_and(|record| record.name != item.output);
        return;
    }
    if before.kind != Kind::Regular {
        retain(item, "unsupported-type");
        return;
    }
    if desired.matches(before) {
        item.record_changed = record.is_none_or(|record| {
            record.name != item.output
                || record.desired != desired
                || !record.observed.matches(before)
        });
        return;
    }
    if let Some(record) = record {
        if !record.observed.matches(before) {
            item.drift = true;
            item.action = Action::Update;
            item.disposition = choices.drift(key);
            item.record_changed = item.disposition != Disposition::Retain;
            if item.disposition == Disposition::Retain {
                item.reason = Some("drift-conflict");
            }
        } else if desired != record.desired {
            item.action = Action::Update;
            item.record_changed = true;
        } else {
            item.record_changed = item.output != record.name;
        }
    } else {
        item.action = Action::Update;
        if takeover {
            item.disposition = Disposition::Takeover;
            item.record_changed = true;
        } else {
            retain(item, "unmanaged");
        }
    }
}

pub fn update_record(
    records: &mut BTreeMap<String, Record>,
    item: &Item,
    current: Option<&Entry>,
    platform: Platform,
) -> Result<(), String> {
    if !item.record_changed {
        return Ok(());
    }
    let key = item.target.key(platform);
    if item.cleanup {
        records.remove(&key);
        return Ok(());
    }
    let desired = item.desired.as_ref().ok_or("missing desired output")?;
    let current = current.ok_or_else(|| format!("{}: output no longer exists", item.target))?;
    if !item.writes_target()
        && item.disposition != Disposition::Accept
        && let Some(record) = records.get_mut(&key)
        && !Fingerprint::new(&desired.bytes, desired.mode).matches(current)
    {
        record.name.clone_from(&item.output);
        return Ok(());
    }
    records.insert(
        key,
        Record {
            name: item.output.clone(),
            target: item.target.as_str().to_owned(),
            parent: Parent::from_entry(current, platform),
            observed: Fingerprint::new(
                &current.bytes,
                if desired.mode.is_some() {
                    current.mode
                } else {
                    None
                },
            ),
            desired: Fingerprint::new(&desired.bytes, desired.mode),
            forget_pending: false,
        },
    );
    Ok(())
}
