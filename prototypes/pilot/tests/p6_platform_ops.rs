//! P6: the platform operations 05 §6 added (ASM-16–ASM-21).
//!
//! - ASM-16: creating a file or renaming a directory never replaces what is
//!   already at the new name.
//! - ASM-17 (Unix): exchanging two names is one step, under concurrent
//!   readers and under kill; replace and delete built on it find a file
//!   swapped in or changed in place after the last check and put it back.
//! - ASM-18 (Windows): while the target is held for its last check, other
//!   processes cannot rename, delete, write or replace it.
//! - ASM-19 (Windows): a directory created with its own DACL passes it on to
//!   new files; copying a file's DACL to the temporary file keeps it through
//!   the replacement.
//! - ASM-20: the lock keeps a second process out and is released when the
//!   holder is killed.
//! - ASM-21 (Unix): opening a FIFO with `O_NONBLOCK` returns at once.
//!
//! "External" changes happen in this process at the sync point unless the
//! assumption is about other processes (ASM-18, ASM-20); then a child
//! `pilot` process makes them.

use std::ffi::OsStr;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};

use rosamta_pilot::atomic;
use rosamta_pilot::bound::{Dir, Kind};
use rosamta_pilot::ops::{self, Done, Observed};
use rosamta_pilot::platform::{ScratchDir, identity, report};

const OLD: &[u8] = b"old content\n";
const NEW: &[u8] = b"new content\n";
const EXTERNAL: &[u8] = b"external content\n";

fn os(s: &str) -> &OsStr {
    OsStr::new(s)
}

fn pilot<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(env!("CARGO_BIN_EXE_pilot"));
    command.args(args);
    command
}

