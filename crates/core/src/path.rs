//! Home-relative and source-relative path types (05 §4.1 and §6.1).

use crate::host::{HostContext, Platform};
use std::{
    ffi::OsStr,
    fmt,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HomePath(String);

impl HomePath {
    pub fn relative(text: &str) -> Result<Self, String> {
        if text.starts_with('/') || text.contains('\\') {
            return Err(format!(
                "expected a relative path with / separators: {text:?}"
            ));
        }
        let mut parts = Vec::new();
        for component in text.split('/') {
            match component {
                "" | "." => {}
                ".." => {
                    parts
                        .pop()
                        .ok_or_else(|| format!("path escapes its root: {text:?}"))?;
                }
                part => {
                    validate_component(part)?;
                    parts.push(part);
                }
            }
        }
        if parts.is_empty() {
            return Err(format!("path must name an entry below its root: {text:?}"));
        }
        Ok(Self(parts.join("/")))
    }

    pub fn from_absolute(path: &Path, host: &HostContext) -> Result<Self, String> {
        let path = normalize(path)?;
        let parts = std::iter::once(&host.home)
            .chain(host.actual_home.iter())
            .find_map(|home| {
                let home = normalize(home).ok()?;
                let mut parts = path.components();
                home.components()
                    .all(|expected| {
                        parts.next().is_some_and(|actual| match host.platform {
                            Platform::Windows => actual
                                .as_os_str()
                                .eq_ignore_ascii_case(expected.as_os_str()),
                            _ => actual == expected,
                        })
                    })
                    .then_some(parts)
            })
            .ok_or_else(|| format!("{} is outside the home directory", path.display()))?;
        let parts = parts
            .map(|part| part.as_os_str().to_str().ok_or("path must be UTF-8"))
            .collect::<Result<Vec<_>, _>>()?;
        Self::relative(&parts.join("/"))
    }

    pub fn expand(text: &str, host: &HostContext) -> Result<Self, String> {
        if text.contains('\\') {
            return Err(format!("use / in configured path {text:?}"));
        }
        let mut output = String::new();
        let mut rest = text;
        if text == "~" || text.starts_with("~/") {
            output.push_str(host.home.to_str().ok_or("home must be UTF-8")?);
            rest = &rest[1..];
        }
        while let Some((prefix, suffix)) = rest.split_once('$') {
            output.push_str(prefix);
            if let Some(suffix) = suffix.strip_prefix('$') {
                output.push('$');
                rest = suffix;
            } else if let Some((name, suffix)) = suffix
                .strip_prefix('{')
                .and_then(|suffix| suffix.split_once('}'))
            {
                if !name
                    .bytes()
                    .next()
                    .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
                    || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                {
                    return Err(format!("invalid environment variable name {name:?}"));
                }
                let value = host
                    .env(name)
                    .and_then(OsStr::to_str)
                    .ok_or_else(|| format!("{name} is unset or not UTF-8"))?;
                output.push_str(value);
                rest = suffix;
            } else {
                return Err(format!("invalid dollar expansion in path {text:?}"));
            }
        }
        output.push_str(rest);
        Self::from_absolute(Path::new(&output), host).map_err(|error| format!("{text:?}: {error}"))
    }

    pub fn argument(value: &OsStr, host: &HostContext, cwd: &Path) -> Result<Self, String> {
        let text = value.to_str().ok_or("path must be UTF-8")?;
        let path = if text == "~" {
            host.home.clone()
        } else if let Some(rest) = text.strip_prefix("~/") {
            host.home.join(rest)
        } else {
            cwd.join(text)
        };
        Self::from_absolute(&path, host)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn to_absolute(&self, host: &HostContext) -> PathBuf {
        let mut path = host.home.clone();
        path.extend(self.0.split('/'));
        path
    }
    pub fn join(&self, suffix: &str) -> Result<Self, String> {
        let suffix = Self::relative(suffix)?;
        Ok(Self(format!("{}/{}", self.0, suffix.0)))
    }
    pub fn parent(&self) -> Option<Self> {
        self.0
            .rsplit_once('/')
            .map(|(parent, _)| Self(parent.to_owned()))
    }
    pub fn starts_with(&self, other: &Self, platform: Platform) -> bool {
        let this = self.key(platform);
        let other = other.key(platform);
        this == other
            || this
                .strip_prefix(&other)
                .is_some_and(|tail| tail.starts_with('/'))
    }
    pub fn key(&self, platform: Platform) -> String {
        if platform == Platform::Windows {
            self.0.to_lowercase()
        } else {
            self.0.clone()
        }
    }
}

impl fmt::Display for HomePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "~/{}", self.0)
    }
}

fn validate_component(part: &str) -> Result<(), String> {
    let device = part
        .split('.')
        .next()
        .unwrap_or(part)
        .trim_end()
        .to_ascii_uppercase();
    let numbered = |prefix| {
        device.strip_prefix(prefix).is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    };
    if part.ends_with([' ', '.'])
        || part
            .chars()
            .any(|ch| ch.is_control() || "<>:\"|?*".contains(ch))
        || matches!(
            device.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        )
        || numbered("COM")
        || numbered("LPT")
    {
        return Err(format!("unsupported path component {part:?}"));
    }
    Ok(())
}

fn normalize(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!("expected an absolute path, got {}", path.display()));
    }
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::Prefix(prefix) if !matches!(prefix.kind(), std::path::Prefix::Disk(_)) => {
                return Err("UNC and device paths are unsupported".into());
            }
            Component::ParentDir if result.file_name().is_none() => {
                return Err(format!(
                    "path escapes its filesystem root: {}",
                    path.display()
                ));
            }
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => result.push(part.as_os_str()),
        }
    }
    Ok(result)
}
