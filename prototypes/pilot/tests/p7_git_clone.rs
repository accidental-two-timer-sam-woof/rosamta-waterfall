//! P7: `git clone` into a staging directory while another process replaces
//! a directory on the way to it (04 §5 A7; DC-002, ASM-22).
//!
//! Initialization clones the remote into a staging directory that rosamta
//! created and holds, then moves it into place (05 §4.16). git writes by
//! path, so the clone cannot go through the held handles the way the bound
//! operations do (P1). Layout of each case, under a scratch root:
//!
//! ```text
//! origin/                    a local repository, cloned through file://
//! home/a/b/stage/            the staging directory (held)
//! outside/b/stage/           a same-named directory outside home
//! ```
//!
//! After the staging directory is created and every level is held, a
//! separate process replaces `b` (or `a`) before git starts. Then:
//!
//! - `path`: git gets the staging directory's path (the design before P7).
//!   On Unix, a link swap sends the whole clone outside home.
//! - `held`: Unix changes the child's working directory to the held staging
//!   directory with `fchdir` and runs `git clone -- <url> .`; Windows holds
//!   every level without `FILE_SHARE_DELETE` while git runs on the path.
//!
//! With `held`, a link or directory swap must not put any file outside home
//! or in the replacement, and the clone must be in the held directory. A
//! move-out swap moves the held directory itself outside home on Unix, so
//! the clone lands there (ASM-06); the recheck after the clone must find
//! that the path no longer leads to it. Windows must block every swap and
//! still complete the clone.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use rosamta_pilot::bound::{Dir, Identity, Swap};
use rosamta_pilot::platform::{ScratchDir, identity, report};

const STAGE: &str = ".src.rosamta-tmp.1.0";

/// The files committed to the origin repository.
const FILES: &[(&str, &[u8])] = &[("a.txt", b"from origin\n"), ("dir/b.txt", b"nested\n")];

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Path,
    Held,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Mode::Path => "path",
            Mode::Held => "held",
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
                out.insert(rel, format!("{} bytes", meta.len()));
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