/// Runs a child `pilot` to completion and returns its stdout.
fn run<I, S>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let out = pilot(args).output().unwrap();
    assert!(
        out.status.success(),
        "pilot failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// Starts a child `pilot` and waits for its first line.
fn spawn_ready<I, S>(args: I, expect: &str) -> Child
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = pilot(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.as_mut().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert_eq!(line.trim(), expect, "child did not start");
    child
}

fn observe(dir: &Dir, root: &Path, name: &str) -> Observed {
    let entry = dir.entry(os(name)).unwrap().unwrap();
    assert_eq!(entry.kind, Kind::File);
    Observed {
        identity: entry.identity,
        content: fs::read(root.join(name)).unwrap(),
    }
}

fn kind_of(dir: &Dir, name: &str) -> Option<Kind> {
    dir.entry(os(name)).unwrap().map(|e| e.kind)
}

fn no_temps(root: &Path) {
    let left = atomic::leftover_temps(root).unwrap();
    assert!(left.is_empty(), "temporary files left: {left:?}");
}

// ---------------------------------------------------------------- ASM-16

/// ASM-16: the final step of "建立" fails when the name is taken, whatever
/// takes it, and leaves the occupant alone.
#[test]
fn p6_create_does_not_replace() {
    report("P6", &[("identity", &identity().unwrap())]);
    let scratch = ScratchDir::new("p6-create").unwrap();
    let root = scratch.path();
    let dir = Dir::open_root(root).unwrap();

    let occupants: &[&str] = if cfg!(unix) {
        &["file", "dir", "link"]
    } else {
        &["file", "dir"]
    };
    for &occupant in occupants {
        let name = format!("taken-{occupant}");
        let path = root.join(&name);
        match occupant {
            "file" => fs::write(&path, EXTERNAL).unwrap(),
            "dir" => {
                fs::create_dir(&path).unwrap();
                fs::write(path.join("inner"), EXTERNAL).unwrap();
            }
            #[cfg(unix)]
            "link" => std::os::unix::fs::symlink("nowhere", &path).unwrap(),
            _ => unreachable!(),
        }
        let before = dir.entry(os(&name)).unwrap().unwrap();
        let temp = ops::temp_name(os(&name));
        ops::write_temp(&dir, &temp, NEW).unwrap();

        let err = ops::create_no_replace(&dir, &temp, os(&name)).unwrap_err();
        let after = dir.entry(os(&name)).unwrap().unwrap();
        assert_eq!(after, before, "{occupant}: occupant changed");
        match occupant {
            "file" => assert_eq!(fs::read(&path).unwrap(), EXTERNAL),
            "dir" => assert_eq!(fs::read(path.join("inner")).unwrap(), EXTERNAL),
            _ => assert_eq!(fs::read_link(&path).unwrap(), Path::new("nowhere")),
        }
        assert_eq!(
            fs::read(root.join(&temp)).unwrap(),
            NEW,
            "{occupant}: temp lost"
        );
        fs::remove_file(root.join(&temp)).unwrap();
        report(
            "P6",
            &[
                ("case", &"create-no-replace"),
                ("occupant", &occupant),
                ("result", &"refused"),
                ("error", &ops::error_name(&err)),
            ],
        );
    }

    let temp = ops::temp_name(os("free"));
    ops::write_temp(&dir, &temp, NEW).unwrap();
    ops::create_no_replace(&dir, &temp, os("free")).unwrap();
    assert_eq!(fs::read(root.join("free")).unwrap(), NEW);
    no_temps(root);
    report(
        "P6",
        &[
            ("case", &"create-no-replace"),
            ("occupant", &"none"),
            ("result", &"created"),
        ],
    );
}

/// ASM-16: initialization's directory rename fails when the name is taken,
/// even by an empty directory, which a plain rename would replace.
#[test]
fn p6_rename_dir_does_not_replace() {
    let scratch = ScratchDir::new("p6-rename-dir").unwrap();
    let root = scratch.path();
    let dir = Dir::open_root(root).unwrap();
    let source = root.join("clone");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("manifest"), NEW).unwrap();

    for occupant in ["empty-dir", "dir", "file"] {
        let name = format!("taken-{occupant}");
        let path = root.join(&name);
        match occupant {
            "empty-dir" => fs::create_dir(&path).unwrap(),
            "dir" => {
                fs::create_dir(&path).unwrap();
                fs::write(path.join("inner"), EXTERNAL).unwrap();
            }
            _ => fs::write(&path, EXTERNAL).unwrap(),
        }
        let before = dir.entry(os(&name)).unwrap().unwrap();
        let err = ops::rename_no_replace(&dir, os("clone"), os(&name)).unwrap_err();
        assert_eq!(dir.entry(os(&name)).unwrap().unwrap(), before, "{occupant}");
        assert_eq!(fs::read(source.join("manifest")).unwrap(), NEW);
        report(
            "P6",
            &[
                ("case", &"rename-dir-no-replace"),
                ("occupant", &occupant),
                ("result", &"refused"),
                ("error", &ops::error_name(&err)),
            ],
        );
    }

    ops::rename_no_replace(&dir, os("clone"), os("source")).unwrap();
    assert_eq!(fs::read(root.join("source").join("manifest")).unwrap(), NEW);
    report(
        "P6",
        &[
            ("case", &"rename-dir-no-replace"),
            ("occupant", &"none"),
            ("result", &"renamed"),
        ],
    );

    // Negative control: a plain rename does replace an empty directory.
    #[cfg(unix)]
    {
        fs::create_dir(root.join("victim")).unwrap();
        ops::rename(&dir, os("source"), os("victim")).unwrap();
        assert!(root.join("victim").join("manifest").exists());
        report(
            "P6",
            &[
                ("case", &"rename-dir-plain-control"),
                ("result", &"replaced-empty-dir"),
            ],
        );
    }
}

// ---------------------------------------------------------------- ASM-17

