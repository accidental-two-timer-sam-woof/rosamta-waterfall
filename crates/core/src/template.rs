use minijinja::{
    Environment, Error, ErrorKind, UndefinedBehavior, Value as JinjaValue, value::ValueKind,
};
use serde::Deserialize;
use std::collections::BTreeSet;

use crate::{
    config::{Source, parse},
    fs::FileSystem,
    name::{FieldRef, ModuleName},
    path::HomePath,
};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TemplateMeta {
    pub program: Option<String>,
    pub format_version: Option<String>,
    pub requires: std::collections::BTreeMap<ModuleName, Requirement>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Requirement {
    One(u32),
    Many(Vec<u32>),
}

impl Requirement {
    pub fn supports(&self, version: Option<u32>) -> bool {
        version.is_some_and(|version| match self {
            Self::One(value) => version == *value,
            Self::Many(values) => values.contains(&version),
        })
    }
}

#[derive(Debug, Default)]
pub struct References {
    pub fields: BTreeSet<FieldRef>,
    pub modules: BTreeSet<ModuleName>,
}

pub struct Template {
    environment: Environment<'static>,
    origin: String,
    pub meta: TemplateMeta,
    pub references: References,
}

impl Template {
    pub fn load(fs: &dyn FileSystem, root: &HomePath, path: &str) -> Result<Self, String> {
        if let Some(name) = path.strip_prefix("builtin:") {
            let text = builtin(name).ok_or_else(|| format!("unknown built-in template {name}"))?;
            return Self::compile(path, text);
        }
        let file = root.join(path)?;
        let bytes = fs
            .read(&file)
            .map_err(|error| format!("{file}: {error}"))?
            .ok_or_else(|| format!("{file}: template does not exist"))?;
        let text = std::str::from_utf8(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes))
            .map_err(|error| format!("{file}: {error}"))?;
        Self::compile(&file.to_string(), &text.replace("\r\n", "\n"))
    }

    pub fn compile(origin: &str, text: &str) -> Result<Self, String> {
        let mut environment = environment().map_err(|error| error.to_string())?;
        let meta = if let Some(text) = text.strip_prefix("{#- rosamta\n") {
            let (header, _) = text
                .split_once("-#}")
                .ok_or_else(|| format!("{origin}:1: template metadata is not closed"))?;
            parse(origin, header.as_bytes()).map_err(|error| error.to_string())?
        } else {
            TemplateMeta::default()
        };
        environment
            .add_template_owned(origin.to_owned(), text.to_owned())
            .map_err(|error| format!("{origin}: {error}"))?;
        let template = environment
            .get_template(origin)
            .map_err(|error| format!("{origin}: {error}"))?;
        let mut references = References::default();
        for path in template.undeclared_variables(true) {
            let mut parts = path.split('.');
            let Some(module) = parts.next() else { continue };
            if matches!(module, "host" | "rgb" | "rgb_unit") {
                continue;
            }
            if let Some(field) = parts.next() {
                references.fields.insert(
                    format!("{module}.{field}")
                        .parse()
                        .map_err(|error| format!("{origin}: {error}"))?,
                );
            } else {
                references.modules.insert(
                    module
                        .parse()
                        .map_err(|error| format!("{origin}: {error}"))?,
                );
            }
        }
        Ok(Self {
            environment,
            origin: origin.into(),
            meta,
            references,
        })
    }

    pub fn origin(&self) -> &str {
        &self.origin
    }

    pub fn render(&self, context: &serde_json::Value) -> Result<Vec<u8>, String> {
        self.environment
            .get_template(&self.origin)
            .and_then(|template| template.render(minijinja::value::Serde(context)))
            .map(String::into_bytes)
            .map_err(|error| {
                format!(
                    "{}: {error}; check the template syntax and field names",
                    self.origin
                )
            })
    }
}

pub fn source_name(source: &Source) -> &str {
    match source {
        Source::Copy(path) | Source::Template(path) => path,
    }
}

fn environment() -> Result<Environment<'static>, Error> {
    let mut env = Environment::empty();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_syntax(
        minijinja::syntax::SyntaxConfig::builder()
            .keep_trailing_newline(true)
            .build()?,
    );
    env.set_loader(|_| -> Result<Option<String>, Error> {
        Err(Error::new(
            ErrorKind::InvalidOperation,
            "template includes and imports are unsupported; define a macro in the same template",
        ))
    });
    env.add_function("rgb", rgb);
    env.add_function("rgb_unit", |color: String| {
        rgb(color).map(|rgb| rgb.map(|channel| f64::from(channel) / 255.0))
    });
    env.add_test("none", |value: JinjaValue| value.is_none());
    env.add_filter("lower", |value: String| value.to_lowercase());
    env.add_filter("upper", |value: String| value.to_uppercase());
    env.add_filter("replace", |value: String, old: String, new: String| {
        value.replace(&old, &new)
    });
    env.add_filter("split", |value: String, sep: String| {
        value.split(&sep).map(str::to_owned).collect::<Vec<_>>()
    });
    env.add_filter("join", |values: Vec<String>, sep: String| values.join(&sep));
    env.add_filter("length", |value: JinjaValue| {
        value.len().ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidOperation,
                "length requires a collection or string",
            )
        })
    });
    env.add_filter("items", |value: JinjaValue| {
        if value.kind() != ValueKind::Map {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "items requires a map",
            ));
        }
        let mut keys = value
            .try_iter()?
            .map(|key| key.to_string())
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.into_iter()
            .map(|key| value.get_attr(&key).map(|value| (key, value)))
            .collect::<Result<Vec<_>, _>>()
    });
    env.add_filter("tojson", |value: JinjaValue| {
        serde_json::to_string(&value)
            .map(|json| json.replace('\x7f', "\\u007f"))
            .map_err(|error| Error::new(ErrorKind::InvalidOperation, error.to_string()))
    });
    env.add_filter("sh_quote", |value: String| {
        format!("'{}'", value.replace('\'', "'\\''"))
    });
    env.add_filter("ps_quote", |value: String| {
        format!("'{}'", value.replace('\'', "''"))
    });
    Ok(env)
}

fn rgb(color: String) -> Result<[u8; 3], Error> {
    let invalid = || Error::new(ErrorKind::InvalidOperation, "expected #RRGGBB");
    let color = color.strip_prefix('#').ok_or_else(invalid)?;
    if color.len() != 6 || !color.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid());
    }
    let number = u32::from_str_radix(color, 16).map_err(|_| invalid())?;
    let [_, r, g, b] = number.to_be_bytes();
    Ok([r, g, b])
}

fn builtin(name: &str) -> Option<&'static str> {
    match name {
        "ghostty" => Some(include_str!("config/builtin/ghostty.j2")),
        "alacritty" => Some(include_str!("config/builtin/alacritty.j2")),
        "posix-shell" => Some(include_str!("config/builtin/posix-shell.j2")),
        "powershell" => Some(include_str!("config/builtin/powershell.j2")),
        _ => None,
    }
}
