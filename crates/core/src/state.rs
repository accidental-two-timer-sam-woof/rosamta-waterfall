//! Managed output records and state persistence (05 §4.11).

use crate::{
    fs::{Entry, FileSystem},
    host::Platform,
    layout::Layout,
    name::OutputName,
    path::HomePath,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Parent {
    pub file_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
}

impl Parent {
    pub fn from_entry(entry: &Entry, platform: Platform) -> Self {
        let (volume, file_id) = entry.parent.split_once(':').unwrap_or(("", &entry.parent));
        Self {
            file_id: file_id.into(),
            volume: (platform == Platform::Windows).then(|| volume.to_owned()),
        }
    }

    pub fn matches(&self, entry: &Entry, platform: Platform) -> bool {
        self == &Self::from_entry(entry, platform)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Fingerprint {
    pub sha256: String,
    pub mode: Option<String>,
}

impl Fingerprint {
    pub fn new(bytes: &[u8], mode: Option<u32>) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut sha256 = String::with_capacity(64);
        for byte in Sha256::digest(bytes) {
            sha256.push(HEX[usize::from(byte >> 4)].into());
            sha256.push(HEX[usize::from(byte & 15)].into());
        }
        Self {
            sha256,
            mode: mode.map(|mode| format!("{:04o}", mode & 0o777)),
        }
    }

    pub fn matches(&self, entry: &Entry) -> bool {
        *self
            == Self::new(
                &entry.bytes,
                if self.mode.is_some() {
                    entry.mode
                } else {
                    None
                },
            )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub name: String,
    pub target: String,
    pub parent: Parent,
    pub observed: Fingerprint,
    pub desired: Fingerprint,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub forget_pending: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StateFile {
    format: u32,
    outputs: Vec<Record>,
}

pub struct State {
    pub path: HomePath,
    pub before: Option<Entry>,
    pub records: BTreeMap<String, Record>,
}

impl State {
    pub fn load(fs: &dyn FileSystem, layout: &Layout, platform: Platform) -> Result<Self, String> {
        let path = layout.state.join("state.json")?;
        let before = fs
            .inspect(&path)
            .map_err(|error| format!("{path}: {error}"))?;
        let mut records = BTreeMap::new();
        if let Some(entry) = &before {
            if entry.kind != crate::fs::Kind::Regular {
                return Err(format!("{path}: state must be a regular file"));
            }
            let header: serde_json::Value =
                serde_json::from_slice(&entry.bytes).map_err(|e| format!("{path}: {e}"))?;
            if header["format"].as_u64().is_some_and(|version| version > 1) {
                return Err(format!(
                    "{path}: state was written by a newer version of rosamta"
                ));
            }
            let state: StateFile =
                serde_json::from_slice(&entry.bytes).map_err(|e| format!("{path}: {e}"))?;
            if state.format != 1 {
                return Err(format!("{path}: invalid state version"));
            }
            for record in state.outputs {
                let target =
                    HomePath::relative(&record.target).map_err(|e| format!("{path}: {e}"))?;
                let hex = |text: &str| {
                    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_hexdigit())
                };
                let valid = |data: &Fingerprint| {
                    data.sha256.len() == 64
                        && hex(&data.sha256)
                        && data.mode.as_ref().is_none_or(|mode| {
                            mode.len() == 4
                                && mode.starts_with('0')
                                && mode.bytes().all(|digit| matches!(digit, b'0'..=b'7'))
                        })
                };
                if target.as_str() != record.target
                    || record.name.parse::<OutputName>().is_err()
                    || !valid(&record.observed)
                    || !valid(&record.desired)
                    || !hex(&record.parent.file_id)
                    || record
                        .parent
                        .volume
                        .as_ref()
                        .is_some_and(|volume| !hex(volume))
                {
                    return Err(format!(
                        "{path}: invalid state record for {}",
                        record.target
                    ));
                }
                if records.insert(target.key(platform), record).is_some() {
                    return Err(format!("{path}: duplicate state target {target}"));
                }
            }
        }
        Ok(Self {
            path,
            before,
            records,
        })
    }

    pub fn encode(records: &BTreeMap<String, Record>) -> Result<Vec<u8>, String> {
        let mut outputs: Vec<_> = records.values().cloned().collect();
        outputs.sort_by(|left, right| left.target.cmp(&right.target));
        let mut bytes = serde_json::to_vec_pretty(&StateFile { format: 1, outputs })
            .map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn commit(
        &mut self,
        fs: &dyn FileSystem,
        records: BTreeMap<String, Record>,
    ) -> Result<(), String> {
        let bytes = Self::encode(&records)?;
        self.before = fs
            .change(
                &self.path,
                self.before.as_ref(),
                Some(crate::fs::Replacement {
                    bytes: &bytes,
                    mode: Some(0o600),
                }),
            )
            .map_err(|error| format!("{}: {error}", self.path))?;
        self.records = records;
        Ok(())
    }
}
