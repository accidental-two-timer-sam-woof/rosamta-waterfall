use rosamta_core::{
    apply::{Choices, deploy},
    host::{HostContext, Platform},
    path::HomePath,
};
use rosamta_platform::SystemFs;
use rosamta_testkit::{FaultFs, temporary_home};
use std::{cell::Cell, fs, io};

#[test]
fn sixth_output_and_state_commit_failures_roll_back_previous_writes() {
    let platform = match std::env::consts::OS {
        "windows" => Platform::Windows,
        "macos" => Platform::Macos,
        _ => Platform::Linux,
    };
    for failure_at in ["file-06", ".local/state/rosamta/state.json"] {
        let fixture = temporary_home().unwrap();
        let host = HostContext::new(platform, "fault-test".into(), fixture.path().into(), []);
        let filesystem = SystemFs::open(fixture.path()).unwrap();
        let source = fixture.path().join(".local/share/rosamta");
        fs::create_dir_all(&source).unwrap();
        let names = (1..=10)
            .map(|index| format!("file-{index:02}"))
            .collect::<Vec<_>>();
        let manifest = names
            .iter()
            .map(|name| format!("[outputs.{name}]\ncopy = \"contents\"\ntarget = \"~/{name}\"\n"))
            .collect::<String>();
        let profile = format!(
            "[[rules]]\nenable = [{}]\n",
            names
                .iter()
                .map(|name| format!("\"{name}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        fs::write(source.join("manifest.toml"), manifest).unwrap();
        fs::write(source.join("profile.toml"), profile).unwrap();
        fs::write(source.join("contents"), "before").unwrap();
        let (_, first) =
            deploy(&filesystem, &host, &Choices::default(), "20261111T010101Z").unwrap();
        assert_eq!(
            first.exit_code(),
            0,
            "{:?}",
            first.failure.as_ref().map(|failure| &failure.message)
        );
        let state = fixture.path().join(".local/state/rosamta/state.json");
        let original_state = fs::read(&state).unwrap();
        fs::write(source.join("contents"), "after").unwrap();
        let observed_updates = Cell::new(None);
        let faults = FaultFs {
            inner: &filesystem,
            before_change: |path: &HomePath| {
                if path.as_str() != failure_at {
                    return Ok(());
                }
                let count = names
                    .iter()
                    .filter(|name| {
                        fs::read(fixture.path().join(name)).is_ok_and(|bytes| bytes == b"after")
                    })
                    .count();
                observed_updates.set(Some(count));
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected write failure",
                ))
            },
        };
        let (_, report) = deploy(&faults, &host, &Choices::default(), "20261111T010102Z").unwrap();
        assert_eq!(report.exit_code(), 1);
        assert!(report.restored);
        assert!(report.recovery.is_empty());
        assert_eq!(
            observed_updates.get(),
            Some(if failure_at == "file-06" { 5 } else { 10 })
        );
        for name in &names {
            assert_eq!(fs::read(fixture.path().join(name)).unwrap(), b"before");
        }
        assert_eq!(fs::read(&state).unwrap(), original_state);
    }
}
