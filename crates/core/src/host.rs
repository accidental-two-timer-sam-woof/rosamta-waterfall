//! Host context passed into configuration and profile matching (05 §4.1).

use serde::Deserialize;
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Macos,
    Windows,
}

impl Platform {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Macos => "macos",
            Self::Windows => "windows",
        }
    }
}

pub struct HostContext {
    pub platform: Platform,
    pub hostname: String,
    pub home: PathBuf,
    env: Vec<(OsString, OsString)>,
}

impl HostContext {
    pub fn new(
        platform: Platform,
        hostname: String,
        home: PathBuf,
        env: impl IntoIterator<Item = (OsString, OsString)>,
    ) -> Self {
        Self {
            platform,
            hostname,
            home,
            env: env.into_iter().collect(),
        }
    }

    pub fn env(&self, name: &str) -> Option<&OsStr> {
        let name = OsStr::new(name);
        self.env
            .iter()
            .find(|(key, _)| match self.platform {
                Platform::Windows => key.eq_ignore_ascii_case(name),
                _ => key == name,
            })
            .map(|(_, value)| value.as_os_str())
    }
}