/// ASM-17: one exchange swaps the two names, including when the target is
/// a symbolic link (the link itself moves, the file it points to does not).
#[cfg(unix)]
#[test]
fn p6_exchange_swaps_names() {
    let scratch = ScratchDir::new("p6-exchange").unwrap();
    let root = scratch.path();
    let dir = Dir::open_root(root).unwrap();

    fs::write(root.join("target"), OLD).unwrap();
    let old = dir.entry(os("target")).unwrap().unwrap();
    let new = ops::write_temp(&dir, os("temp"), NEW).unwrap();
    ops::exchange(&dir, os("temp"), os("target")).unwrap();
    assert_eq!(
        dir.entry(os("target")).unwrap().unwrap().identity,
        new.identity
    );
    assert_eq!(
        dir.entry(os("temp")).unwrap().unwrap().identity,
        old.identity
    );
    assert_eq!(fs::read(root.join("target")).unwrap(), NEW);
    assert_eq!(fs::read(root.join("temp")).unwrap(), OLD);
    report("P6", &[("case", &"exchange-file"), ("result", &"swapped")]);

    fs::write(root.join("pointee"), EXTERNAL).unwrap();
    fs::remove_file(root.join("target")).unwrap();
    std::os::unix::fs::symlink("pointee", root.join("target")).unwrap();
    let link = dir.entry(os("target")).unwrap().unwrap();
    ops::exchange(&dir, os("temp"), os("target")).unwrap();
    assert_eq!(dir.entry(os("temp")).unwrap().unwrap(), link);
    assert_eq!(
        fs::read_link(root.join("temp")).unwrap(),
        Path::new("pointee")
    );
    assert_eq!(fs::read(root.join("pointee")).unwrap(), EXTERNAL);
    assert_eq!(kind_of(&dir, "target"), Some(Kind::File));
    report(
        "P6",
        &[("case", &"exchange-link"), ("result", &"link-moved")],
    );

    let err = ops::exchange(&dir, os("target"), os("absent")).unwrap_err();
    assert_eq!(kind_of(&dir, "target"), Some(Kind::File));
    report(
        "P6",
        &[
            ("case", &"exchange-missing"),
            ("result", &"refused"),
            ("error", &ops::error_name(&err)),
        ],
    );
}

/// ASM-17: while another process exchanges two names in a loop, a reader
/// always finds both names, each a complete file; after a kill, both names
/// still hold the two complete files.
#[cfg(unix)]
#[test]
fn p6_exchange_is_atomic() {
    use rosamta_pilot::payload::{self, Rng};

    const SIZE: usize = 64 * 1024;
    const ROUNDS: usize = 50;
    const READS: usize = 200;
    let scratch = ScratchDir::new("p6-exchange-atomic").unwrap();
    let root = scratch.path();
    let a = payload::build(b'A', SIZE);
    let b = payload::build(b'B', SIZE);
    let mut rng = Rng::new(20261110);
    let (mut seen_a, mut seen_b, mut missing, mut torn, mut broken_after_kill) = (0, 0, 0, 0, 0);

    for _ in 0..ROUNDS {
        fs::write(root.join("a"), &a).unwrap();
        fs::write(root.join("b"), &b).unwrap();
        let mut child = spawn_ready(
            [os("p6-exchange"), root.as_os_str(), os("a"), os("b")],
            "ready",
        );
        for i in 0..READS {
            let name = if i % 2 == 0 { "a" } else { "b" };
            match fs::read(root.join(name)) {
                Ok(got) if got == a => seen_a += 1,
                Ok(got) if got == b => seen_b += 1,
                Ok(_) => torn += 1,
                Err(_) => missing += 1,
            }
        }
        std::thread::sleep(std::time::Duration::from_micros(rng.below(5_000)));
        child.kill().unwrap();
        child.wait().unwrap();
        let after = (fs::read(root.join("a")), fs::read(root.join("b")));
        match after {
            (Ok(x), Ok(y)) if (x == a && y == b) || (x == b && y == a) => {}
            _ => broken_after_kill += 1,
        }
    }
    report(
        "P6",
        &[
            ("case", &"exchange-concurrent"),
            ("rounds", &ROUNDS),
            ("reads", &(ROUNDS * READS)),
            ("a", &seen_a),
            ("b", &seen_b),
            ("missing", &missing),
            ("torn", &torn),
            ("broken_after_kill", &broken_after_kill),
        ],
    );
    assert_eq!((missing, torn, broken_after_kill), (0, 0, 0));
}

