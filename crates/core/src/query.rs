//! Output, value and reference queries (05 §4.9).

use std::str::FromStr;

use crate::{
    config::{Output, Source, load::Config},
    fs::FileSystem,
    host::HostContext,
    name::OutputName,
    path::HomePath,
    resolve::{Resolved, Source as ValueSource},
    template::{References, Template, source_name},
};
use serde_json::{Value, json};
use std::{ffi::OsStr, path::Path};

pub struct QueryService<'a> {
    pub config: &'a Config,
    pub resolved: &'a Resolved<'a>,
    pub fs: &'a dyn FileSystem,
    pub host: &'a HostContext,
}

pub struct QueryResult {
    pub value: Value,
    pub lines: Vec<String>,
    pub errors: Vec<String>,
}

impl QueryService<'_> {
    pub fn list(&self) -> QueryResult {
        let (enabled, _) = self.config.enabled(self.host);
        let outputs: Vec<_> = self
            .config
            .outputs
            .values()
            .map(|output| self.listing(output, enabled.get(&output.name)))
            .collect();
        let lines = outputs
            .iter()
            .map(|output| {
                format!(
                    "{}  {}  {}  {}",
                    output["name"].as_str().unwrap_or_default(),
                    output["kind"].as_str().unwrap_or_default(),
                    if output["enabled"] == true {
                        "enabled"
                    } else {
                        "disabled"
                    },
                    output["target"]["path"]
                        .as_str()
                        .or_else(|| output["target"]["unavailable"].as_str())
                        .unwrap_or_default(),
                )
            })
            .collect();
        QueryResult {
            value: json!({"outputs": outputs}),
            lines,
            errors: Vec::new(),
        }
    }

    pub fn refs(&self, search: &RefQuery) -> Result<QueryResult, String> {
        match search {
            RefQuery::Module(module) if !self.resolved.versions.contains_key(module) => {
                return Err(format!("unknown module {module}"));
            }
            RefQuery::Field(field) if !self.resolved.fields.contains_key(field) => {
                return Err(format!("unknown field {field}"));
            }
            _ => {}
        }
        let (enabled, _) = self.config.enabled(self.host);
        let mut result = Vec::new();
        let mut undetermined = Vec::new();
        let mut errors = Vec::new();
        let mut lines = Vec::new();
        for output in self
            .config
            .outputs
            .values()
            .filter(|output| !output.removed)
        {
            let listing = self.listing(output, enabled.get(&output.name));
            let summary = json!({"name": listing["name"], "target": listing["target"], "enabled": listing["enabled"]});
            match self.references(output) {
                Ok(references) if mentions(&references, search) => {
                    lines.push(format!("{}  {}", output.name, display_target(&summary)));
                    result.push(summary);
                }
                Ok(_) => {}
                Err(error) => {
                    undetermined.push(summary);
                    errors.push(format!("{}: {error}", output.name));
                }
            }
        }
        Ok(QueryResult {
            value: json!({"outputs": result, "undetermined": undetermined}),
            lines,
            errors,
        })
    }

    pub fn output(&self, key: &OsStr, cwd: &Path) -> Result<QueryResult, String> {
        let (enabled, _) = self.config.enabled(self.host);
        let candidates = match key
            .to_str()
            .and_then(|value| value.parse::<OutputName>().ok())
        {
            Some(name) => self
                .config
                .outputs
                .get(&name)
                .into_iter()
                .collect::<Vec<_>>(),
            None => {
                let path = HomePath::argument(key, self.host, cwd)?.key(self.host.platform);
                self.config
                    .outputs
                    .values()
                    .filter(|output| {
                        self.target(output)
                            .is_ok_and(|target| target.key(self.host.platform) == path)
                    })
                    .collect()
            }
        };
        if candidates.is_empty() {
            return Err(format!("no output matches {}", key.to_string_lossy()));
        }
        let mut outputs = Vec::new();
        let mut lines = Vec::new();
        let mut errors = Vec::new();
        for output in candidates {
            let mut listing = self.listing(output, enabled.get(&output.name));
            let references = match self.references(output) {
                Ok(references) => references,
                Err(error) => {
                    errors.push(format!("{}: {error}", output.name));
                    continue;
                }
            };
            let fields: Vec<_> = references.fields.iter().map(ToString::to_string).collect();
            let modules: Vec<_> = references.modules.iter().map(ToString::to_string).collect();
            lines.push(format!(
                "{}  {}  {}\n  fields: {}\n  modules: {}",
                output.name,
                display_target(&listing),
                source_name(&output.source),
                fields.join(", "),
                modules.join(", ")
            ));
            listing["references"] = json!({"fields": fields, "modules": modules});
            outputs.push(listing);
        }
        Ok(QueryResult {
            value: json!({"outputs": outputs}),
            lines,
            errors,
        })
    }

    pub fn value(&self, field: &FieldRef) -> Result<QueryResult, String> {
        let found = self
            .resolved
            .fields
            .get(field)
            .ok_or_else(|| format!("unknown field {field}"))?;
        let description = found.source.as_ref().map_or_else(
            || "unset".into(),
            |source| format!("{} ({})", source.layer, source.location),
        );
        Ok(QueryResult {
            value: json!({"field": field.to_string(), "declared_at": found.declared_at, "value": found.value,
                "source": found.source.as_ref().map(source_json), "overridden": found.overridden.iter().map(source_json).collect::<Vec<_>>() }),
            lines: vec![format!("{field} = {}  {description}", found.value)],
            errors: Vec::new(),
        })
    }

    fn listing(&self, output: &Output, rules: Option<&Vec<usize>>) -> Value {
        let source = match &output.source {
            Source::Copy(path) => json!({"copy": path}),
            Source::Template(path) => match path.strip_prefix("builtin:") {
                Some(name) => json!({"builtin_template": name}),
                None => json!({"template": path}),
            },
        };
        let target = match self.target(output) {
            Ok(path) => json!({"path": path.to_absolute(self.host)}),
            Err(error) => json!({"unavailable": error}),
        };
        json!({"name": output.name.to_string(), "kind": output.kind, "enabled": rules.is_some(), "enabled_by": rules.cloned().unwrap_or_default(),
            "target": target, "source": source, "ownership": output.ownership.name(), "permissions": output.permissions.map(|mode| format!("{mode:04o}")), "takeover": output.takeover})
    }

    fn target(&self, output: &Output) -> Result<HomePath, String> {
        let target = output
            .target
            .for_platform(self.host.platform)
            .ok_or_else(|| format!("no target for {}", self.host.platform.name()))?;
        HomePath::expand(target, self.host)
    }

    fn references(&self, output: &Output) -> Result<References, String> {
        match &output.source {
            Source::Copy(_) => Ok(References::default()),
            Source::Template(path) => {
                let template = Template::load(self.fs, &self.config.layout.source, path)?;
                for module in &template.references.modules {
                    if !self.resolved.versions.contains_key(module) {
                        return Err(format!("{}: unknown module {module}", template.origin()));
                    }
                }
                for field in &template.references.fields {
                    if !self.resolved.fields.contains_key(field) {
                        return Err(format!("{}: unknown field {field}", template.origin()));
                    }
                }
                Ok(template.references)
            }
        }
    }
}

fn display_target(output: &Value) -> &str {
    output["target"]["path"]
        .as_str()
        .or_else(|| output["target"]["unavailable"].as_str())
        .unwrap_or_default()
}

fn source_json(source: &ValueSource) -> Value {
    json!({"layer": source.layer, "location": source.location, "rule": source.rule})
}

fn mentions(references: &References, search: &RefQuery) -> bool {
    match search {
        RefQuery::Module(module) => {
            references.modules.contains(module)
                || references
                    .fields
                    .iter()
                    .any(|field| &field.module == module)
        }
        RefQuery::Field(field) => {
            references.fields.contains(field) || references.modules.contains(&field.module)
        }
    }
}

use crate::name::{FieldRef, ModuleName, NameError};

#[derive(Debug, Clone)]
pub enum RefQuery {
    Module(ModuleName),
    Field(FieldRef),
}

impl FromStr for RefQuery {
    type Err = NameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.contains('.') {
            value.parse().map(Self::Field)
        } else {
            value.parse().map(Self::Module)
        }
    }
}
