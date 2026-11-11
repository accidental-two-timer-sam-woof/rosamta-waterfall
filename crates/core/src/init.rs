//! Source and local settings initialization, including the Git client boundary (05 §4.16).

use crate::{
    apply::{self, Choices, Deployment},
    config::{LocalSettings, load::load_layout, parse},
    docedit::{Editor, parse_assignment_value, set},
    fs::{Entry, FileSystem, Kind, Replacement},
    host::HostContext,
    name::{FieldRef, Tag},
    path::HomePath,
};

pub trait GitClient {
    fn clone_into(
        &self,
        url: &str,
        staging: &HomePath,
        expected_identity: &str,
    ) -> Result<(), String>;
}

pub struct Request<'a> {
    pub repo: Option<&'a str>,
    pub source: Option<HomePath>,
    pub tags: &'a [Tag],
    pub values: &'a [String],
    pub apply: Option<Choices>,
    pub batch: &'a str,
}

pub struct ResultInfo {
    pub source: HomePath,
    pub source_created: bool,
    pub changes: Vec<(HomePath, &'static str)>,
    pub apply: Option<Deployment>,
}

pub fn initialize(
    fs: &dyn FileSystem,
    git: &dyn GitClient,
    host: &HostContext,
    request: Request<'_>,
) -> Result<ResultInfo, String> {
    if let Some(repo) = request.repo {
        validate_repo(repo)?;
    }
    let (mut layout, _) = load_layout(fs, host)?;
    let _lock = fs
        .lock(&layout.state.join("lock")?)
        .map_err(|error| error.to_string())?;
    let (fresh, _) = load_layout(fs, host)?;
    layout = fresh;
    if let Some(source) = &request.source {
        layout.source = source.clone();
    }
    layout.validate(host)?;

    let local_path = layout.config.join("local.toml")?;
    let local_before = regular(fs, &local_path)?;
    let local = local_before.as_ref().map_or(&[][..], |entry| &entry.bytes);
    let after = edit_local(&local_path, local, &request)?;
    let source = layout.source;
    let mut changes = Vec::new();
    let source_created;
    if let Some(repo) = request.repo {
        if fs
            .inspect(&source)
            .map_err(|error| format!("{source}: {error}"))?
            .is_some()
        {
            return Err(format!(
                "{source} already exists; use a new source directory"
            ));
        }
        let staging = new_stage(fs, &source, request.batch)?;
        let staged = fs
            .inspect(&staging)
            .map_err(|error| format!("{staging}: {error}"))?
            .filter(|entry| entry.kind == Kind::Directory)
            .ok_or_else(|| format!("{staging}: staging directory is missing"))?;
        match git.clone_into(repo, &staging, &staged.identity) {
            Ok(()) => {}
            Err(error) => {
                if !matches!(fs.inspect(&staging), Ok(Some(ref current)) if current == &staged) {
                    return Err(format!("{error}; staging directory changed at {staging}"));
                }
                return match fs.remove_empty_directory(&staging) {
                    Ok(()) => Err(error),
                    Err(_) => Err(format!(
                        "{error}; inspect remaining staging directory {staging}"
                    )),
                };
            }
        }
        let updated_local =
            match store_local(fs, &local_path, local_before.as_ref(), &after, &mut changes) {
                Ok(entry) => entry,
                Err(error) => return Err(format!("{error}; cloned source remains at {staging}")),
            };
        if let Err(error) = fs.publish_directory(&staging, &staged.identity, &source) {
            let rollback = if let Some(entry) = &updated_local {
                let previous = local_before.as_ref().map(|entry| Replacement {
                    bytes: &entry.bytes,
                    mode: entry.mode,
                });
                fs.change(&local_path, Some(entry), previous).map(|_| ())
            } else {
                Ok(())
            };
            return Err(match rollback {
                Ok(()) => format!("{source}: {error}; inspect staging directory {staging}"),
                Err(restore) => format!(
                    "{source}: {error}; inspect {staging}; local settings could not be restored: {restore}"
                ),
            });
        }
        source_created = true;
    } else {
        let existing = fs
            .inspect(&source)
            .map_err(|error| format!("{source}: {error}"))?;
        match existing {
            Some(entry) if entry.kind == Kind::Directory => source_created = false,
            Some(_) => return Err(format!("{source}: expected a plain directory")),
            None => {
                fs.create_directory(&source)
                    .map_err(|error| format!("{source}: {error}"))?;
                source_created = true;
            }
        }
        let manifest = source.join("manifest.toml")?;
        let mut added = None;
        if regular(fs, &manifest)?.is_none() {
            added = fs
                .change(
                    &manifest,
                    None,
                    Some(Replacement {
                        bytes: b"",
                        mode: Some(0o600),
                    }),
                )
                .map_err(|error| format!("{manifest}: {error}"))?;
            changes.push((manifest.clone(), "created"));
        }
        if let Err(error) =
            store_local(fs, &local_path, local_before.as_ref(), &after, &mut changes)
        {
            if let Some(entry) = &added
                && let Err(rollback) = fs.change(&manifest, Some(entry), None)
            {
                return Err(format!("{error}; could not remove {manifest}: {rollback}"));
            }
            if source_created {
                let _ = fs.remove_empty_directory(&source);
            }
            return Err(error);
        }
    }
    let apply = request
        .apply
        .as_ref()
        .map(|choices| apply::execute_locked(fs, host, choices, request.batch));
    Ok(ResultInfo {
        source,
        source_created,
        changes,
        apply,
    })
}

fn regular(fs: &dyn FileSystem, path: &HomePath) -> Result<Option<Entry>, String> {
    let entry = fs
        .inspect(path)
        .map_err(|error| format!("{path}: {error}"))?;
    if entry
        .as_ref()
        .is_some_and(|entry| entry.kind != Kind::Regular)
    {
        return Err(format!("{path}: expected a regular file"));
    }
    Ok(entry)
}

fn edit_local(path: &HomePath, bytes: &[u8], request: &Request<'_>) -> Result<Vec<u8>, String> {
    let mut editor = Editor::parse(bytes).map_err(|error| format!("{path}: {error}"))?;
    let root = editor.document.as_table_mut();
    if let Some(source) = &request.source {
        set(root, "source", source.to_string().into());
    }
    if !request.tags.is_empty() {
        let mut tags = root
            .get("tags")
            .and_then(toml_edit::Item::as_array)
            .cloned()
            .unwrap_or_default();
        for tag in request.tags {
            let tag = tag.to_string();
            if !tags.iter().any(|existing| existing.as_str() == Some(&tag)) {
                tags.push(tag);
            }
        }
        set(root, "tags", toml_edit::Value::Array(tags));
    }
    for assignment in request.values {
        let (field, value) = assignment
            .split_once('=')
            .ok_or("expected MODULE.FIELD=VALUE")?;
        let field = field
            .parse::<FieldRef>()
            .map_err(|error| error.to_string())?;
        let table = editor.table(&["values", &field.module.to_string()])?;
        set(
            table,
            &field.field.to_string(),
            parse_assignment_value(value),
        );
    }
    let bytes = editor.bytes();
    parse::<LocalSettings>(&path.to_string(), &bytes).map_err(|error| error.to_string())?;
    Ok(bytes)
}

fn store_local(
    fs: &dyn FileSystem,
    path: &HomePath,
    before: Option<&Entry>,
    bytes: &[u8],
    changes: &mut Vec<(HomePath, &'static str)>,
) -> Result<Option<Entry>, String> {
    if before.is_some_and(|entry| entry.bytes == bytes) {
        return Ok(None);
    }
    let written = fs
        .change(
            path,
            before,
            Some(Replacement {
                bytes,
                mode: Some(0o600),
            }),
        )
        .map_err(|error| format!("{path}: {error}"))?;
    changes.push((
        path.clone(),
        if before.is_none() {
            "created"
        } else {
            "modified"
        },
    ));
    Ok(written)
}

fn new_stage(fs: &dyn FileSystem, source: &HomePath, batch: &str) -> Result<HomePath, String> {
    let name = source
        .as_str()
        .rsplit('/')
        .next()
        .ok_or("source must have a name")?;
    let prefix = format!(".{name}.rosamta-clone.");
    let parent = source.parent();
    if let Some(parent) = &parent {
        let names = fs
            .children(parent)
            .map_err(|error| format!("{parent}: {error}"))?;
        if let Some(existing) = names.iter().find(|name| name.starts_with(&prefix)) {
            return Err(format!(
                "an earlier clone left {}; inspect it before retrying",
                parent.join(existing)?
            ));
        }
    }
    let leaf = format!("{prefix}{batch}");
    let path = match parent {
        Some(parent) => parent.join(&leaf)?,
        None => HomePath::relative(&leaf)?,
    };
    fs.create_directory(&path)
        .map_err(|error| format!("{path}: {error}"))?;
    Ok(path)
}

fn validate_repo(repo: &str) -> Result<(), String> {
    if repo.chars().any(|ch| ch.is_whitespace() || ch.is_control()) {
        return Err("Git URL cannot contain whitespace".into());
    }
    let protocol = ["https://", "ssh://"].into_iter().any(|prefix| {
        repo.strip_prefix(prefix)
            .is_some_and(|url| !url.is_empty() && !url.starts_with('/'))
    });
    let scp = repo.split_once(':').is_some_and(|(host, path)| {
        host.contains('@')
            && !host.contains(['/', '\\', ':'])
            && !host.starts_with('-')
            && !path.is_empty()
    });
    if protocol || scp {
        Ok(())
    } else {
        Err("expected an HTTPS or SSH Git URL".into())
    }
}