/// What happens to the target at the `BeforeOperation` sync point.
#[cfg(unix)]
#[derive(Clone, Copy, Debug)]
enum Interference {
    Nothing,
    ReplacedByFile,
    ModifiedInPlace,
    ReplacedByLink,
    ReplacedByDir,
}

#[cfg(unix)]
impl Interference {
    fn all() -> Vec<Interference> {
        vec![
            Interference::Nothing,
            Interference::ReplacedByFile,
            Interference::ModifiedInPlace,
            Interference::ReplacedByLink,
            Interference::ReplacedByDir,
        ]
    }

    fn name(self) -> &'static str {
        match self {
            Interference::Nothing => "none",
            Interference::ReplacedByFile => "replaced-by-file",
            Interference::ModifiedInPlace => "modified-in-place",
            Interference::ReplacedByLink => "replaced-by-link",
            Interference::ReplacedByDir => "replaced-by-dir",
        }
    }

    /// Makes the change at the target, from this process.
    fn apply(self, root: &Path) {
        let target = root.join("target");
        let staged = root.join("staged");
        match self {
            Interference::Nothing => {}
            Interference::ReplacedByFile => {
                fs::write(&staged, EXTERNAL).unwrap();
                fs::rename(&staged, &target).unwrap();
            }
            Interference::ModifiedInPlace => fs::write(&target, EXTERNAL).unwrap(),
            Interference::ReplacedByLink => {
                fs::write(root.join("pointee"), EXTERNAL).unwrap();
                std::os::unix::fs::symlink("pointee", &staged).unwrap();
                fs::rename(&staged, &target).unwrap();
            }
            Interference::ReplacedByDir => {
                fs::remove_file(&target).unwrap();
                fs::create_dir(&target).unwrap();
                fs::write(target.join("inner"), EXTERNAL).unwrap();
            }
        }
    }

    /// The external change is still at the target, untouched.
    fn assert_kept(self, dir: &Dir, root: &Path) {
        let target = root.join("target");
        match self {
            Interference::Nothing => unreachable!(),
            Interference::ReplacedByFile | Interference::ModifiedInPlace => {
                assert_eq!(fs::read(&target).unwrap(), EXTERNAL)
            }
            Interference::ReplacedByLink => {
                assert_eq!(kind_of(dir, "target"), Some(Kind::Link));
                assert_eq!(fs::read(root.join("pointee")).unwrap(), EXTERNAL);
            }
            Interference::ReplacedByDir => {
                assert_eq!(fs::read(target.join("inner")).unwrap(), EXTERNAL)
            }
        }
    }
}

/// 05 §6.3 "取代" on Unix: a change made after the last check is found
/// after the exchange and put back; otherwise the replacement goes through.
#[cfg(unix)]
#[test]
fn p6_replace_puts_back_late_changes() {
    for interference in Interference::all() {
        let scratch = ScratchDir::new("p6-replace").unwrap();
        let root = scratch.path();
        let dir = Dir::open_root(root).unwrap();
        fs::write(root.join("target"), OLD).unwrap();
        let observed = observe(&dir, root, "target");

        let done = ops::replace(&dir, os("target"), NEW, &observed, &mut || {
            interference.apply(root)
        })
        .unwrap();
        match interference {
            Interference::Nothing => {
                assert_eq!(done, Done::Applied);
                assert_eq!(fs::read(root.join("target")).unwrap(), NEW);
            }
            _ => {
                assert_eq!(done, Done::PutBack, "{interference:?}");
                interference.assert_kept(&dir, root);
            }
        }
        no_temps(root);
        report(
            "P6",
            &[
                ("case", &"replace-exchange"),
                ("interference", &interference.name()),
                ("result", &format!("{done:?}")),
            ],
        );
    }
}

