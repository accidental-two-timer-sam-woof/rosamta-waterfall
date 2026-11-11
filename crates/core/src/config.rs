//! Configuration types, loading, schemas and built-in resources (05 §4.2–4.3).

pub mod builtin;
pub mod load;
pub mod schema;

use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{
    host::{HostContext, Platform},
    name::{FieldName, ModuleName, OutputName, Tag},
};

pub type Assignments = BTreeMap<ModuleName, BTreeMap<FieldName, Value>>;

#[derive(Debug)]
pub struct ConfigError {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)?;
        if let Some(line) = self.line {
            write!(f, ":{line}")?;
        }
        if let Some(column) = self.column {
            write!(f, ":{column}")?;
        }
        write!(f, ": {}", self.message)
    }
}

impl std::error::Error for ConfigError {}

pub fn parse<T: DeserializeOwned>(path: &str, bytes: &[u8]) -> Result<T, ConfigError> {
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|error| {
        let line = bytes[..error.valid_up_to()]
            .iter()
            .filter(|&&byte| byte == b'\n')
            .count()
            + 1;
        ConfigError {
            path: path.into(),
            line: Some(line),
            column: None,
            message: error.to_string(),
        }
    })?;
    let text = text.replace("\r\n", "\n");
    toml_edit::de::from_str(&text).map_err(|error| {
        let (line, column) =
            error
                .span()
                .and_then(|span| text.get(..span.start))
                .map_or((None, None), |prefix| {
                    (
                        Some(prefix.bytes().filter(|&byte| byte == b'\n').count() + 1),
                        Some(prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1),
                    )
                });
        ConfigError {
            path: path.into(),
            line,
            column,
            message: error.message().to_owned(),
        }
    })
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Manifest {
    pub outputs: BTreeMap<OutputName, OutputPatch>,
    pub builtin: BTreeMap<OutputName, OutputPatch>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OutputPatch {
    pub template: Option<String>,
    pub copy: Option<String>,
    pub target: Option<Target>,
    pub ownership: Option<Ownership>,
    pub permissions: Option<String>,
    pub takeover: Option<bool>,
    pub removed: Option<bool>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Ownership {
    #[default]
    Full,
    CreateOnly,
}

impl Ownership {
    pub fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::CreateOnly => "create-only",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Target {
    All(String),
    Platforms(BTreeMap<Platform, String>),
}

impl Target {
    pub fn for_platform(&self, platform: Platform) -> Option<&str> {
        match self {
            Self::All(value) => Some(value),
            Self::Platforms(values) => values.get(&platform).map(String::as_str),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Source {
    Copy(String),
    Template(String),
}

#[derive(Clone, Debug)]
pub struct Output {
    pub name: OutputName,
    pub source: Source,
    pub target: Target,
    pub ownership: Ownership,
    pub permissions: Option<u32>,
    pub takeover: bool,
    pub kind: &'static str,
    pub removed: bool,
}

pub fn permissions(text: &str) -> Result<u32, String> {
    let digits = if text.len() == 4 {
        text.strip_prefix('0').unwrap_or(text)
    } else {
        text
    };
    if digits.len() != 3 || !digits.bytes().all(|byte| matches!(byte, b'0'..=b'7')) {
        return Err(format!(
            "invalid permissions {text:?}; expected 000–777 with an optional leading zero"
        ));
    }
    u32::from_str_radix(digits, 8).map_err(|err| err.to_string())
}

impl Manifest {
    pub fn effective(&self) -> Result<BTreeMap<OutputName, Output>, Vec<String>> {
        let defaults = builtin::manifest().map_err(|error| vec![error.to_string()])?;
        let mut errors = Vec::new();
        let mut outputs = BTreeMap::new();
        for name in self.builtin.keys() {
            if !defaults.outputs.contains_key(name) {
                errors.push(format!("unknown built-in output {name}"));
            }
        }
        for (name, output) in &defaults.outputs {
            let patch = self.builtin.get(name);
            let kind = match patch {
                Some(patch) if patch.removed == Some(true) => "builtin-removed",
                Some(_) => "builtin-overridden",
                None => "builtin",
            };
            let output = patch.map_or_else(|| output.clone(), |patch| merge(output, patch));
            match build_output(name, output, kind) {
                Ok(output) => {
                    outputs.insert(name.clone(), output);
                }
                Err(error) => errors.push(error),
            }
        }
        for (name, output) in &self.outputs {
            if defaults.outputs.contains_key(name) {
                errors.push(format!("{name} is a built-in output; use [builtin.{name}]"));
            } else if output.removed.is_some() {
                errors.push(format!(
                    "{name}: removed is only available for built-in outputs"
                ));
            } else {
                match build_output(name, output.clone(), "user") {
                    Ok(output) => {
                        outputs.insert(name.clone(), output);
                    }
                    Err(error) => errors.push(error),
                }
            }
        }
        if errors.is_empty() {
            Ok(outputs)
        } else {
            Err(errors)
        }
    }
}

fn merge(base: &OutputPatch, patch: &OutputPatch) -> OutputPatch {
    let (template, copy) = if patch.template.is_some() || patch.copy.is_some() {
        (patch.template.clone(), patch.copy.clone())
    } else {
        (base.template.clone(), base.copy.clone())
    };
    let target = match (&base.target, &patch.target) {
        (Some(Target::Platforms(base)), Some(Target::Platforms(patch))) => {
            let mut targets = base.clone();
            targets.extend(patch.clone());
            Some(Target::Platforms(targets))
        }
        (_, Some(target)) => Some(target.clone()),
        (target, None) => target.clone(),
    };
    OutputPatch {
        template,
        copy,
        target,
        ownership: patch.ownership.or(base.ownership),
        permissions: patch
            .permissions
            .clone()
            .or_else(|| base.permissions.clone()),
        takeover: patch.takeover.or(base.takeover),
        removed: patch.removed,
    }
}

fn build_output(
    name: &OutputName,
    patch: OutputPatch,
    kind: &'static str,
) -> Result<Output, String> {
    let source = match (patch.template, patch.copy) {
        (Some(path), None) => Source::Template(path),
        (None, Some(path)) => Source::Copy(path),
        _ => return Err(format!("{name}: expected exactly one of template or copy")),
    };
    Ok(Output {
        name: name.clone(),
        source,
        target: patch
            .target
            .ok_or_else(|| format!("{name}: missing target"))?,
        ownership: patch.ownership.unwrap_or_default(),
        permissions: patch
            .permissions
            .as_deref()
            .map(permissions)
            .transpose()
            .map_err(|error| format!("{name}: {error}"))?,
        takeover: patch.takeover.unwrap_or(false),
        kind,
        removed: patch.removed.unwrap_or(false),
    })
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Profile {
    pub rules: Vec<ProfileRule>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProfileRule {
    pub os: Option<Platform>,
    pub hostname: Option<String>,
    pub tags: Vec<Tag>,
    pub enable: Vec<OutputName>,
    pub values: Assignments,
}

impl ProfileRule {
    pub fn matches(&self, host: &HostContext, local: &LocalSettings) -> bool {
        self.os.is_none_or(|os| host.platform == os)
            && self
                .hostname
                .as_ref()
                .is_none_or(|name| host.hostname.eq_ignore_ascii_case(name))
            && self.tags.iter().all(|tag| local.tags.contains(tag))
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LocalSettings {
    pub source: Option<String>,
    pub tags: Vec<Tag>,
    pub values: Assignments,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ModuleFile {
    pub modules: BTreeMap<ModuleName, Module>,
    pub values: Assignments,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub version: Option<u32>,
    pub fields: BTreeMap<FieldName, Field>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(rename = "type")]
    pub kind: FieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub options: Option<Vec<String>>,
    pub items: Option<Box<Field>>,
    pub min_items: Option<usize>,
    pub max_items: Option<usize>,
    pub keys: Option<MapKeys>,
    pub values: Option<Box<Field>>,
    pub default: Option<Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Integer,
    Decimal,
    Boolean,
    Color,
    Enum,
    List,
    Map,
}

impl FieldType {
    pub fn name(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::Boolean => "boolean",
            Self::Color => "color",
            Self::Enum => "enum",
            Self::List => "list",
            Self::Map => "map",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MapKeys {
    Identifier,
    Word,
}
