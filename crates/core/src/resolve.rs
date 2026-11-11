//! Profile matching, assignment validation and the four value layers.

use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

use crate::{
    config::{Assignments, Field, FieldType, MapKeys, Module, builtin, load::Config},
    host::HostContext,
    name::{FieldName, FieldRef, ModuleName},
};

pub struct ResolvedValue {
    pub value: Value,
    pub required: bool,
    pub declared_at: String,
    pub source: Option<Source>,
    pub overridden: Vec<Source>,
}

#[derive(Clone)]
pub struct Source {
    pub layer: &'static str,
    pub location: String,
    pub rule: Option<usize>,
}

pub struct Resolved<'a> {
    pub fields: BTreeMap<FieldRef, ResolvedValue>,
    pub versions: BTreeMap<ModuleName, Option<u32>>,
    pub host: &'a HostContext,
    pub tags: Vec<String>,
}

impl Resolved<'_> {
    pub fn template_context(&self) -> Value {
        let mut modules = Map::new();
        for module in self.versions.keys() {
            modules.insert(module.to_string(), Value::Object(Map::new()));
        }
        for (field, value) in &self.fields {
            if let Some(Value::Object(module)) = modules.get_mut(&field.module.to_string()) {
                module.insert(field.field.to_string(), value.value.clone());
            }
        }
        modules.insert("host".into(), json!({"os": self.host.platform.name(), "hostname": self.host.hostname, "tags": self.tags}));
        Value::Object(modules)
    }
}

