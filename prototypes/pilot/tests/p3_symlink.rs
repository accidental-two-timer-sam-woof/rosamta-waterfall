//! P3: a symbolic link at a target path is taken over and restored without
//! touching the file it points to, and without creating a new link
//! (SEC-002, DC-001, DC-003; 04 §3.3, §5 A3, A6).
//!
//! Layout of each case, under a scratch root:
//!
//! ```text
//! home/a/target       the symbolic link (relative)
//! home/inside/target  a file inside home the link may point to
//! home/backup/        the backup directory (0700 on Unix)
//! outside/target      a file outside home the link may point to
//! ```
//!
//! Test fixtures need symbolic links. On Windows without developer mode a
//! standard user cannot create them, so CI creates a pool of links as the
//! administrator beforehand (`ci/make-link-pool.ps1`), and the tests move
//! one into place. Moving a link needs no privilege.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rosamta_pilot::atomic::TEMP_MARKER;
use rosamta_pilot::bound::{Dir, Entry, Kind, Preflight};
use rosamta_pilot::link;
use rosamta_pilot::platform::{ScratchDir, identity, report};

const INSIDE: &[u8] = b"inside home\n";
const OUTSIDE: &[u8] = b"outside home\n";
const NEW: &[u8] = b"new content\n";

#[derive(Clone, Copy, Debug)]
enum Pointee {
    Outside,
    Inside,
}

impl Pointee {
    fn name(self) -> &'static str {
        match self {
            Pointee::Outside => "outside",
            Pointee::Inside => "inside",
        }
    }

    /// The link's target, relative to `home/a`.
    fn link_target(self) -> PathBuf {
        match self {
            Pointee::Outside => Path::new("..").join("..").join("outside").join("target"),
            Pointee::Inside => Path::new("..").join("inside").join("target"),
        }
    }
}

struct Layout {
    _scratch: ScratchDir,
    root: PathBuf,
    home: PathBuf,
    pointee: PathBuf,
    /// How the fixture link was made: `created` or `pool`.
    fixture: &'static str,
}

impl Layout {
    fn new(label: &str, pointee: Pointee) -> Layout {
        let scratch = ScratchDir::new(label).unwrap();
        let root = scratch.path().to_path_buf();
        let home = root.join("home");
        fs::create_dir_all(home.join("a")).unwrap();
        fs::create_dir_all(home.join("inside")).unwrap();
        fs::create_dir_all(root.join("outside")).unwrap();
        fs::write(home.join("inside").join("target"), INSIDE).unwrap();
        fs::write(root.join("outside").join("target"), OUTSIDE).unwrap();
        make_backup_dir(&home.join("backup"));
        let fixture = make_link(&home.join("a").join("target"), pointee);
        let pointee = match pointee {
            Pointee::Outside => root.join("outside").join("target"),
            Pointee::Inside => home.join("inside").join("target"),
        };
        Layout {
            _scratch: scratch,
            root,
            home,
            pointee,
            fixture,
        }
    }

    fn open(&self) -> (Dir, Dir) {
        let home = Dir::open_root(&self.home).unwrap();
        let a = home.open_child(OsStr::new("a")).unwrap();
        let backup = home.open_child(OsStr::new("backup")).unwrap();
        (a, backup)
    }
}

#[cfg(unix)]
fn make_backup_dir(path: &Path) {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(path).unwrap();
}

#[cfg(windows)]
fn make_backup_dir(path: &Path) {
    fs::create_dir(path).unwrap();
}

/// Creates the fixture link, or on Windows without the privilege, moves
/// one in from the pool.
fn make_link(path: &Path, pointee: Pointee) -> &'static str {
    let target = pointee.link_target();
    let how = match link::create(path, &target) {
        Ok(()) => "created",
        #[cfg(windows)]
        Err(e) if e.raw_os_error() == Some(1314) => {
            take_from_pool(path, pointee);
            "pool"
        }
        Err(e) => panic!("cannot create the fixture link: {e}"),
    };
    assert_eq!(fs::read_link(path).unwrap(), target, "fixture link target");
    how
}

