use super::{ConfigError, Manifest, ModuleFile, parse};

pub fn modules() -> Result<ModuleFile, ConfigError> {
    parse("builtin:modules", include_bytes!("builtin/modules.toml"))
}

pub fn manifest() -> Result<Manifest, ConfigError> {
    parse("builtin:manifest", include_bytes!("builtin/manifest.toml"))
}