/// 05 §6.3 "刪除" on Unix: the target is moved aside and checked there; a
/// change made after the last check is moved back without replacing.
#[cfg(unix)]
#[test]
fn p6_remove_puts_back_late_changes() {
    for interference in Interference::all() {
        let scratch = ScratchDir::new("p6-remove").unwrap();
        let root = scratch.path();
        let dir = Dir::open_root(root).unwrap();
        fs::write(root.join("target"), OLD).unwrap();
        let observed = observe(&dir, root, "target");

        let done = ops::remove(&dir, os("target"), &observed, &mut || {
            interference.apply(root)
        })
        .unwrap();
        match interference {
            Interference::Nothing => {
                assert_eq!(done, Done::Applied);
                assert_eq!(kind_of(&dir, "target"), None);
            }
            _ => {
                assert_eq!(done, Done::PutBack, "{interference:?}");
                interference.assert_kept(&dir, root);
            }
        }
        no_temps(root);
        report(
            "P6",
            &[
                ("case", &"remove-aside"),
                ("interference", &interference.name()),
                ("result", &format!("{done:?}")),
            ],
        );
    }
}

// ---------------------------------------------------------------- ASM-18

/// Asks a child process to do `op` to `target`; returns its answer.
#[cfg(windows)]
fn external(op: &str, root: &Path) -> String {
    let other = match op {
        "rename" => root.join("moved"),
        "replace" => {
            let staged = root.join("staged");
            fs::write(&staged, EXTERNAL).unwrap();
            staged
        }
        _ => root.join("unused"),
    };
    run([
        os("p6-external"),
        os(op),
        root.join("target").as_os_str(),
        other.as_os_str(),
    ])
}

#[cfg(windows)]
const EXTERNAL_OPS: [&str; 4] = ["rename", "delete", "write", "replace"];

/// ASM-18: while "取代" and "刪除" hold the target for the last check,
/// another process cannot rename, delete, write or replace it, but can
/// still read it. The operation then goes through on the checked file.
#[cfg(windows)]
#[test]
fn p6_held_target_blocks_other_processes() {
    for operation in ["replace", "remove"] {
        let scratch = ScratchDir::new("p6-held").unwrap();
        let root = scratch.path();
        let dir = Dir::open_root(root).unwrap();
        fs::write(root.join("target"), OLD).unwrap();
        let observed = observe(&dir, root, "target");

        let mut answers = Vec::new();
        let mut hook = || {
            for op in EXTERNAL_OPS.iter().chain(["read"].iter()) {
                answers.push((*op, external(op, root)));
            }
        };
        let done = match operation {
            "replace" => ops::replace(&dir, os("target"), NEW, &observed, &mut hook),
            _ => ops::remove(&dir, os("target"), &observed, &mut hook),
        }
        .unwrap();
        assert_eq!(done, Done::Applied);
        for (op, answer) in &answers {
            report(
                "P6",
                &[
                    ("case", &"held-target"),
                    ("operation", &operation),
                    ("external", op),
                    ("result", answer),
                ],
            );
        }
        for (op, answer) in &answers {
            if *op == "read" {
                assert_eq!(answer, "ok", "{operation}: read");
            } else {
                assert!(
                    answer.starts_with("blocked"),
                    "{operation}: {op} was {answer}"
                );
            }
        }
        match operation {
            "replace" => assert_eq!(fs::read(root.join("target")).unwrap(), NEW),
            _ => assert_eq!(kind_of(&dir, "target"), None),
        }
        assert!(!root.join("moved").exists());
    }
}

/// Negative control for ASM-18: a handle with attribute access only does
/// not take part in sharing checks (the ASM-08 finding, for files).
/// Reported, not asserted.
#[cfg(windows)]
#[test]
fn p6_attribute_handle_control() {
    let scratch = ScratchDir::new("p6-held-control").unwrap();
    let root = scratch.path();
    let dir = Dir::open_root(root).unwrap();
    for op in EXTERNAL_OPS {
        fs::write(root.join("target"), OLD).unwrap();
        let _ = fs::remove_file(root.join("moved"));
        let held = ops::hold_attributes_only(&dir, os("target")).unwrap();
        let answer = external(op, root);
        drop(held);
        report(
            "P6",
            &[
                ("case", &"attribute-handle-control"),
                ("external", &op),
                ("result", &answer),
            ],
        );
    }
}

