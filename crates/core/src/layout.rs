//! Source, settings and state locations and reserved paths (05 §5.1–5.2).

use crate::{host::HostContext, path::HomePath};
use std::path::Path;

pub struct Layout {
    pub config: HomePath,
    pub source: HomePath,
    pub state: HomePath,
}

impl Layout {
    pub fn new(host: &HostContext) -> Result<Self, String> {
        fn directory(host: &HostContext, key: &str, default: &str) -> Result<HomePath, String> {
            match host.env(key) {
                None => HomePath::relative(default),
                Some(value) => HomePath::from_absolute(Path::new(value), host)
                    .and_then(|path| path.join("rosamta"))
                    .map_err(|error| format!("{key}: {error}")),
            }
        }
        let layout = Self {
            config: directory(host, "XDG_CONFIG_HOME", ".config/rosamta")?,
            source: directory(host, "XDG_DATA_HOME", ".local/share/rosamta")?,
            state: directory(host, "XDG_STATE_HOME", ".local/state/rosamta")?,
        };
        layout.validate(host)?;
        Ok(layout)
    }

    pub fn validate(&self, host: &HostContext) -> Result<(), String> {
        for (left, right) in [
            (&self.source, &self.state),
            (&self.source, &self.config),
            (&self.state, &self.config),
        ] {
            if left.starts_with(right, host.platform) || right.starts_with(left, host.platform) {
                return Err(format!(
                    "{left} overlaps {right}; source, configuration and state must use separate directories"
                ));
            }
        }
        Ok(())
    }

    pub fn reserved(&self, target: &HomePath, host: &HostContext) -> bool {
        [&self.config, &self.source, &self.state]
            .into_iter()
            .any(|path| {
                target.starts_with(path, host.platform) || path.starts_with(target, host.platform)
            })
    }
}
