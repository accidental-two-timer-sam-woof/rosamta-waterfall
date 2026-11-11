use crate::{
    fs::{FileSystem, Kind},
    host::HostContext,
    layout::Layout,
    name::OutputName,
    path::HomePath,
};
use std::collections::{BTreeMap, BTreeSet};

use super::{LocalSettings, Manifest, ModuleFile, Output, Profile, parse};

pub struct Config {
    pub layout: Layout,
    pub local: LocalSettings,
    pub manifest: Manifest,
    pub profile: Profile,
    pub modules: Vec<(HomePath, ModuleFile)>,
    pub outputs: BTreeMap<OutputName, Output>,
}

impl Config {
    pub fn load(fs: &dyn FileSystem, host: &HostContext) -> Result<Self, Vec<String>> {
        let (layout, local) = load_layout(fs, host).map_err(|error| vec![error])?;
        let mut errors = Vec::new();
        let manifest: Option<Manifest> = collect(load_manifest(fs, &layout), &mut errors);
        let profile: Option<Profile> = collect(
            read_config(
                fs,
                &layout.source.join("profile.toml").map_err(|e| vec![e])?,
                false,
            ),
            &mut errors,
        );
        let modules = modules(
            fs,
            &layout.source.join("modules").map_err(|e| vec![e])?,
            &mut errors,
        );
        let outputs = manifest
            .as_ref()
            .and_then(|manifest| match manifest.effective() {
                Ok(outputs) => Some(outputs),
                Err(output_errors) => {
                    errors.extend(output_errors);
                    None
                }
            });
        if let (Some(manifest), Some(profile), Some(outputs)) = (manifest, profile, outputs)
            && errors.is_empty()
        {
            return Ok(Self {
                layout,
                local,
                manifest,
                profile,
                modules,
                outputs,
            });
        }
        Err(errors)
    }

    pub fn enabled(
        &self,
        host: &HostContext,
    ) -> (BTreeMap<&OutputName, Vec<usize>>, BTreeSet<&OutputName>) {
        let mut enabled = BTreeMap::<_, Vec<_>>::new();
        let mut unknown = BTreeSet::new();
        for (index, rule) in self
            .profile
            .rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| rule.matches(host, &self.local))
        {
            for output in &rule.enable {
                match self.outputs.get(output) {
                    Some(definition) if !definition.removed => {
                        enabled.entry(output).or_default().push(index + 1)
                    }
                    Some(_) | None => {
                        unknown.insert(output);
                    }
                }
            }
        }
        (enabled, unknown)
    }
}

pub fn load_layout(
    fs: &dyn FileSystem,
    host: &HostContext,
) -> Result<(Layout, LocalSettings), String> {
    let mut layout = Layout::new(host)?;
    let local: LocalSettings = read_config(fs, &layout.config.join("local.toml")?, false)?;
    if let Some(source) = &local.source {
        layout.source = HomePath::expand(source, host)?;
    }
    if [&layout.config, &layout.state].into_iter().any(|path| {
        layout.source.starts_with(path, host.platform)
            || path.starts_with(&layout.source, host.platform)
    }) {
        return Err("source directory overlaps local settings or state".into());
    }
    Ok((layout, local))
}

pub fn load_manifest(fs: &dyn FileSystem, layout: &Layout) -> Result<Manifest, String> {
    read_config(fs, &layout.source.join("manifest.toml")?, true)
}

fn read_config<T: serde::de::DeserializeOwned + Default>(
    fs: &dyn FileSystem,
    path: &HomePath,
    required: bool,
) -> Result<T, String> {
    match fs.read(path).map_err(|error| format!("{path}: {error}"))? {
        Some(bytes) => parse(&path.to_string(), &bytes).map_err(|error| error.to_string()),
        None if !required => Ok(T::default()),
        None => Err(format!("{path}: file does not exist")),
    }
}

fn collect<T>(result: Result<T, String>, errors: &mut Vec<String>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            errors.push(error);
            None
        }
    }
}

fn modules(
    fs: &dyn FileSystem,
    root: &HomePath,
    errors: &mut Vec<String>,
) -> Vec<(HomePath, ModuleFile)> {
    let mut modules = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(dir) = pending.pop() {
        let Some(mut names) = collect(fs.children(&dir).map_err(|e| format!("{dir}: {e}")), errors)
        else {
            continue;
        };
        names.sort_unstable();
        for name in names {
            let Some(path) = collect(dir.join(&name), errors) else {
                continue;
            };
            let Some(Some(entry)) = collect(
                fs.inspect(&path).map_err(|e| format!("{path}: {e}")),
                errors,
            ) else {
                continue;
            };
            match entry.kind {
                Kind::Directory => pending.push(path),
                Kind::Regular if name.ends_with(".toml") => {
                    if let Some(module) = collect(
                        parse(&path.to_string(), &entry.bytes).map_err(|e| e.to_string()),
                        errors,
                    ) {
                        modules.push((path, module));
                    }
                }
                Kind::Symlink => {
                    errors.push(format!("{path}: links are not supported under modules/"))
                }
                _ => {}
            }
        }
    }
    modules.sort_by(|(left, _), (right, _)| left.cmp(right));
    modules
}
