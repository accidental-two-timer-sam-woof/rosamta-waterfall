//! P1: the last check stays bound to the operation while another process
//! replaces a directory on the way to the target (04 §3.3, A10, A16, A20;
//! DC-002, SEC-002).
//!
//! Layout of each case, under a scratch root:
//!
//! ```text
//! home/a/b/target    the target ("original"; absent for create)
//! home/alt/...       a directory inside home with a same-named target
//! outside/...        a directory outside home with a same-named target
//! ```
//!
//! At a [`Sync`] point the harness runs `pilot p1-swap` as a separate
//! process, which replaces the parent (`b`) or an ancestor (`a`). Whatever
//! the platform does with that attempt, these must hold:
//!
//! - no file outside home and no file in the replacement directory changes;
//! - a snapshot never contains the replacement's content;
//! - when the swap succeeded, the operation is refused, or undone through
//!   the held handle, or (for a snapshot) reported as invalid;
//! - when the platform blocked the swap, the operation completes as planned
//!   (04 §5: the injection must be shown to be what was blocked).
//!
//! The churn tests replace the parent in a loop while operations run.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rosamta_pilot::atomic::TEMP_MARKER;
use rosamta_pilot::bound::{self, Op, Outcome, Preflight, Swap, Sync};
use rosamta_pilot::platform::{ScratchDir, identity, report};

const ORIGINAL: &[u8] = b"original\n";
const NEW: &[u8] = b"new content\n";
const REPLACEMENT: &[u8] = b"replacement inside home\n";
const OUTSIDE: &[u8] = b"outside home\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Level {
    Parent,
    Ancestor,
}

impl Level {
    fn name(self) -> &'static str {
        match self {
            Level::Parent => "parent",
            Level::Ancestor => "ancestor",
        }
    }
}

