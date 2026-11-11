//! Adding, modifying and forgetting manifest outputs (05 §4.8).

use crate::{
    config::{Manifest, Output, Ownership, Source, load::load_layout, parse, permissions},
    docedit::{Editor, set},
    fs::{Entry, FileSystem, Kind, Replacement},
    host::{HostContext, Platform},
    layout::Layout,
    name::OutputName,
    path::HomePath,
    state::State,
};

use std::{collections::BTreeMap, io};
use toml_edit::{InlineTable, Item, Table, TableLike, Value};

pub struct Operation {
    pub output: OutputName,
    pub changes: Vec<(HomePath, &'static str)>,
}

pub struct ManifestService<'a> {
    pub fs: &'a dyn FileSystem,
    pub host: &'a HostContext,
}

struct Snapshot {
    path: HomePath,
    entry: Entry,
    outputs: BTreeMap<OutputName, Output>,
}

#[derive(Default)]
pub struct Patch {
    pub source: Option<Option<Source>>,
    pub targets: Vec<(Option<Platform>, Option<String>)>,
    pub ownership: Option<Option<Ownership>>,
    pub permissions: Option<Option<String>>,
    pub takeover: Option<Option<bool>>,
}

impl ManifestService<'_> {
    pub fn add(&self, target: HomePath, name: Option<OutputName>) -> Result<Operation, String> {
        let (layout, _) = load_layout(self.fs, self.host)?;
        let _lock = self
            .fs
            .lock(&layout.state.join("lock")?)
            .map_err(|error| error.to_string())?;
        let snapshot = self.snapshot(&layout)?;
        if layout.reserved(&target, self.host) {
            return Err(format!("{target} overlaps a reserved directory"));
        }
        let existing = self
            .fs
            .inspect(&target)
            .map_err(|error| format!("{target}: {error}"))?
            .filter(|entry| entry.kind == Kind::Regular)
            .ok_or_else(|| format!("{target}: expected an existing regular file"))?;
        for output in snapshot.outputs.values().filter(|output| !output.removed) {
            if output
                .target
                .for_platform(self.host.platform)
                .and_then(|target| HomePath::expand(target, self.host).ok())
                .is_some_and(|other| {
                    target.key(self.host.platform) == other.key(self.host.platform)
                })
            {
                return Err(format!("{target} is already targeted by {}", output.name));
            }
        }
        let name = match name {
            Some(name) if snapshot.outputs.contains_key(&name) => {
                return Err(format!("output name {name} already exists"));
            }
            Some(name) => name,
            None => unique_name(&target, &snapshot.outputs)?,
        };
        let mut editor = snapshot.editor()?;
        let table = editor.table(&["outputs", &name.to_string()])?;
        let mut targets = Table::new();
        targets.set_dotted(true);
        set(
            &mut targets,
            self.host.platform.name(),
            Value::from(target.to_string()),
        );
        table.insert("target", Item::Table(targets));

        for index in 1..=99 {
            let suffix = copy_suffix(target.as_str(), index);
            let relative = format!("files/{suffix}");
            let path = layout.source.join(&relative)?;
            if self
                .fs
                .inspect(&path)
                .map_err(|error| format!("{path}: {error}"))?
                .is_some()
            {
                continue;
            }
            let after = match self.fs.change(
                &path,
                None,
                Some(Replacement {
                    bytes: &existing.bytes,
                    mode: Some(0o600),
                }),
            ) {
                Ok(Some(entry)) => entry,
                Ok(None) => return Err(format!("{path}: source copy disappeared")),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("{path}: {error}")),
            };
            set(table, "copy", Value::from(relative));
            let mut changes = vec![(path.clone(), "created")];
            if let Err(error) = self.save_manifest(&snapshot, editor, &mut changes) {
                return match self.fs.change(&path, Some(&after), None) {
                    Ok(_) => Err(error),
                    Err(rollback) => Err(format!("{error}; could not remove {path}: {rollback}")),
                };
            }
            return Ok(Operation {
                output: name,
                changes,
            });
        }
        Err(format!("{target}: no available path under files/"))
    }

    pub fn modify(&self, name: OutputName, patch: Patch) -> Result<Operation, String> {
        let (layout, _) = load_layout(self.fs, self.host)?;
        let _lock = self
            .fs
            .lock(&layout.state.join("lock")?)
            .map_err(|error| error.to_string())?;
        let snapshot = self.snapshot(&layout)?;
        let output = snapshot
            .outputs
            .get(&name)
            .filter(|output| !output.removed)
            .ok_or_else(|| format!("unknown or removed output {name}"))?;
        let name_text = name.to_string();
        let parent = if output.kind == "user" {
            "outputs"
        } else {
            "builtin"
        };
        let mut editor = snapshot.editor()?;
        let table = editor.table(&[parent, &name_text])?;
        if let Some(source) = patch.source {
            match source {
                Some(Source::Copy(path)) => {
                    HomePath::relative(&path)?;
                    set(table, "copy", Value::from(path));
                    table.remove("template");
                }
                Some(Source::Template(path)) => {
                    if let Some(name) = path.strip_prefix("builtin:") {
                        name.parse::<OutputName>()
                            .map_err(|error| error.to_string())?;
                    } else {
                        HomePath::relative(&path)?;
                    }
                    set(table, "template", Value::from(path));
                    table.remove("copy");
                }
                None => {
                    table.remove("template");
                    table.remove("copy");
                }
            }
        }
        for (platform, target) in patch.targets {
            if target.as_ref().is_some_and(|path| {
                path.is_empty() || path.contains('\\') || path.chars().any(char::is_control)
            }) {
                return Err("target requires a nonempty path with / separators".into());
            }
            patch_target(table, platform, target)?;
        }
        if let Some(ownership) = patch.ownership {
            optional(
                table,
                "ownership",
                ownership.map(|value| Value::from(value.name())),
            );
        }
        if let Some(value) = patch.permissions {
            if let Some(value) = &value {
                permissions(value)?;
            }
            optional(table, "permissions", value.map(Value::from));
        }
        if let Some(takeover) = patch.takeover {
            optional(table, "takeover", takeover.map(Value::from));
        }
        if table.is_empty() && parent == "builtin" {
            editor.table(&[parent])?.remove(&name_text);
        }
        let mut changes = Vec::new();
        self.save_manifest(&snapshot, editor, &mut changes)?;
        Ok(Operation {
            output: name,
            changes,
        })
    }

    pub fn forget(&self, name: OutputName) -> Result<Operation, String> {
        let (layout, _) = load_layout(self.fs, self.host)?;
        let _lock = self
            .fs
            .lock(&layout.state.join("lock")?)
            .map_err(|error| error.to_string())?;
        let snapshot = self.snapshot(&layout)?;
        let output = snapshot.outputs.get(&name).filter(|output| !output.removed);
        let target = output
            .and_then(|output| output.target.for_platform(self.host.platform))
            .map(|path| HomePath::expand(path, self.host))
            .transpose()?;
        if let Some(path) = &target {
            self.fs
                .inspect(path)
                .map_err(|error| format!("{path}: {error}"))?;
        }
        let mut state = State::load(self.fs, &layout, self.host.platform)?;
        let mut pending = state.records.clone();
        let name_text = name.to_string();
        let mut selected = Vec::new();
        for (key, record) in &mut pending {
            let same_target = target
                .as_ref()
                .is_some_and(|target| target.key(self.host.platform) == *key);
            let belongs_to_other = record.name.parse::<OutputName>().ok().is_some_and(|other| {
                snapshot
                    .outputs
                    .get(&other)
                    .is_some_and(|output| !output.removed)
            });
            if (record.name == name_text && (output.is_some() || record.forget_pending))
                || (same_target && !belongs_to_other)
            {
                selected.push(key.clone());
                record.forget_pending = true;
                record.name.clone_from(&name_text);
            }
        }
        if output.is_none() && selected.is_empty() {
            return Err(format!("unknown output {name}"));
        }
        let mut editor = snapshot.editor()?;
        if let Some(output) = output {
            if output.kind == "user" {
                editor.table(&["outputs"])?.remove(&name_text);
            } else {
                let mut removed = Table::new();
                set(&mut removed, "removed", Value::from(true));
                editor
                    .table(&["builtin"])?
                    .insert(&name_text, Item::Table(removed));
            }
        }
        let mut changes = Vec::new();
        if pending != state.records {
            state.commit(self.fs, pending.clone())?;
        }
        self.save_manifest(&snapshot, editor, &mut changes)?;
        if !selected.is_empty() {
            for key in selected {
                pending.remove(&key);
            }
            state.commit(self.fs, pending)?;
            changes.push((state.path, "modified"));
        }
        Ok(Operation {
            output: name,
            changes,
        })
    }

    fn snapshot(&self, layout: &Layout) -> Result<Snapshot, String> {
        let path = layout.source.join("manifest.toml")?;
        let entry = self
            .fs
            .inspect(&path)
            .map_err(|error| format!("{path}: {error}"))?
            .filter(|entry| entry.kind == Kind::Regular)
            .ok_or_else(|| format!("{path}: expected a regular file"))?;
        let manifest: Manifest =
            parse(&path.to_string(), &entry.bytes).map_err(|error| error.to_string())?;
        let outputs = manifest.effective().map_err(|errors| errors.join("\n"))?;
        Ok(Snapshot {
            path,
            entry,
            outputs,
        })
    }

    fn save_manifest(
        &self,
        snapshot: &Snapshot,
        editor: Editor,
        changes: &mut Vec<(HomePath, &'static str)>,
    ) -> Result<(), String> {
        let bytes = editor.bytes();
        if bytes != snapshot.entry.bytes {
            let manifest: Manifest =
                parse(&snapshot.path.to_string(), &bytes).map_err(|error| error.to_string())?;
            manifest.effective().map_err(|errors| errors.join("\n"))?;
            self.fs
                .change(
                    &snapshot.path,
                    Some(&snapshot.entry),
                    Some(Replacement {
                        bytes: &bytes,
                        mode: snapshot.entry.mode,
                    }),
                )
                .map_err(|error| format!("{}: {error}", snapshot.path))?;
            changes.push((snapshot.path.clone(), "modified"));
        }
        Ok(())
    }
}