// ---------------------------------------------------------------- ASM-19

/// The trustees and flags of each ACE in an SDDL DACL string.
#[cfg(windows)]
fn aces(sddl: &str) -> Vec<(String, String)> {
    sddl.split('(')
        .skip(1)
        .map(|ace| {
            let fields: Vec<&str> = ace.trim_end_matches(')').split(';').collect();
            (fields[1].to_string(), fields[5].to_string())
        })
        .collect()
}

/// ASM-19: a directory created with a protected DACL has exactly that DACL,
/// and a file created in it inherits it.
#[cfg(windows)]
#[test]
fn p6_private_dir_dacl() {
    let sid = ops::user_sid().unwrap();
    let sddl = format!("D:P(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)");
    let scratch = ScratchDir::new("p6-dacl-dir").unwrap();
    let private = scratch.path().join("private");
    ops::create_dir_with_dacl(&private, &sddl).unwrap();
    fs::write(private.join("file"), NEW).unwrap();

    let dir_sddl = ops::dacl_sddl(&private).unwrap();
    let file_sddl = ops::dacl_sddl(&private.join("file")).unwrap();
    report(
        "P6",
        &[
            ("case", &"private-dir"),
            ("dir", &dir_sddl),
            ("file", &file_sddl),
        ],
    );

    let trustees = |s: &str| {
        let mut t: Vec<String> = aces(s).into_iter().map(|(_, who)| who).collect();
        t.sort();
        t
    };
    let mut expected = vec![sid.clone(), "SY".to_string(), "BA".to_string()];
    expected.sort();
    assert!(
        dir_sddl.starts_with("D:P"),
        "directory DACL not protected: {dir_sddl}"
    );
    assert_eq!(trustees(&dir_sddl), expected);
    assert_eq!(trustees(&file_sddl), expected);
    assert!(
        aces(&file_sddl)
            .iter()
            .all(|(flags, _)| flags.contains("ID")),
        "file ACEs not all inherited: {file_sddl}"
    );
}

/// ASM-19: copying the original's DACL to the temporary file keeps it
/// through `MoveFileExW`; without the copy the target gets the inherited
/// one (ASM-02, reported only).
#[cfg(windows)]
#[test]
fn p6_replace_keeps_copied_dacl() {
    let sid = ops::user_sid().unwrap();
    let cases = [
        ("protected", format!("D:P(A;;FA;;;{sid})(A;;FR;;;BU)")),
        ("explicit-ace", "D:(A;;FR;;;BU)".to_string()),
    ];
    for (case, sddl) in cases {
        for copy in [true, false] {
            let scratch = ScratchDir::new("p6-dacl-copy").unwrap();
            let root = scratch.path();
            let dir = Dir::open_root(root).unwrap();
            fs::write(root.join("target"), OLD).unwrap();
            ops::set_dacl(&root.join("target"), &sddl).unwrap();
            let before = ops::dacl_sddl(&root.join("target")).unwrap();

            let temp = ops::temp_name(os("target"));
            ops::write_temp(&dir, &temp, NEW).unwrap();
            if copy {
                ops::copy_dacl(&root.join("target"), &root.join(&temp)).unwrap();
            }
            ops::move_replace(&dir, &temp, os("target")).unwrap();
            let after = ops::dacl_sddl(&root.join("target")).unwrap();
            assert_eq!(fs::read(root.join("target")).unwrap(), NEW);
            report(
                "P6",
                &[
                    ("case", &"replace-dacl"),
                    ("dacl", &case),
                    ("copy", &copy),
                    ("same", &(after == before)),
                    ("before", &before),
                    ("after", &after),
                ],
            );
            if copy {
                assert_eq!(after, before, "{case}: DACL not kept");
            }
        }
    }
}

// ---------------------------------------------------------------- ASM-20

