//! Format-preserving TOML editing (05 §4.7).

use toml_edit::{DocumentMut, Item, Table, TableLike, Value};

pub struct Editor {
    pub document: DocumentMut,
    bom: bool,
    crlf: bool,
}

impl Editor {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let (bytes, bom) = bytes
            .strip_prefix(b"\xef\xbb\xbf")
            .map_or((bytes, false), |bytes| (bytes, true));
        let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
        let mut endings = text
            .split_inclusive('\n')
            .filter(|line| line.ends_with('\n'))
            .map(|line| line.ends_with("\r\n"));
        let crlf = endings.next().unwrap_or(false);
        if let Some(index) = endings.position(|other| other != crlf) {
            return Err(format!(
                "line {} uses a different line ending; use either LF or CRLF",
                index + 2
            ));
        }
        let text = text.replace("\r\n", "\n");
        let document = text
            .parse::<DocumentMut>()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            document,
            bom,
            crlf,
        })
    }

    pub fn bytes(self) -> Vec<u8> {
        let text = self.document.to_string();
        let text = if self.crlf {
            text.replace('\n', "\r\n")
        } else {
            text
        };
        let mut bytes = Vec::with_capacity(text.len() + usize::from(self.bom) * 3);
        if self.bom {
            bytes.extend_from_slice(b"\xef\xbb\xbf");
        }
        bytes.extend_from_slice(text.as_bytes());
        bytes
    }

    pub fn table<'a>(&'a mut self, keys: &[&str]) -> Result<&'a mut dyn TableLike, String> {
        table(self.document.as_table_mut(), keys)
    }
}

fn table<'a>(
    parent: &'a mut dyn TableLike,
    keys: &[&str],
) -> Result<&'a mut dyn TableLike, String> {
    let Some((key, rest)) = keys.split_first() else {
        return Ok(parent);
    };
    if !parent.contains_key(key) {
        let mut child = Table::new();
        child.set_implicit(!rest.is_empty());
        parent.insert(key, Item::Table(child));
    }
    let child = parent
        .get_mut(key)
        .and_then(Item::as_table_like_mut)
        .ok_or_else(|| format!("{key} must be a table"))?;
    table(child, rest)
}

pub fn set(table: &mut dyn TableLike, key: &str, mut value: Value) {
    if let Some(old) = table.get_mut(key).and_then(Item::as_value_mut) {
        if old.to_string().trim() == value.to_string().trim() {
            return;
        }
        *value.decor_mut() = old.decor().clone();
        *old = value;
    } else {
        table.insert(key, Item::Value(value));
    }
}

pub fn parse_assignment_value(value: &str) -> Value {
    match value.parse::<Value>() {
        Ok(value) if !value.is_datetime() => value,
        _ => Value::from(value),
    }
}
