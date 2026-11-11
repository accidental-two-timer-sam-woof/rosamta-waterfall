//! Batch validation and preflight analysis (05 §4.6).

use crate::{
    config::{Output, Source, load::Config},
    fs::FileSystem,
    host::{HostContext, Platform},
    path::HomePath,
    plan::Desired,
    resolve::Resolved,
    template::Template,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn desired(
    fs: &dyn FileSystem,
    host: &HostContext,
    config: &Config,
    resolved: &Resolved<'_>,
) -> Result<Vec<(Output, Desired)>, Vec<String>> {
    let mut errors = Vec::new();
    let mut targets = BTreeMap::new();
    let mut outputs = Vec::new();
    let (enabled, _) = config.enabled(host);
    let context = resolved.template_context();
    for name in enabled.keys() {
        let output = &config.outputs[*name];
        match target(output, host, config) {
            Ok(target) => {
                if let Some(previous) =
                    targets.insert(target.key(host.platform), (target.clone(), name))
                {
                    errors.push(format!("{name} and {} target {target}", previous.1));
                }
                match content(fs, config, resolved, output, &context) {
                    Ok(bytes) => outputs.push((
                        output.clone(),
                        Desired {
                            target,
                            bytes,
                            mode: (host.platform != Platform::Windows)
                                .then_some(output.permissions)
                                .flatten(),
                        },
                    )),
                    Err(error) => errors.push(format!("{name}: {error}")),
                }
            }
            Err(error) => errors.push(format!("{name}: {error}")),
        }
    }
    for (key, (target, name)) in &targets {
        let prefix = format!("{key}/");
        if let Some((key, (_, other))) = targets.range(prefix.clone()..).next()
            && key.starts_with(&prefix)
        {
            errors.push(format!(
                "{name} targets {target}, a parent directory of {other}"
            ));
        }
    }
    if errors.is_empty() {
        Ok(outputs)
    } else {
        Err(errors)
    }
}

fn target(output: &Output, host: &HostContext, config: &Config) -> Result<HomePath, String> {
    let raw = output
        .target
        .for_platform(host.platform)
        .ok_or_else(|| format!("no target for {}", host.platform.name()))?;
    let path = HomePath::expand(raw, host)?;
    if config.layout.reserved(&path, host) {
        return Err(format!("{path} overlaps a reserved directory"));
    }
    Ok(path)
}

fn content(
    fs: &dyn FileSystem,
    config: &Config,
    resolved: &Resolved<'_>,
    output: &Output,
    context: &serde_json::Value,
) -> Result<Vec<u8>, String> {
    match &output.source {
        Source::Copy(source) => {
            let source = config.layout.source.join(source)?;
            fs.read(&source)
                .map_err(|error| format!("{source}: {error}"))?
                .ok_or_else(|| format!("{source}: source file does not exist"))
        }
        Source::Template(source) => {
            let template = Template::load(fs, &config.layout.source, source)?;
            let mut errors = Vec::new();
            let mut used = BTreeSet::new();
            for module in &template.references.modules {
                if !resolved.versions.contains_key(module) {
                    errors.push(format!("unknown module {module}"));
                }
                used.extend(
                    resolved
                        .fields
                        .keys()
                        .filter(|field| &field.module == module),
                );
            }
            for field in &template.references.fields {
                if !resolved.fields.contains_key(field) {
                    errors.push(format!("unknown field {field}"));
                }
                used.insert(field);
            }
            for field in used {
                if resolved
                    .fields
                    .get(field)
                    .is_some_and(|value| value.required && value.value.is_null())
                {
                    errors.push(format!("required field {field} has no value"));
                }
            }
            for (module, requirement) in &template.meta.requires {
                let version = resolved.versions.get(module).copied().flatten();
                if !requirement.supports(version) {
                    errors.push(format!(
                        "module {module} has version {version:?}, template requires {requirement:?}"
                    ));
                }
            }
            if !errors.is_empty() {
                return Err(format!(
                    "{}: {}; check module declarations and template references",
                    template.origin(),
                    errors.join("; ")
                ));
            }
            template.render(context)
        }
    }
}
