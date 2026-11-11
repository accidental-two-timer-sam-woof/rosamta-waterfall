//! Output, value and reference queries (05 §4.9).

use std::str::FromStr;

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