/// ASM-20: the lock keeps a second process out; it is released when the
/// holder lets go or is killed, and the lock file stays.
#[test]
fn p6_lock_excludes_other_processes() {
    let scratch = ScratchDir::new("p6-lock").unwrap();
    let root = scratch.path();
    let dir = Dir::open_root(root).unwrap();
    let try_lock = || run([os("p6-lock"), os("try"), root.as_os_str(), os("lock")]);

    let held = ops::lock(&dir, os("lock")).unwrap();
    let busy = try_lock();
    assert!(busy.starts_with("busy"), "second process got {busy}");
    report("P6", &[("case", &"lock-held"), ("other", &busy)]);

    // flock belongs to the open file description: opening and closing the
    // same file again in this process must not release it (fcntl locks
    // would be released here; 05 §6.7).
    #[cfg(unix)]
    {
        drop(fs::File::open(root.join("lock")).unwrap());
        let still = try_lock();
        assert!(
            still.starts_with("busy"),
            "after closing another fd: {still}"
        );
        report(
            "P6",
            &[("case", &"lock-other-fd-closed"), ("other", &still)],
        );
    }

    drop(held);
    assert_eq!(try_lock(), "acquired");
    report("P6", &[("case", &"lock-released"), ("other", &"acquired")]);

    let mut holder = spawn_ready(
        [os("p6-lock"), os("hold"), root.as_os_str(), os("lock")],
        "held",
    );
    let err = ops::lock(&dir, os("lock")).err().expect("lock taken twice");
    assert!(ops::is_busy(&err), "unexpected error {err}");
    holder.kill().unwrap();
    holder.wait().unwrap();
    // Windows closes the handles of a killed process shortly after it ends.
    let mut attempts = 0;
    let relocked = loop {
        attempts += 1;
        match ops::lock(&dir, os("lock")) {
            Ok(lock) => break lock,
            Err(e) if ops::is_busy(&e) && attempts < 50 => {
                std::thread::sleep(std::time::Duration::from_millis(20))
            }
            Err(e) => panic!("lock not released after kill: {e}"),
        }
    };
    drop(relocked);
    assert!(root.join("lock").exists());
    report(
        "P6",
        &[
            ("case", &"lock-holder-killed"),
            ("before_kill", &ops::error_name(&err)),
            ("attempts_after_kill", &attempts),
        ],
    );
}

// ---------------------------------------------------------------- ASM-21

/// ASM-21: a FIFO opened with `O_NONBLOCK` returns at once and is reported
/// as another kind, so it is not read. Without the flag the open blocks
/// (negative control).
#[cfg(unix)]
#[test]
fn p6_fifo_open_does_not_block() {
    use std::sync::mpsc;
    use std::time::Duration;

    let scratch = ScratchDir::new("p6-fifo").unwrap();
    let root = scratch.path().to_path_buf();
    let status = Command::new("mkfifo")
        .arg(root.join("fifo"))
        .status()
        .unwrap();
    assert!(status.success());

    let (tx, rx) = mpsc::channel();
    let r = root.clone();
    std::thread::spawn(move || {
        let dir = Dir::open_root(&r).unwrap();
        let _ = tx.send(ops::open_nonblocking(&dir, os("fifo")).map(|(e, _)| e.kind));
    });
    let kind = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("O_NONBLOCK open blocked")
        .unwrap();
    assert_eq!(kind, Kind::Other);
    report(
        "P6",
        &[
            ("case", &"fifo-nonblock"),
            ("result", &"returned"),
            ("kind", &"other"),
        ],
    );

    let (tx, rx) = mpsc::channel();
    let r = root.clone();
    let blocked = std::thread::spawn(move || {
        let _ = tx.send(fs::File::open(r.join("fifo")).map(drop));
    });
    let control = match rx.recv_timeout(Duration::from_millis(500)) {
        Err(mpsc::RecvTimeoutError::Timeout) => "blocked",
        _ => "returned",
    };
    if control == "blocked" {
        // Open the write end to release the blocked reader.
        drop(
            fs::OpenOptions::new()
                .write(true)
                .open(root.join("fifo"))
                .unwrap(),
        );
    }
    blocked.join().unwrap();
    report(
        "P6",
        &[("case", &"fifo-blocking-control"), ("result", &control)],
    );
}