/// git with no system or global configuration and no prompts.
fn git(scratch: &Path) -> Command {
    let empty = scratch.join("gitconfig");
    if !empty.exists() {
        fs::write(&empty, b"").unwrap();
    }
    let mut command = Command::new("git");
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", &empty)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn check(output: Output, what: &str) {
    assert!(
        output.status.success(),
        "{what} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Creates `origin` with one commit and returns its file:// URL.
fn make_origin(scratch: &Path) -> String {
    let origin = scratch.join("origin");
    fs::create_dir(&origin).unwrap();
    for (rel, content) in FILES {
        let path = origin.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    let run = |args: &[&str]| {
        check(
            git(scratch)
                .current_dir(&origin)
                .args(args)
                .output()
                .unwrap(),
            &format!("git {}", args.join(" ")),
        )
    };
    run(&["-c", "init.defaultBranch=main", "init", "-q"]);
    run(&["add", "-A"]);
    run(&[
        "-c",
        "user.name=pilot",
        "-c",
        "user.email=pilot@example.invalid",
        "commit",
        "-q",
        "-m",
        "init",
    ]);
    let path = origin.to_string_lossy().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

struct Layout {
    scratch: ScratchDir,
    url: String,
    home: PathBuf,
    victim: PathBuf,
    aside: PathBuf,
    outside: PathBuf,
    alt: PathBuf,
    /// The staging directory relative to `victim`.
    stage_in_victim: PathBuf,
}

impl Layout {
    fn new(label: &str, level: Level) -> Layout {
        let scratch = ScratchDir::new(label).unwrap();
        let root = scratch.path().to_path_buf();
        let url = make_origin(&root);
        let home = root.join("home");
        fs::create_dir_all(home.join("a").join("b").join(STAGE)).unwrap();
        let (victim, stage_in_victim) = match level {
            Level::Parent => (home.join("a").join("b"), PathBuf::from(STAGE)),
            Level::Ancestor => (home.join("a"), Path::new("b").join(STAGE)),
        };
        // Same-named, empty staging directories outside home and in a
        // replacement inside home: a misdirected clone would fill them.
        let outside = root.join("outside");
        let alt = home.join("alt");
        fs::create_dir_all(outside.join(&stage_in_victim)).unwrap();
        fs::create_dir_all(alt.join(&stage_in_victim)).unwrap();
        Layout {
            url,
            aside: home.join("aside"),
            home,
            victim,
            outside,
            alt,
            stage_in_victim,
            scratch,
        }
    }

    fn stage(&self) -> PathBuf {
        self.home.join("a").join("b").join(STAGE)
    }

    fn other(&self, swap: Swap) -> PathBuf {
        match swap {
            Swap::LinkOut => self.outside.clone(),
            Swap::DirIn => self.alt.clone(),
            Swap::MoveOut => self.outside.join("moved"),
        }
    }

    /// Where the original victim directory is after a successful swap.
    fn moved_to(&self, swap: Swap) -> PathBuf {
        match swap {
            Swap::LinkOut | Swap::DirIn => self.aside.clone(),
            Swap::MoveOut => self.outside.join("moved"),
        }
    }
}

/// Every level from home to the staging directory, held open.
struct Held {
    dirs: Vec<Dir>,
    identities: Vec<Identity>,
}

impl Held {
    fn open(home: &Path) -> Held {
        let mut dirs = vec![Dir::open_root(home).unwrap()];
        for name in ["a", "b", STAGE] {
            let next = dirs.last().unwrap().open_child(OsStr::new(name)).unwrap();
            dirs.push(next);
        }
        let identities = dirs.iter().map(|d| d.identity().unwrap()).collect();
        Held { dirs, identities }
    }

    #[cfg_attr(windows, allow(dead_code))]
    fn stage(&self) -> &Dir {
        self.dirs.last().unwrap()
    }

    /// The recheck after the clone: walking from home again must reach the
    /// same directories (05 §4.16 step 3).
    fn still_reached(&self, home: &Path) -> bool {
        let Ok(mut dir) = Dir::open_root(home) else {
            return false;
        };
        for (i, name) in ["a", "b", STAGE].into_iter().enumerate() {
            match dir.open_child(OsStr::new(name)) {
                Ok(next) if next.identity().ok() == Some(self.identities[i + 1]) => dir = next,
                _ => return false,
            }
        }
        true
    }
}

/// Runs `pilot p1-swap` and returns `Ok(())` when it swapped, or the error
/// that blocked it.
fn swap_in_child(swap: Swap, victim: &Path, aside: &Path, other: &Path) -> Result<(), String> {
    let output = Command::new(env!("CARGO_BIN_EXE_pilot"))
        .arg("p1-swap")
        .arg(swap.name())
        .args([victim, aside, other])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(output.status.success(), "p1-swap failed: {stdout}");
    match stdout.strip_prefix("blocked ") {
        None if stdout == "swapped" => Ok(()),
        Some(reason) => Err(reason.to_string()),
        None => panic!("unexpected p1-swap output: {stdout}"),
    }
}

fn clone(layout: &Layout, held: &Held, mode: Mode) -> Output {
    let mut command = git(layout.scratch.path());
    command.args(["clone", "-q", "--", &layout.url]);
    match mode {
        Mode::Path => {
            command.arg(layout.stage());
        }
        Mode::Held => held_clone(&mut command, layout, held),
    }
    command.output().unwrap()
}

#[cfg(unix)]
fn held_clone(command: &mut Command, _layout: &Layout, held: &Held) {
    use std::os::fd::{AsRawFd, BorrowedFd};
    use std::os::unix::process::CommandExt;
    let fd = held.stage().fd().as_raw_fd();
    command.arg(".");
    // SAFETY: fchdir is async-signal-safe; the fd stays open in the parent
    // until the child has exec'd (the command runs to completion below).
    unsafe {
        command.pre_exec(move || {
            rustix::process::fchdir(BorrowedFd::borrow_raw(fd)).map_err(Into::into)
        });
    }
}

#[cfg(windows)]
fn held_clone(command: &mut Command, layout: &Layout, _held: &Held) {
    // The handles in `held` block renames of every level while git runs.
    command.arg(layout.stage());
}

/// Whether the clone's files are all in `dir`.
fn has_clone(dir: &Path) -> bool {
    dir.join(".git").is_dir()
        && FILES
            .iter()
            .all(|(rel, content)| fs::read(dir.join(rel)).ok().as_deref() == Some(*content))
}

fn run_case(mode: Mode, level: Level, swap: Swap) {
    let label = format!("p7-{}-{}-{}", mode.name(), level.name(), swap.name());
    let layout = Layout::new(&label, level);
    let held = Held::open(&layout.home);
    let outside_before = tree(&layout.outside);
    let alt_before = tree(&layout.alt);

    let swapped = swap_in_child(swap, &layout.victim, &layout.aside, &layout.other(swap));
    let output = clone(&layout, &held, mode);
    let reached = held.still_reached(&layout.home);

    // The held staging directory, wherever it is now.
    let held_stage = match &swapped {
        Ok(()) => layout.moved_to(swap).join(&layout.stage_in_victim),
        Err(_) => layout.stage(),
    };
    // Dir-in moves the replacement into the victim's place.
    let replacement = match (&swapped, swap) {
        (Ok(()), Swap::DirIn) => layout.victim.clone(),
        _ => layout.alt.clone(),
    };
    let outside_changed = tree(&layout.outside) != outside_before;
    let alt_changed = tree(&replacement) != alt_before;
    report(
        "P7",
        &[
            ("mode", &mode.name()),
            ("level", &level.name()),
            ("swap", &swap.name()),
            (
                "injection",
                &match &swapped {
                    Ok(()) => "swapped".to_string(),
                    Err(e) => format!("blocked({})", e.replace(' ', "_")),
                },
            ),
            ("git_status", &output.status.code().unwrap_or(-1)),
            ("clone_in_held_dir", &has_clone(&held_stage)),
            ("outside_home_changed", &outside_changed),
            ("replacement_changed", &alt_changed),
            ("recheck_reaches_held_dir", &reached),
        ],
    );
    drop(held);

    match (mode, &swapped) {
        // The platform blocked the swap (Windows): the clone completes in
        // place and nothing else changes.
        (_, Err(e)) => {
            if cfg!(unix) {
                panic!("{label}: swap blocked on Unix: {e}");
            }
            check(output, &label);
            assert!(has_clone(&layout.stage()), "{label}: clone missing");
            assert!(!outside_changed && !alt_changed, "{label}: wrote elsewhere");
            assert!(reached, "{label}: recheck failed without a swap");
        }
        (_, Ok(())) => {
            if cfg!(windows) {
                panic!("{label}: Windows let the swap through");
            }
            assert!(!reached, "{label}: recheck missed the swap");
            if mode == Mode::Held {
                check(output, &label);
                assert!(has_clone(&held_stage), "{label}: clone not in held dir");
                assert!(!alt_changed, "{label}: wrote into the replacement");
                // Move-out takes the held directory itself outside home.
                assert_eq!(outside_changed, swap == Swap::MoveOut, "{label}");
            }
        }
    }
}

#[test]
fn p7_platform() {
    let git = Command::new("git").arg("--version").output().unwrap();
    report(
        "P7-env",
        &[
            ("identity", &identity().unwrap()),
            (
                "git",
                &String::from_utf8_lossy(&git.stdout)
                    .trim()
                    .replace(' ', "_"),
            ),
        ],
    );
}

#[test]
fn p7_path_mode() {
    for level in [Level::Parent, Level::Ancestor] {
        run_case(Mode::Path, level, Swap::LinkOut);
    }
}

#[test]
fn p7_held_mode() {
    for level in [Level::Parent, Level::Ancestor] {
        for swap in [Swap::LinkOut, Swap::DirIn, Swap::MoveOut] {
            run_case(Mode::Held, level, swap);
        }
    }
}
