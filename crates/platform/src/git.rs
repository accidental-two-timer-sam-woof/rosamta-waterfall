//! Adapter for the system Git executable (05 §4.16).

use crate::SystemFs;
use rosamta_core::{
    fs::{FileSystem, Kind},
    init::GitClient,
    path::HomePath,
};
use std::process::{Command, Stdio};

impl GitClient for SystemFs {
    fn clone_into(
        &self,
        url: &str,
        staging: &HomePath,
        expected_identity: &str,
    ) -> Result<(), String> {
        let chain = self
            .chain(staging.as_str())
            .map_err(|error| format!("{staging}: {error}"))?;
        let dir = chain.last().ok_or("missing staging directory handle")?;
        let id = dir.id().map_err(|error| error.to_string())?;
        if id != expected_identity {
            return Err(format!(
                "{staging}: staging directory changed before cloning"
            ));
        }
        let mut command = Command::new("git");
        command
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "clone",
                "--template=",
                "--",
                url,
                ".",
            ])
            .stdin(Stdio::null())
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GCM_INTERACTIVE", "never")
            .env("SSH_ASKPASS_REQUIRE", "never")
            .env("GIT_ALLOW_PROTOCOL", "https:ssh");
        for key in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_INDEX_FILE",
            "GIT_OBJECT_DIRECTORY",
            "GIT_COMMON_DIR",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        ] {
            command.env_remove(key);
        }
        dir.enter_child_process(&mut command);
        let result = command.output();
        let current = self.inspect(staging);
        if !matches!(current, Ok(Some(ref entry)) if entry.kind == Kind::Directory && entry.identity == id)
        {
            return Err(format!(
                "{staging}: staging directory changed while Git was running"
            ));
        }
        let output = result.map_err(|error| format!("could not run git clone: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "git clone {url} failed ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(())
    }
}