pub fn resolve<'a>(config: &'a Config, host: &'a HostContext) -> Result<Resolved<'a>, Vec<String>> {
    let builtins = builtin::modules().map_err(|error| vec![error.to_string()])?;
    let mut errors = Vec::new();
    let mut declarations = BTreeMap::new();
    for (name, module) in &builtins.modules {
        declarations.insert(name.clone(), (module, format!("builtin:{name}")));
    }
    for (path, file) in &config.modules {
        for (name, module) in &file.modules {
            if matches!(name.to_string().as_str(), "host" | "rgb" | "rgb_unit") {
                errors.push(format!("{path}: {name} is reserved by the template engine"));
            } else if let Some((_, previous)) = declarations.get(name) {
                errors.push(format!(
                    "{path}: duplicate module {name}, first declared in {previous}"
                ));
            } else {
                declarations.insert(name.clone(), (module, path.to_string()));
            }
        }
    }

    let mut assignments = BTreeMap::new();
    let module_files = config.modules.iter().map(|(path, file)| {
        (
            &file.values,
            Source {
                layer: "user-module",
                location: path.to_string(),
                rule: None,
            },
        )
    });
    let profiles = config
        .profile
        .rules
        .iter()
        .enumerate()
        .filter(|(_, rule)| rule.matches(host, &config.local))
        .map(|(index, rule)| {
            (
                &rule.values,
                Source {
                    layer: "profile",
                    location: format!("{}/profile.toml", config.layout.source),
                    rule: Some(index + 1),
                },
            )
        });
    let local = (
        &config.local.values,
        Source {
            layer: "local",
            location: format!("{}/local.toml", config.layout.config),
            rule: None,
        },
    );
    for (values, source) in module_files.chain(profiles).chain(std::iter::once(local)) {
        assign(values, source, &declarations, &mut assignments, &mut errors);
    }

    let mut fields = BTreeMap::new();
    let mut versions = BTreeMap::new();
    for (module, (declaration, location)) in declarations {
        versions.insert(module.clone(), declaration.version);
        for (name, field) in &declaration.fields {
            let reference = FieldRef {
                module: module.clone(),
                field: name.clone(),
            };
            let from_builtin = location.starts_with("builtin:");
            if let Err(message) = validate_declaration(field, from_builtin, false) {
                errors.push(format!("{location}: {reference}: {message}"));
            }
            let mut layers = Vec::new();
            if let Some(value) = field.default.as_ref() {
                match validate_value(field, value) {
                    Ok(value) => layers.push((
                        value,
                        Source {
                            layer: "builtin-default",
                            location: location.clone(),
                            rule: None,
                        },
                    )),
                    Err(message) => errors.push(format!("{location}: {reference}: {message}")),
                }
            }
            layers.extend(assignments.remove(&reference).unwrap_or_default());
            let (value, source) = match layers.pop() {
                Some((value, source)) => (value, Some(source)),
                None => (Value::Null, None),
            };
            fields.insert(
                reference,
                ResolvedValue {
                    value,
                    required: field.required,
                    source,
                    declared_at: location.clone(),
                    overridden: layers.into_iter().map(|(_, source)| source).collect(),
                },
            );
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let mut tags: Vec<_> = config.local.tags.iter().map(ToString::to_string).collect();
    tags.sort_unstable();
    tags.dedup();
    Ok(Resolved {
        fields,
        versions,
        host,
        tags,
    })
}

fn assign(
    values: &Assignments,
    source: Source,
    declarations: &BTreeMap<ModuleName, (&Module, String)>,
    assignments: &mut BTreeMap<FieldRef, Vec<(Value, Source)>>,
    errors: &mut Vec<String>,
) {
    for (module, values) in values {
        for (name, value) in values {
            let reference = FieldRef {
                module: module.clone(),
                field: name.clone(),
            };
            let Some(declaration) = declarations
                .get(module)
                .and_then(|(module, _)| module.fields.get(name))
            else {
                errors.push(format!("{}: unknown field {reference}", source.location));
                continue;
            };
            let layers = assignments.entry(reference.clone()).or_default();
            if let Some((_, previous)) = layers
                .iter()
                .find(|(_, previous)| previous.layer == source.layer)
            {
                errors.push(format!(
                    "{}: duplicate value for {reference} in {} ({:?} and {:?})",
                    source.location, previous.location, previous.rule, source.rule
                ));
                continue;
            }
            match validate_value(declaration, value) {
                Ok(value) => layers.push((value, source.clone())),
                Err(message) => errors.push(format!("{}: {reference}: {message}", source.location)),
            }
        }
    }
}

fn validate_declaration(field: &Field, builtin: bool, nested: bool) -> Result<(), &'static str> {
    use FieldType::*;
    let wrong_key = ((!matches!(field.kind, Integer | Decimal))
        && (field.min.is_some() || field.max.is_some()))
        || (field.kind != Enum && field.options.is_some())
        || (field.kind != List
            && (field.items.is_some() || field.min_items.is_some() || field.max_items.is_some()))
        || (field.kind != Map && (field.keys.is_some() || field.values.is_some()))
        || (nested && (field.required || !field.description.is_empty()))
        || ((!builtin || nested) && field.default.is_some());
    if wrong_key {
        return Err("field declaration contains an attribute that does not apply");
    }
    match field.kind {
        Integer | Decimal => {
            for bound in [&field.min, &field.max].into_iter().flatten() {
                if (field.kind == Integer && bound.as_i64().is_none())
                    || bound.as_f64().is_none_or(|x| !x.is_finite())
                {
                    return Err("invalid numeric bound");
                }
            }
            if let (Some(min), Some(max)) = (&field.min, &field.max)
                && numeric_less(max, min, field.kind)
            {
                return Err("maximum is less than minimum");
            }
        }
        Enum => {
            let options = field.options.as_ref().ok_or("enum requires options")?;
            let set = options.iter().collect::<std::collections::BTreeSet<_>>();
            if options.is_empty() || options.len() != set.len() {
                return Err("enum options must be nonempty and unique");
            }
        }
        List => {
            if let (Some(min), Some(max)) = (field.min_items, field.max_items)
                && max < min
            {
                return Err("max_items is less than min_items");
            }
            validate_declaration(
                field.items.as_deref().ok_or("list requires items")?,
                builtin,
                true,
            )?;
        }
        Map => validate_declaration(
            field.values.as_deref().ok_or("map requires values")?,
            builtin,
            true,
        )?,
        _ => {}
    }
    Ok(())
}

fn numeric_less(a: &Value, b: &Value, kind: FieldType) -> bool {
    if kind == FieldType::Integer {
        a.as_i64() < b.as_i64()
    } else {
        a.as_f64() < b.as_f64()
    }
}

fn validate_value(field: &Field, value: &Value) -> Result<Value, String> {
    use FieldType::*;
    let invalid = || format!("expected a valid {} value", field.kind.name());
    match field.kind {
        String if value.as_str().is_some_and(|text| !text.is_empty()) => {}
        Boolean if value.is_boolean() => {}
        Integer | Decimal => {
            if (field.kind == Integer && value.as_i64().is_none())
                || value.as_f64().is_none_or(|x| !x.is_finite())
            {
                return Err(invalid());
            }
            if field
                .min
                .as_ref()
                .is_some_and(|min| numeric_less(value, min, field.kind))
                || field
                    .max
                    .as_ref()
                    .is_some_and(|max| numeric_less(max, value, field.kind))
            {
                return Err("value is outside the declared range".into());
            }
        }
        Color => {
            let color = value
                .as_str()
                .and_then(|text| text.strip_prefix('#'))
                .ok_or_else(invalid)?;
            if color.len() != 6 || !color.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(invalid());
            }
            return Ok(json!(format!("#{}", color.to_ascii_lowercase())));
        }
        Enum if value
            .as_str()
            .is_some_and(|text| field.options.iter().flatten().any(|choice| choice == text)) => {}
        List => {
            let items = value.as_array().ok_or_else(invalid)?;
            if field.min_items.is_some_and(|min| items.len() < min)
                || field.max_items.is_some_and(|max| items.len() > max)
            {
                return Err("list length is outside the declared range".into());
            }
            let spec = field.items.as_deref().ok_or_else(invalid)?;
            return items
                .iter()
                .map(|value| validate_value(spec, value))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array);
        }
        Map => {
            let items = value.as_object().ok_or_else(invalid)?;
            let spec = field.values.as_deref().ok_or_else(invalid)?;
            return items
                .iter()
                .map(|(name, value)| {
                    let valid = match field.keys.unwrap_or(MapKeys::Word) {
                        MapKeys::Identifier => name.parse::<FieldName>().is_ok(),
                        MapKeys::Word => {
                            !name.is_empty()
                                && !name.chars().any(|ch| ch.is_whitespace() || ch.is_control())
                        }
                    };
                    if !valid {
                        return Err(format!("invalid map key {name:?}"));
                    }
                    Ok((name.clone(), validate_value(spec, value)?))
                })
                .collect::<Result<serde_json::Map<_, _>, _>>()
                .map(Value::Object);
        }
        _ => return Err(invalid()),
    }
    Ok(value.clone())
}