#[cfg(windows)]
fn take_from_pool(path: &Path, pointee: Pointee) {
    let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let pool = Path::new(&format!("{drive}\\"))
        .join("rosamta-pilot-links")
        .join(pointee.name());
    let mut entries: Vec<PathBuf> = fs::read_dir(&pool)
        .unwrap_or_else(|e| panic!("no link pool at {}: {e}", pool.display()))
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for entry in entries {
        if fs::rename(&entry, path).is_ok() {
            return;
        }
    }
    panic!("link pool {} is empty", pool.display());
}

/// What must not change about the file the link points to.
#[derive(Debug, PartialEq, Eq)]
struct Pointed {
    content: Vec<u8>,
    modified: SystemTime,
    len: u64,
}

fn pointed(path: &Path) -> Pointed {
    let meta = fs::symlink_metadata(path).unwrap();
    assert!(meta.is_file());
    Pointed {
        content: fs::read(path).unwrap(),
        modified: meta.modified().unwrap(),
        len: meta.len(),
    }
}

fn link_entry(dir: &Dir, name: &str) -> Entry {
    let entry = dir.entry(OsStr::new(name)).unwrap().expect("entry exists");
    assert_eq!(entry.kind, Kind::Link, "{name} is a link");
    entry
}

fn no_temps(root: &Path) -> bool {
    fn walk(dir: &Path) -> bool {
        fs::read_dir(dir).unwrap().all(|e| {
            let e = e.unwrap();
            let path = e.path();
            if path.to_string_lossy().contains(TEMP_MARKER) {
                return false;
            }
            let meta = fs::symlink_metadata(&path).unwrap();
            !meta.is_dir() || meta.file_type().is_symlink() || walk(&path)
        })
    }
    walk(root)
}

#[test]
fn p3_identity() {
    report(
        "ENV",
        &[
            ("os", &std::env::consts::OS),
            ("identity", &identity().unwrap()),
        ],
    );
}

/// SEC-002: the link is seen as a link, even when the file it points to
/// already holds the expected content, and nothing reads through it.
#[test]
fn p3_link_is_not_followed() {
    for pointee in [Pointee::Outside, Pointee::Inside] {
        let layout = Layout::new(&format!("p3-seen-{}", pointee.name()), pointee);
        let before = pointed(&layout.pointee);
        let (a, _) = layout.open();
        let entry = link_entry(&a, "target");

        let preflight = Preflight::take(&layout.home, &Path::new("a").join("target"));
        let read = a.read_file(OsStr::new("target"), entry.identity);
        report(
            "P3",
            &[
                ("case", &"seen"),
                ("pointee", &pointee.name()),
                ("fixture", &layout.fixture),
                ("kind", &format!("{:?}", entry.kind).to_lowercase()),
                ("preflight_refused", &preflight.is_err()),
                ("read_through_link", &read.is_ok()),
            ],
        );
        assert!(preflight.is_err(), "a link is not a regular file target");
        assert!(read.is_err(), "reading must not follow the link");
        assert_eq!(pointed(&layout.pointee), before);
    }
}