impl Snapshot {
    fn editor(&self) -> Result<Editor, String> {
        Editor::parse(&self.entry.bytes).map_err(|error| format!("{}: {error}", self.path))
    }
}

fn unique_name(
    target: &HomePath,
    outputs: &BTreeMap<OutputName, Output>,
) -> Result<OutputName, String> {
    let file = target.as_str().rsplit('/').next().unwrap_or_default();
    let cleaned = file
        .trim_start_matches('.')
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let cleaned = cleaned.trim_start_matches('_');
    let base = if cleaned.is_empty() { "file" } else { cleaned };
    for index in 1..=99 {
        let name = if index == 1 {
            base.to_owned()
        } else {
            format!("{base}-{index}")
        };
        let name = name
            .parse()
            .map_err(|error: crate::name::NameError| error.to_string())?;
        if !outputs.contains_key(&name) {
            return Ok(name);
        }
    }
    Err(format!("{target}: no available output name"))
}

fn copy_suffix(path: &str, index: u32) -> String {
    if index == 1 {
        return path.to_owned();
    }
    let (parent, name) = path.rsplit_once('/').unwrap_or(("", path));
    let name = match name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() => format!("{stem}-{index}.{extension}"),
        _ => format!("{name}-{index}"),
    };
    if parent.is_empty() {
        name
    } else {
        format!("{parent}/{name}")
    }
}

fn optional(table: &mut dyn TableLike, key: &str, value: Option<Value>) {
    match value {
        Some(value) => set(table, key, value),
        None => {
            table.remove(key);
        }
    }
}

fn patch_target(
    table: &mut dyn TableLike,
    platform: Option<Platform>,
    target: Option<String>,
) -> Result<(), String> {
    let Some(platform) = platform else {
        optional(table, "target", target.map(Value::from));
        return Ok(());
    };
    if let Some(original) = table.get("target").and_then(Item::as_str) {
        let original = original.to_owned();
        let mut targets = InlineTable::new();
        for platform in [Platform::Linux, Platform::Macos, Platform::Windows] {
            targets.insert(platform.name(), Value::from(original.clone()));
        }
        set(table, "target", Value::InlineTable(targets));
    }
    if !table.contains_key("target") {
        if target.is_none() {
            return Ok(());
        }
        set(table, "target", Value::InlineTable(InlineTable::new()));
    }
    let targets = table
        .get_mut("target")
        .and_then(Item::as_table_like_mut)
        .ok_or("target is not a platform table")?;
    optional(targets, platform.name(), target.map(Value::from));
    if targets.is_empty() {
        table.remove("target");
    }
    Ok(())
}
