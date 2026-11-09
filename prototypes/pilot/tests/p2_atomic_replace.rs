//! P2: single-file atomic replacement under forced termination.
//!
//! REL-001 and REL-002 acceptance: kill the writer at a random moment 100
//! times; every time, the target holds a complete old or new version. The
//! child replaces the target in a loop, alternating payloads A and B, so each
//! kill lands somewhere inside a replacement. `std::process::Child::kill`
//! sends SIGKILL on Unix and calls TerminateProcess on Windows.
//!
//! Environment: `PILOT_P2_KILLS` (default 100) and `PILOT_SEED` (default
//! below) replay a run.

use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use rosamta_pilot::atomic::{self, Method};
use rosamta_pilot::payload::{self, Rng};
use rosamta_pilot::platform::{ScratchDir, identity, report};

const OUTPUT_SIZE: usize = 1024 * 1024; // REL-001: 1 MB output
const STATE_SIZE: usize = 4 * 1024; // REL-002: a small state file
const MAX_DELAY_US: u64 = 20_000;

fn kills() -> usize {
    std::env::var("PILOT_P2_KILLS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100)
}

fn seed() -> u64 {
    std::env::var("PILOT_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20261109)
}

#[derive(Default)]
struct Outcome {
    a: usize,
    b: usize,
    /// The target path did not exist.
    missing: usize,
    /// The target could not be read for another reason.
    unreadable: usize,
    /// The target held neither complete payload.
    torn: usize,
    leftover_temps: usize,
    temp_modes: std::collections::BTreeSet<String>,
    target_modes: std::collections::BTreeSet<String>,
}

fn mode_of(path: &Path) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::symlink_metadata(path)
            .map(|m| format!("{:o}", m.permissions().mode() & 0o7777))
            .unwrap_or_else(|_| "missing".into())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        "n/a".into()
    }
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

fn kill_loop(case: &str, size: usize, method: Method) -> Outcome {
    let dir = ScratchDir::new(&format!("p2-{case}-{}", method.name())).unwrap();
    let target = dir.path().join("target.out");
    let a = payload::build(b'A', size);
    let b = payload::build(b'B', size);
    let seed = seed();
    let mut rng = Rng::new(seed ^ size as u64);
    let mut outcome = Outcome::default();

    for _ in 0..kills() {
        fs::write(&target, &a).unwrap();
        #[cfg(unix)]
        set_mode(&target, 0o600);

        let mut child = Command::new(env!("CARGO_BIN_EXE_pilot"))
            .args([
                "p2-write",
                target.to_str().unwrap(),
                &size.to_string(),
                method.name(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut line = String::new();
        BufReader::new(child.stdout.as_mut().unwrap())
            .read_line(&mut line)
            .unwrap();
        if line.trim() != "ready" {
            let mut err = String::new();
            child
                .stderr
                .as_mut()
                .unwrap()
                .read_to_string(&mut err)
                .unwrap();
            panic!("writer did not start: {err}");
        }
        std::thread::sleep(Duration::from_micros(rng.below(MAX_DELAY_US)));
        child.kill().unwrap();
        child.wait().unwrap();

        match fs::read(&target) {
            Ok(got) if got == a => outcome.a += 1,
            Ok(got) if got == b => outcome.b += 1,
            Ok(_) => outcome.torn += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => outcome.missing += 1,
            Err(_) => outcome.unreadable += 1,
        }
        outcome.target_modes.insert(mode_of(&target));
        for temp in atomic::leftover_temps(dir.path()).unwrap() {
            outcome.leftover_temps += 1;
            outcome.temp_modes.insert(mode_of(&temp));
            fs::remove_file(temp).unwrap();
        }
    }

    let join = |s: &std::collections::BTreeSet<String>| {
        if s.is_empty() {
            "-".to_string()
        } else {
            s.iter().cloned().collect::<Vec<_>>().join(",")
        }
    };
    report(
        "P2",
        &[
            ("case", &case),
            ("method", &method.name()),
            ("size", &size),
            ("kills", &kills()),
            ("seed", &seed),
            ("old_a", &outcome.a),
            ("new_b", &outcome.b),
            ("missing", &outcome.missing),
            ("unreadable", &outcome.unreadable),
            ("torn", &outcome.torn),
            ("leftover_temps", &outcome.leftover_temps),
            ("temp_modes", &join(&outcome.temp_modes)),
            ("target_modes", &join(&outcome.target_modes)),
        ],
    );
    outcome
}

#[test]
fn p2_identity() {
    report(
        "ENV",
        &[
            ("os", &std::env::consts::OS),
            ("identity", &identity().unwrap()),
        ],
    );
}

impl Outcome {
    /// Every kill left a complete old or new version at the target path.
    fn assert_intact(&self, what: &str) {
        assert_eq!(
            (self.missing, self.unreadable, self.torn),
            (0, 0, 0),
            "{what}: (missing, unreadable, torn)"
        );
    }
}

/// REL-001: a 1 MB output is never left half written.
#[test]
fn p2_output_survives_kill() {
    let o = kill_loop("output", OUTPUT_SIZE, Method::Rename);
    o.assert_intact("output");
    #[cfg(unix)]
    {
        assert_eq!(o.target_modes.iter().collect::<Vec<_>>(), ["600"]);
        assert!(
            o.temp_modes.iter().all(|m| m == "600"),
            "temp modes {:?}",
            o.temp_modes
        );
    }
}

/// REL-002: the state file is never left half written.
#[test]
fn p2_state_file_survives_kill() {
    kill_loop("state", STATE_SIZE, Method::Rename).assert_intact("state");
}

/// Rejected candidate: `ReplaceFileW` would keep the target's ACL, but on
/// the windows-2025 runner one kill out of 100 left the target neither old
/// nor new. Measured and reported only, as the evidence for `MoveFileExW`.
#[cfg(windows)]
#[test]
fn p2_replace_file_is_measured() {
    kill_loop("output", OUTPUT_SIZE, Method::ReplaceFile);
    kill_loop("state", STATE_SIZE, Method::ReplaceFile);
}

/// Negative control: the same harness catches a writer that is not atomic.
#[test]
fn p2_in_place_write_is_caught() {
    let o = kill_loop("control", OUTPUT_SIZE, Method::InPlace);
    assert!(o.torn > 0, "harness never caught a torn in-place write");
}

/// A6: without a declared mode, replacement keeps the existing permission bits.
#[cfg(unix)]
#[test]
fn p2_keeps_existing_mode() {
    let dir = ScratchDir::new("p2-mode").unwrap();
    for mode in [0o600, 0o640, 0o644, 0o755] {
        let target = dir.path().join(format!("file-{mode:o}"));
        fs::write(&target, b"old\n").unwrap();
        set_mode(&target, mode);
        atomic::replace(&target, b"new\n", Method::Rename).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new\n");
        assert_eq!(mode_of(&target), format!("{mode:o}"));
    }
    assert!(atomic::leftover_temps(dir.path()).unwrap().is_empty());
}
