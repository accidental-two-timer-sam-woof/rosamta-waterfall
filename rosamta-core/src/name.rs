//! Validated configuration names (05 §4.1 and §5.3).

use std::{fmt, str::FromStr};

#[derive(Debug)]
pub struct NameError(&'static str);

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "expected {}", self.0)
    }
}

impl std::error::Error for NameError {}

macro_rules! name_type {
    ($name:ident, $pattern:literal, $first:expr, $rest:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl FromStr for $name {
            type Err = NameError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let mut chars = value.chars();
                if chars.next().is_some_and($first) && chars.all($rest) {
                    Ok(Self(value.to_owned()))
                } else {
                    Err(NameError($pattern))
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

fn identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn identifier_rest(ch: char) -> bool {
    identifier_start(ch) || ch.is_ascii_digit()
}

fn output_character(ch: char) -> bool {
    identifier_rest(ch) || ch == '-'
}

name_type!(
    ModuleName,
    "[A-Za-z_][A-Za-z0-9_]*",
    identifier_start,
    identifier_rest
);
name_type!(
    FieldName,
    "[A-Za-z_][A-Za-z0-9_]*",
    identifier_start,
    identifier_rest
);
name_type!(
    OutputName,
    "[A-Za-z0-9][A-Za-z0-9_-]*",
    |ch: char| ch.is_ascii_alphanumeric(),
    output_character
);
name_type!(Tag, "[A-Za-z0-9_-]+", output_character, output_character);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldRef {
    pub module: ModuleName,
    pub field: FieldName,
}

impl FromStr for FieldRef {
    type Err = NameError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let expected = || NameError("a field reference in the form module.field");
        let (module, field) = value.split_once('.').ok_or_else(expected)?;
        Ok(Self {
            module: module.parse().map_err(|_| expected())?,
            field: field.parse().map_err(|_| expected())?,
        })
    }
}

impl fmt::Display for FieldRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.module, self.field)
    }
}