/// SEC-002, DC-003, 04 §3.3: takeover keeps the link itself as the backup,
/// leaves a regular file at the target path, and restore puts the very same
/// link back. The file it points to never changes.
#[test]
fn p3_takeover_and_restore() {
    for pointee in [Pointee::Outside, Pointee::Inside] {
        let label = format!("p3-takeover-{}", pointee.name());
        let layout = Layout::new(&label, pointee);
        let before = pointed(&layout.pointee);
        let (a, backup) = layout.open();
        let name = OsStr::new("target");
        let backup_name = OsStr::new("target.backup");
        let original = link_entry(&a, "target");
        let target_text = link::read_target(&a, name).unwrap();

        // Backup: the target path still holds the link.
        link::backup(&a, name, original.identity, &backup, backup_name).unwrap();
        assert_eq!(
            link_entry(&a, "target"),
            original,
            "{label}: target after backup"
        );
        let backed_up = link_entry(&backup, "target.backup");
        assert_eq!(
            backed_up.identity, original.identity,
            "{label}: backup is the link"
        );

        // Takeover: one rename replaces the link with a regular file.
        let written = a.write(name, NEW, None).unwrap();
        let now = a.entry(name).unwrap().unwrap();
        assert_eq!((now.kind, now.identity), (Kind::File, written), "{label}");
        let meta = fs::symlink_metadata(layout.home.join("a").join("target")).unwrap();
        assert!(
            meta.is_file() && !meta.file_type().is_symlink(),
            "{label}: DC-003"
        );
        assert_eq!(fs::read(layout.home.join("a").join("target")).unwrap(), NEW);
        assert_eq!(
            link_entry(&backup, "target.backup").identity,
            original.identity
        );
        assert_eq!(
            link::read_target(&backup, backup_name).unwrap(),
            target_text
        );
        let after_takeover = pointed(&layout.pointee);

        // Restore: the same link, under its old name.
        link::restore(&backup, backup_name, original.identity, &a, name, written).unwrap();
        let restored = link_entry(&a, "target");
        let restored_text = link::read_target(&a, name).unwrap();
        let backup_left = backup.entry(backup_name).unwrap();
        let after_restore = pointed(&layout.pointee);

        #[cfg(unix)]
        let backup_mode = {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(layout.home.join("backup"))
                .unwrap()
                .permissions()
                .mode();
            format!("{:o}", mode & 0o777)
        };
        #[cfg(windows)]
        let backup_mode = "n/a".to_string();

        report(
            "P3",
            &[
                ("case", &"takeover"),
                ("pointee", &pointee.name()),
                ("fixture", &layout.fixture),
                ("method", &"hardlink+rename"),
                (
                    "backup_same_identity",
                    &(backed_up.identity == original.identity),
                ),
                ("target_regular_file", &true),
                (
                    "restored_same_identity",
                    &(restored.identity == original.identity),
                ),
                ("restored_same_target", &(restored_text == target_text)),
                (
                    "pointee_unchanged",
                    &(after_takeover == before && after_restore == before),
                ),
                ("backup_dir_mode", &backup_mode),
            ],
        );
        assert_eq!(restored.identity, original.identity, "{label}: same link");
        assert_eq!(restored_text, target_text, "{label}: same link target");
        assert_eq!(backup_left, None, "{label}: backup consumed by restore");
        assert_eq!(after_takeover, before, "{label}: pointee after takeover");
        assert_eq!(after_restore, before, "{label}: pointee after restore");
        assert!(no_temps(&layout.root), "{label}: leftover temporary files");
    }
}

/// 04 §3.3: restore leaves alone a target that is no longer the file this
/// batch wrote, and keeps the backup.
#[test]
fn p3_restore_refuses_changed_target() {
    let layout = Layout::new("p3-restore-refused", Pointee::Outside);
    let before = pointed(&layout.pointee);
    let (a, backup) = layout.open();
    let name = OsStr::new("target");
    let backup_name = OsStr::new("target.backup");
    let original = link_entry(&a, "target");
    link::backup(&a, name, original.identity, &backup, backup_name).unwrap();
    let written = a.write(name, NEW, None).unwrap();

    // Another program replaces the file this batch wrote.
    let external = layout.home.join("a").join("external");
    fs::write(&external, b"external\n").unwrap();
    fs::rename(&external, layout.home.join("a").join("target")).unwrap();

    let result = link::restore(&backup, backup_name, original.identity, &a, name, written);
    let target_now = fs::read(layout.home.join("a").join("target")).unwrap();
    let backup_kept = backup.entry(backup_name).unwrap().map(|e| e.identity);
    report(
        "P3",
        &[
            ("case", &"restore-changed-target"),
            ("fixture", &layout.fixture),
            ("refused", &result.is_err()),
            ("external_kept", &(target_now == b"external\n")),
            ("backup_kept", &(backup_kept == Some(original.identity))),
        ],
    );
    assert!(result.is_err());
    assert_eq!(target_now, b"external\n");
    assert_eq!(backup_kept, Some(original.identity));
    assert_eq!(pointed(&layout.pointee), before);
}