/// A snapshot of a directory tree that never follows links.
fn tree(root: &Path) -> BTreeMap<PathBuf, String> {
    fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            let rel = path.strip_prefix(root).unwrap().to_path_buf();
            let meta = fs::symlink_metadata(&path).unwrap();
            if meta.is_dir() && !meta.file_type().is_symlink() && !is_reparse(&meta) {
                out.insert(rel, "dir".into());
                walk(root, &path, out);
            } else if meta.is_file() {
                let bytes = fs::read(&path).unwrap();
                out.insert(rel, String::from_utf8_lossy(&bytes).into_owned());
            } else {
                out.insert(rel, "link".into());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

#[cfg(windows)]
fn is_reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn is_reparse(_: &fs::Metadata) -> bool {
    false
}

fn temps(root: &Path) -> Vec<PathBuf> {
    tree(root)
        .into_keys()
        .filter(|p| p.to_string_lossy().contains(TEMP_MARKER))
        .collect()
}

/// Writes `files` (relative path, content) under `dir`.
fn populate(dir: &Path, files: &[(&str, &[u8])]) {
    fs::create_dir_all(dir).unwrap();
    for (rel, content) in files {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
}

struct Layout {
    _scratch: ScratchDir,
    root: PathBuf,
    home: PathBuf,
    rel: PathBuf,
    /// The directory the swap replaces.
    victim: PathBuf,
    aside: PathBuf,
    alt: PathBuf,
    outside: PathBuf,
    /// The target relative to `victim`.
    target_in_victim: PathBuf,
}

impl Layout {
    fn new(label: &str, level: Level, target_exists: bool) -> Layout {
        let scratch = ScratchDir::new(label).unwrap();
        let root = scratch.path().to_path_buf();
        let home = root.join("home");
        let (victim, target_in_victim) = match level {
            Level::Parent => (home.join("a").join("b"), PathBuf::from("target")),
            Level::Ancestor => (home.join("a"), Path::new("b").join("target")),
        };
        let rel_target = target_in_victim.to_str().unwrap().replace('\\', "/");
        fs::create_dir_all(home.join("a").join("b")).unwrap();
        if target_exists {
            fs::write(home.join("a").join("b").join("target"), ORIGINAL).unwrap();
        }
        let alt = home.join("alt");
        let outside = root.join("outside");
        populate(&alt, &[(&rel_target, REPLACEMENT)]);
        populate(&outside, &[(&rel_target, OUTSIDE)]);
        Layout {
            _scratch: scratch,
            aside: home.join("aside"),
            rel: Path::new("a").join("b").join("target"),
            root,
            home,
            victim,
            alt,
            outside,
            target_in_victim,
        }
    }

    /// The `other` argument of the swap.
    fn other(&self, swap: Swap) -> PathBuf {
        match swap {
            Swap::LinkOut => self.outside.clone(),
            Swap::DirIn => self.alt.clone(),
            Swap::MoveOut => self.outside.join("moved"),
        }
    }

    /// Where the original victim directory lives after a successful swap.
    fn moved_to(&self, swap: Swap) -> PathBuf {
        match swap {
            Swap::LinkOut | Swap::DirIn => self.aside.clone(),
            Swap::MoveOut => self.outside.join("moved"),
        }
    }
}

fn spawn_pilot(args: &[&Path]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pilot"));
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

/// Runs `pilot p1-swap` and returns `Ok(())` when it swapped, or the error
/// that blocked it.
fn swap_in_child(swap: Swap, victim: &Path, aside: &Path, other: &Path) -> Result<(), String> {
    let output = spawn_pilot(&[
        Path::new("p1-swap"),
        Path::new(swap.name()),
        victim,
        aside,
        other,
    ])
    .output()
    .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        output.status.success(),
        "p1-swap failed: {stdout} {}",
        String::from_utf8_lossy(&output.stderr)
    );
    match stdout.strip_prefix("blocked ") {
        None if stdout == "swapped" => Ok(()),
        Some(reason) => Err(reason.to_string()),
        None => panic!("unexpected p1-swap output: {stdout}"),
    }
}

#[derive(Clone, Copy)]
enum OpKind {
    Create,
    Update,
    Remove,
    Snapshot,
}

impl OpKind {
    fn op(self) -> Op {
        match self {
            OpKind::Create => Op::Create(NEW.to_vec()),
            OpKind::Update => Op::Update(NEW.to_vec()),
            OpKind::Remove => Op::Remove,
            OpKind::Snapshot => Op::Snapshot,
        }
    }
}

/// The victim tree as the operation would leave it.
fn after_op(
    before: &BTreeMap<PathBuf, String>,
    target: &Path,
    kind: OpKind,
) -> BTreeMap<PathBuf, String> {
    let mut tree = before.clone();
    let new = String::from_utf8_lossy(NEW).into_owned();
    match kind {
        OpKind::Create | OpKind::Update => {
            tree.insert(target.to_path_buf(), new);
        }
        OpKind::Remove => {
            tree.remove(target);
        }
        OpKind::Snapshot => {}
    }
    tree
}

fn run_case(level: Level, swap: Swap, sync: Sync, kind: OpKind) {
    let op = kind.op();
    let label = format!(
        "p1-{}-{}-{}-{}",
        level.name(),
        swap.name(),
        sync.name(),
        op.name()
    );
    let layout = Layout::new(&label, level, !matches!(kind, OpKind::Create));
    let pre = Preflight::take(&layout.home, &layout.rel).unwrap();
    let victim_before = tree(&layout.victim);
    let alt_before = tree(&layout.alt);
    let outside_before = tree(&layout.outside);

    let mut swapped = None;
    let outcome = bound::run(&layout.home, &layout.rel, &pre, &op, &mut |point| {
        if point == sync {
            swapped = Some(swap_in_child(
                swap,
                &layout.victim,
                &layout.aside,
                &layout.other(swap),
            ));
        }
    });
    let swapped = swapped.unwrap_or_else(|| Err("sync point not reached".into()));

    // Report before asserting, so a failing case still leaves its result.
    // Move-out puts the held directory outside home, so a write through the
    // handle lands there until the recheck undoes it.
    let wrote_outside = swap == Swap::MoveOut
        && swapped.is_ok()
        && sync == Sync::AfterOpen
        && !matches!(kind, OpKind::Snapshot);
    report(
        "P1",
        &[
            ("level", &level.name()),
            ("swap", &swap.name()),
            ("sync", &sync.name()),
            ("op", &op.name()),
            (
                "injection",
                &match &swapped {
                    Ok(()) => "swapped".to_string(),
                    Err(e) => format!("blocked({})", e.replace(' ', "_")),
                },
            ),
            (
                "outcome",
                &match &outcome {
                    Ok(o) => o.name().to_string(),
                    Err(e) => format!("error({})", e.to_string().replace(' ', "_")),
                },
            ),
            ("wrote_outside_home_transiently", &wrote_outside),
        ],
    );
    let outcome = outcome.unwrap_or_else(|e| panic!("{label}: {e}"));

    // Where the original directory is now, and what it must hold.
    let (original_dir, expected_victim) = match (&swapped, &outcome) {
        (Ok(()), Outcome::Done { .. }) => {
            panic!("{label}: swap succeeded but the operation reported done")
        }
        (Ok(()), _) => (layout.moved_to(swap), victim_before.clone()),
        (Err(_), Outcome::Done { .. }) => (
            layout.victim.clone(),
            after_op(&victim_before, &layout.target_in_victim, kind),
        ),
        (Err(e), other) => {
            panic!("{label}: swap blocked ({e}) but the operation did not complete: {other:?}")
        }
    };
    let original_after = tree(&original_dir);

    // A succeeding swap that never reached the operation must be refused.
    if sync == Sync::AfterPreflight {
        assert!(
            swapped.is_ok(),
            "{label}: injection did not happen: {swapped:?}"
        );
        assert!(
            matches!(outcome, Outcome::Refused(_)),
            "{label}: {outcome:?}"
        );
    }
    if sync == Sync::AfterOpen && swapped.is_ok() {
        let ok = match kind {
            OpKind::Snapshot => {
                matches!(&outcome, Outcome::Invalidated { snapshot, .. } if snapshot == ORIGINAL)
            }
            _ => matches!(outcome, Outcome::RolledBack(_)),
        };
        assert!(ok, "{label}: {outcome:?}");
    }
    if let Outcome::Done {
        snapshot: Some(bytes),
    } = &outcome
    {
        assert_eq!(bytes, ORIGINAL, "{label}: snapshot content");
    }

    // The replacement directory and everything outside home are unchanged.
    // For move-out, the original directory itself sits outside home.
    let alt_after = tree(match (swap, &swapped) {
        (Swap::DirIn, Ok(())) => &layout.victim,
        _ => &layout.alt,
    });
    assert_eq!(
        alt_after, alt_before,
        "{label}: replacement directory changed"
    );
    let mut outside_after = tree(&layout.outside);
    if swap == Swap::MoveOut && swapped.is_ok() {
        outside_after.retain(|p, _| !p.starts_with("moved"));
    }
    assert_eq!(
        outside_after, outside_before,
        "{label}: outside home changed"
    );
    assert_eq!(
        original_after, expected_victim,
        "{label}: original directory"
    );
    assert!(
        temps(&layout.root).is_empty(),
        "{label}: leftover temporary files"
    );
}

/// Runs every case even after a failure, so one CI run shows the whole
/// matrix.
fn run_matrix(sync: Sync) {
    let mut failures = Vec::new();
    for level in [Level::Parent, Level::Ancestor] {
        for swap in [Swap::LinkOut, Swap::DirIn, Swap::MoveOut] {
            for kind in [
                OpKind::Create,
                OpKind::Update,
                OpKind::Remove,
                OpKind::Snapshot,
            ] {
                let case = std::panic::catch_unwind(|| run_case(level, swap, sync, kind));
                if let Err(panic) = case {
                    failures.push(
                        panic
                            .downcast_ref::<String>()
                            .cloned()
                            .unwrap_or_else(|| "panic".into()),
                    );
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn p1_identity() {
    report(
        "ENV",
        &[
            ("os", &std::env::consts::OS),
            ("identity", &identity().unwrap()),
        ],
    );
}

/// A10, A20: a replacement made after the preflight is caught by the walk.
#[test]
fn p1_swap_after_preflight_is_refused() {
    run_matrix(Sync::AfterPreflight);
}

/// A10, A16, A20: a replacement after the last check either is blocked or
/// cannot redirect the operation.
#[test]
fn p1_swap_after_last_check_is_contained() {
    run_matrix(Sync::AfterOpen);
}

/// Keeps replacing the parent from another process while updates run. Only
/// the invariants are asserted; the counts show how often the race hit.
fn churn(swap: Swap, rounds: usize) {
    let label = format!("p1-churn-{}", swap.name());
    let layout = Layout::new(&label, Level::Parent, true);
    let fixed = Preflight::take(&layout.home, &layout.rel).unwrap().dirs;
    let alt_before = tree(&layout.alt);
    let outside_before = tree(&layout.outside);
    let stop = layout.root.join("stop");

    let mut child = spawn_pilot(&[
        Path::new("p1-churn"),
        Path::new(swap.name()),
        &layout.victim,
        &layout.aside,
        &layout.other(swap),
        &stop,
    ])
    .spawn()
    .unwrap();

    let payloads: [&[u8]; 2] = [b"payload A\n", b"payload B\n"];
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for round in 0..rounds {
        // The preflight may itself land mid-swap; the directory identities
        // always come from the undisturbed layout.
        let Ok(mut pre) = Preflight::take(&layout.home, &layout.rel) else {
            *counts.entry("preflight-failed").or_default() += 1;
            continue;
        };
        pre.dirs = fixed.clone();
        let op = Op::Update(payloads[round % 2].to_vec());
        let outcome = bound::run(&layout.home, &layout.rel, &pre, &op, &mut |_| {}).unwrap();
        *counts.entry(outcome.name()).or_default() += 1;
    }

    fs::write(&stop, b"").unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.as_mut().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert!(child.wait().unwrap().success());

    let content = fs::read(layout.victim.join("target")).unwrap();
    assert!(
        content == ORIGINAL || payloads.contains(&content.as_slice()),
        "{label}: final target {content:?}"
    );
    assert_eq!(
        tree(&layout.alt),
        alt_before,
        "{label}: replacement directory changed"
    );
    assert_eq!(
        tree(&layout.outside),
        outside_before,
        "{label}: outside home changed"
    );
    assert!(
        temps(&layout.root).is_empty(),
        "{label}: leftover temporary files"
    );
    assert!(!counts.contains_key("invalidated"));

    let counts = counts
        .iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect::<Vec<_>>()
        .join(",");
    report(
        "P1",
        &[
            ("case", &"churn"),
            ("swap", &swap.name()),
            ("rounds", &rounds),
            ("outcomes", &counts),
            ("churn", &line.trim().replace(' ', ",")),
        ],
    );
}

#[test]
fn p1_churn_dir_in() {
    churn(Swap::DirIn, 300);
}

/// Windows creates junctions through `cmd /c mklink /J`, too slow to churn.
#[cfg(unix)]
#[test]
fn p1_churn_link_out() {
    churn(Swap::LinkOut, 300);
}
