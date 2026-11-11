//! Embedded descriptions of the public TOML formats.

use serde::Serialize;
use serde_json::{Value, json};

use crate::name::ModuleName;

use super::{Field, builtin};

#[derive(Clone, Copy)]
pub enum Kind {
    Manifest,
    Module,
    Profile,
    Local,
}

#[derive(Serialize)]
pub struct Key {
    pub key: &'static str,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub required: bool,
    pub default: Option<&'static str>,
    pub description: &'static str,
}

const fn key(
    key: &'static str,
    kind: &'static str,
    required: bool,
    default: Option<&'static str>,
    description: &'static str,
) -> Key {
    Key {
        key,
        kind,
        required,
        default,
        description,
    }
}

#[derive(Serialize)]
pub struct Schema {
    pub kind: &'static str,
    pub keys: Vec<Key>,
    pub example: &'static str,
}

impl Kind {
    pub fn describe(self) -> Schema {
        match self {
            Self::Manifest => Schema {
                kind: "manifest",
                keys: vec![
                    key(
                        "outputs",
                        "table",
                        false,
                        None,
                        "User outputs indexed by name",
                    ),
                    key(
                        "builtin",
                        "table",
                        false,
                        None,
                        "Overrides indexed by built-in output name",
                    ),
                    key(
                        "template",
                        "string",
                        false,
                        None,
                        "Template path or builtin:name; mutually exclusive with copy",
                    ),
                    key(
                        "copy",
                        "string",
                        false,
                        None,
                        "Source path; mutually exclusive with template",
                    ),
                    key(
                        "target",
                        "string or table",
                        true,
                        None,
                        "Home-relative or absolute target; platform keys: linux, macos, windows. Required for user outputs",
                    ),
                    key(
                        "ownership",
                        "string",
                        false,
                        Some("full"),
                        "full or create-only",
                    ),
                    key("permissions", "string", false, None, "Unix mode in octal"),
                    key(
                        "takeover",
                        "boolean",
                        false,
                        Some("false"),
                        "Allow takeover of an unmanaged target",
                    ),
                    key(
                        "removed",
                        "boolean",
                        false,
                        Some("false"),
                        "Suppress a built-in output",
                    ),
                ],
                example: "[outputs.gitconfig]\ncopy = \"files/.gitconfig\"\ntarget = \"~/.gitconfig\"\n",
            },
            Self::Module => Schema {
                kind: "module",
                keys: vec![
                    key(
                        "modules",
                        "table",
                        false,
                        None,
                        "User module declarations indexed by name",
                    ),
                    key(
                        "values",
                        "table",
                        false,
                        None,
                        "Module assignments indexed by module and field",
                    ),
                    key("version", "integer", false, None, "Module format version"),
                    key(
                        "fields",
                        "table",
                        true,
                        None,
                        "Field declarations indexed by name",
                    ),
                    key(
                        "type",
                        "string",
                        true,
                        None,
                        "string, integer, decimal, boolean, color, enum, list, or map",
                    ),
                    key(
                        "required",
                        "boolean",
                        false,
                        Some("false"),
                        "Require a resolved field value",
                    ),
                    key("description", "string", false, None, "Field documentation"),
                    key(
                        "min",
                        "number",
                        false,
                        None,
                        "Inclusive numeric lower bound",
                    ),
                    key(
                        "max",
                        "number",
                        false,
                        None,
                        "Inclusive numeric upper bound",
                    ),
                    key(
                        "options",
                        "array",
                        false,
                        None,
                        "Allowed enum values; required for enum",
                    ),
                    key(
                        "items",
                        "table",
                        false,
                        None,
                        "Element type; required for list",
                    ),
                    key("min_items", "integer", false, None, "Minimum list length"),
                    key("max_items", "integer", false, None, "Maximum list length"),
                    key(
                        "keys",
                        "string",
                        false,
                        Some("word"),
                        "Map key format: word or identifier",
                    ),
                    key(
                        "values",
                        "table",
                        false,
                        None,
                        "Map value type; required for map",
                    ),
                ],
                example: "[modules.prompt.fields.style]\ntype = \"string\"\n\n[values.prompt]\nstyle = \"plain\"\n",
            },
            Self::Profile => Schema {
                kind: "profile",
                keys: vec![
                    key(
                        "rules",
                        "array of tables",
                        false,
                        None,
                        "Host selection rules",
                    ),
                    key("os", "string", false, None, "linux, macos, or windows"),
                    key(
                        "hostname",
                        "string",
                        false,
                        None,
                        "ASCII case-insensitive hostname",
                    ),
                    key(
                        "tags",
                        "array",
                        false,
                        None,
                        "All listed tags must be present",
                    ),
                    key(
                        "enable",
                        "array",
                        false,
                        None,
                        "Outputs enabled by a matching rule",
                    ),
                    key(
                        "values",
                        "table",
                        false,
                        None,
                        "Matching rule's value overrides",
                    ),
                ],
                example: "[[rules]]\nos = \"linux\"\nenable = [\"ghostty\"]\n\n[rules.values.font]\nsize = 14\n",
            },
            Self::Local => Schema {
                kind: "local",
                keys: vec![
                    key(
                        "source",
                        "string",
                        false,
                        Some("~/.local/share/rosamta"),
                        "Source directory within the home directory",
                    ),
                    key("tags", "array", false, None, "This host's tags"),
                    key(
                        "values",
                        "table",
                        false,
                        None,
                        "This host's value overrides",
                    ),
                ],
                example: "source = \"~/dotfiles\"\ntags = [\"work\"]\n\n[values.font]\nsize = 14\n",
            },
        }
    }
}

pub fn builtin_fields(module: Option<&ModuleName>) -> Result<Value, String> {
    let modules = builtin::modules()
        .map_err(|error| error.to_string())?
        .modules;
    if let Some(module) = module
        && !modules.contains_key(module)
    {
        return Err(format!("unknown built-in module {module}"));
    }
    let modules = modules.into_iter().filter(|(name, _)| module.is_none_or(|module| name == module))
        .map(|(name, module)| {
            let fields: Vec<_> = module.fields.into_iter().map(|(name, field)| json!({
                "name": name.to_string(), "type": field.kind.name(), "required": field.required,
                "range": field_range(&field), "default": field.default, "description": field.description,
            })).collect();
            json!({"name": name.to_string(), "version": module.version, "fields": fields})
        }).collect::<Vec<_>>();
    Ok(json!({"modules": modules}))
}

fn field_range(field: &Field) -> Value {
    use super::FieldType::*;
    match field.kind {
        Color => json!("#RRGGBB"),
        Enum => json!(field.options),
        Integer | Decimal if field.min.is_some() || field.max.is_some() => {
            json!({"min": field.min, "max": field.max})
        }
        List => {
            json!({"min_items": field.min_items, "max_items": field.max_items, "items": field.items.as_ref().map(|item| item.kind.name())})
        }
        Map => {
            json!({"keys": field.keys.map(|keys| match keys { super::MapKeys::Identifier => "identifier", super::MapKeys::Word => "word" }), "values": field.values.as_ref().map(|value| value.kind.name())})
        }
        _ => Value::Null,
    }
}