/// Backup refuses when the target path no longer holds the planned link.
#[test]
fn p3_backup_refuses_changed_link() {
    let layout = Layout::new("p3-backup-refused", Pointee::Outside);
    let (a, backup) = layout.open();
    let name = OsStr::new("target");
    let original = link_entry(&a, "target");

    // Replace the link with a regular file that has the pointee's content.
    let target = layout.home.join("a").join("target");
    let other = layout.home.join("a").join("other");
    fs::write(&other, OUTSIDE).unwrap();
    fs::rename(&other, &target).unwrap();

    let result = link::backup(&a, name, original.identity, &backup, OsStr::new("b"));
    let backup_empty = fs::read_dir(layout.home.join("backup"))
        .unwrap()
        .next()
        .is_none();
    report(
        "P3",
        &[
            ("case", &"backup-changed-link"),
            ("fixture", &layout.fixture),
            ("refused", &result.is_err()),
            ("backup_empty", &backup_empty),
        ],
    );
    assert!(result.is_err());
    assert!(backup_empty);
    assert_eq!(fs::read(&target).unwrap(), OUTSIDE);
}

/// DC-001: whether this user could create a link, which a copy-based backup
/// or restore would need. Recorded, not asserted, on Windows.
#[test]
fn p3_create_link_privilege() {
    let scratch = ScratchDir::new("p3-create").unwrap();
    fs::write(scratch.path().join("file"), b"x").unwrap();
    let result = link::create(&scratch.path().join("link"), Path::new("file"));
    report(
        "P3",
        &[
            ("case", &"create-link"),
            (
                "result",
                &match &result {
                    Ok(()) => "ok".to_string(),
                    Err(e) => format!("denied({})", e.to_string().replace(' ', "_")),
                },
            ),
            ("developer_mode", &developer_mode()),
        ],
    );
    #[cfg(unix)]
    result.unwrap();
}

#[cfg(unix)]
fn developer_mode() -> String {
    "n/a".into()
}

#[cfg(windows)]
fn developer_mode() -> String {
    let out = std::process::Command::new("reg")
        .args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock",
            "/v",
            "AllowDevelopmentWithoutDevLicense",
        ])
        .output();
    match out {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .last()
            .unwrap_or("?")
            .to_string(),
        _ => "unset".into(),
    }
}

/// A hard link cannot cross file systems: the backup directory must be on
/// the target's. Runs where a second file system is at hand (Linux
/// `/dev/shm`); the failure must leave everything as it was.
#[cfg(unix)]
#[test]
fn p3_backup_across_file_systems() {
    use std::os::unix::fs::MetadataExt;
    let layout = Layout::new("p3-cross", Pointee::Outside);
    let here = fs::metadata(&layout.home).unwrap().dev();
    let Some(other) = ["/dev/shm"]
        .into_iter()
        .map(Path::new)
        .find(|p| fs::metadata(p).is_ok_and(|m| m.dev() != here))
    else {
        report("P3", &[("case", &"cross-fs"), ("result", &"skipped")]);
        return;
    };
    let foreign = other.join(format!("rosamta-pilot-p3-{}", std::process::id()));
    make_backup_dir(&foreign);
    let (a, _) = layout.open();
    let original = link_entry(&a, "target");
    let foreign_dir = Dir::open_root(&foreign).unwrap();
    let result = link::backup(
        &a,
        OsStr::new("target"),
        original.identity,
        &foreign_dir,
        OsStr::new("b"),
    );
    let untouched = link_entry(&a, "target") == original;
    let empty = fs::read_dir(&foreign).unwrap().next().is_none();
    let _ = fs::remove_dir_all(&foreign);
    report(
        "P3",
        &[
            ("case", &"cross-fs"),
            (
                "result",
                &match &result {
                    Ok(()) => "linked".to_string(),
                    Err(e) => format!("refused({})", e.to_string().replace(' ', "_")),
                },
            ),
            ("link_untouched", &untouched),
            ("backup_empty", &empty),
        ],
    );
    assert!(result.is_err() && untouched && empty);
}
