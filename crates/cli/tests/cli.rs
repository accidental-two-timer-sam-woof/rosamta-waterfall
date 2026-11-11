use std::{fs, process::Output};

use rosamta_testkit::{TempHome, command_in_home, temporary_home};
use serde_json::{Value, json};

fn run(home: &TempHome, args: &[&str]) -> Output {
    command_in_home(env!("CARGO_BIN_EXE_rosamta"), home)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn help_and_version_work_in_an_empty_home() {
    let home = temporary_home().unwrap();
    let commands: &[&[&str]] = &[
        &[],
        &["init"],
        &["diff"],
        &["status"],
        &["apply"],
        &["add"],
        &["modify"],
        &["forget"],
        &["list"],
        &["query"],
        &["query", "refs"],
        &["query", "output"],
        &["query", "value"],
        &["schema"],
        &["schema", "manifest"],
        &["schema", "module"],
        &["schema", "profile"],
        &["schema", "local"],
        &["schema", "builtin"],
    ];
    for command in commands {
        let mut args = command.to_vec();
        args.push("--help");
        let output = run(&home, &args);
        assert!(output.status.success(), "{args:?}: {:?}", output.stderr);
        assert!(output.stderr.is_empty());
        let stdout = String::from_utf8(output.stdout).unwrap();
        for heading in ["Usage:", "Exit status:", "Examples:"] {
            assert!(stdout.contains(heading), "{args:?}: {stdout}");
        }
    }
    let help = run(&home, &["help", "query", "refs"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("MODULE[.FIELD]"));
    let formatted_help = run(&home, &["--format", "json", "help", "query", "refs"]);
    assert_eq!(formatted_help.stdout, help.stdout);
    assert!(formatted_help.status.success());
    let version = run(&home, &["--version"]);
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")));
    assert_eq!(fs::read_dir(home.path()).unwrap().count(), 0);
}

#[test]
fn missing_configuration_reports_failure_without_writing_to_home() {
    let home = temporary_home().unwrap();
    let output = run(&home, &["apply"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error[preflight.failed]"));

    for args in [["--format", "json", "apply"], ["apply", "--format", "json"]] {
        let output = run(&home, &args);
        assert_eq!(output.status.code(), Some(1));
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            result,
            json!({"json_version": 1, "command": "apply", "exit_code": 1, "result": null})
        );
        let diagnostic: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(diagnostic["code"], "preflight.failed");
        assert_eq!(diagnostic["severity"], "error");
        assert!(output.stdout.ends_with(b"\n"));
        assert!(output.stderr.ends_with(b"\n"));
    }
    assert_eq!(fs::read_dir(home.path()).unwrap().count(), 0);
}

#[test]
fn invalid_arguments_fail_before_dispatch() {
    let home = temporary_home().unwrap();
    let cases: &[&[&str]] = &[
        &["init", "--no-apply"],
        &["init", "--strict"],
        &["init", "--set", "theme.bad.field=x"],
        &["apply", "--overwrite-all", "--accept-all"],
        &["modify", "ghostty"],
        &["modify", "ghostty", "--template", "a", "--copy", "b"],
        &[
            "modify",
            "ghostty",
            "--target",
            "linux=~/x",
            "--reset",
            "target.linux",
        ],
        &["query", "value", "theme"],
    ];
    for args in cases {
        let output = run(&home, args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.starts_with("error:"), "{args:?}: {stderr}");
        assert!(
            !stderr.contains("cli.not-implemented"),
            "{args:?}: {stderr}"
        );
        assert!(!stderr.contains('\u{1b}'), "{args:?}: {stderr}");
    }
    let valid = run(
        &home,
        &[
            "modify",
            "ghostty",
            "--target",
            "linux=~/x",
            "--reset",
            "target.windows",
        ],
    );
    assert!(String::from_utf8_lossy(&valid.stderr).contains("manage.failed"));
}

#[test]
fn queries_resolve_profile_values_and_report_template_references() {
    let home = temporary_home().unwrap();
    let source = home.path().join(".local/share/rosamta");
    fs::create_dir_all(source.join("modules")).unwrap();
    fs::write(
        source.join("manifest.toml"),
        r#"
[outputs.sample]
template = "sample.j2"
target = "~/.sample"
"#,
    )
    .unwrap();
    fs::write(
        source.join("profile.toml"),
        "[[rules]]\nenable = [\"sample\"]\n[rules.values.theme]\nforeground = \"#AABBCC\"\n",
    )
    .unwrap();
    fs::write(
        source.join("modules/colors.toml"),
        "[values.theme]\nforeground = \"#112233\"\n",
    )
    .unwrap();
    fs::write(source.join("sample.j2"), "{{ theme.foreground }}\n").unwrap();
    let output = run(
        &home,
        &["query", "value", "theme.foreground", "--format", "json"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["result"]["value"], "#aabbcc");
    assert_eq!(value["result"]["source"]["layer"], "profile");
    assert_eq!(value["result"]["overridden"][1]["layer"], "user-module");

    let output = run(
        &home,
        &["query", "refs", "theme.foreground", "--format", "json"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let listed = value["result"]["outputs"].as_array().unwrap();
    assert!(
        listed
            .iter()
            .any(|output| output["name"] == "sample" && output["enabled"] == true)
    );
    assert!(
        listed
            .iter()
            .any(|output| output["name"] == "ghostty" && output["enabled"] == false)
    );
    assert!(!home.path().join(".sample").exists());
}

#[test]
fn preview_renders_templates_and_rejects_colliding_targets_without_writes() {
    let home = temporary_home().unwrap();
    let source = home.path().join(".local/share/rosamta");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("manifest.toml"),
        "[outputs.sample]\ntemplate = \"sample.j2\"\ntarget = \"~/.sample\"\n",
    )
    .unwrap();
    fs::write(
        source.join("profile.toml"),
        "[[rules]]\nenable = [\"sample\"]\n",
    )
    .unwrap();
    fs::write(
        source.join("sample.j2"),
        "rgb = {{ rgb(theme.foreground) }}\n",
    )
    .unwrap();
    let output = run(&home, &["diff", "--format", "json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["result"]["items"][0]["action"], "create");
    assert!(
        value["result"]["items"][0]["diff"]
            .as_str()
            .unwrap()
            .contains("+rgb = [229, 229, 229]")
    );
    assert!(!home.path().join(".sample").exists());
    assert!(!home.path().join(".local/state").exists());

    fs::write(source.join("manifest.toml"), "[outputs.a]\ntemplate = \"sample.j2\"\ntarget = \"~/.sample\"\n[outputs.b]\ntemplate = \"sample.j2\"\ntarget = \"~/.sample\"\n").unwrap();
    fs::write(
        source.join("profile.toml"),
        "[[rules]]\nenable = [\"a\", \"b\"]\n",
    )
    .unwrap();
    let output = run(&home, &["diff"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("target ~/.sample"));
    assert!(!home.path().join(".sample").exists());
}

fn deployment_fixture(home: &TempHome) -> std::path::PathBuf {
    let source = home.path().join(".local/share/rosamta");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("manifest.toml"), "[outputs.first]\ncopy = \"contents\"\ntarget = \"~/a.txt\"\n[outputs.second]\ncopy = \"contents\"\ntarget = \"~/z.txt\"\n").unwrap();
    fs::write(
        source.join("profile.toml"),
        "[[rules]]\nenable = [\"first\", \"second\"]\n",
    )
    .unwrap();
    fs::write(source.join("contents"), "first\n").unwrap();
    source
}

fn successful(home: &TempHome, args: &[&str]) -> Output {
    let output = run(home, args);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn apply_tracks_drift_and_accepts_it_without_changing_the_source() {
    let home = temporary_home().unwrap();
    let source = deployment_fixture(&home);
    let out = successful(&home, &["apply", "--format", "json"]);
    let applied: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(applied["result"]["items"][0]["action"], "create");
    let target = home.path().join("a.txt");
    let state = home.path().join(".local/state/rosamta/state.json");
    let before = fs::read(&state).unwrap();
    successful(&home, &["apply"]);
    assert_eq!(before, fs::read(&state).unwrap());
    fs::write(&target, "manual\n").unwrap();
    assert_eq!(run(&home, &["apply", "--strict"]).status.code(), Some(1));
    assert_eq!(run(&home, &["apply"]).status.code(), Some(2));
    successful(&home, &["apply", "--accept", "~/a.txt"]);
    assert_eq!(fs::read_to_string(&target).unwrap(), "manual\n");
    assert_eq!(
        fs::read_to_string(source.join("contents")).unwrap(),
        "first\n"
    );
    let value: Value = serde_json::from_slice(&fs::read(&state).unwrap()).unwrap();
    assert_ne!(
        value["outputs"][0]["observed"],
        value["outputs"][0]["desired"]
    );
    fs::write(source.join("contents"), "new desired\n").unwrap();
    successful(&home, &["apply"]);
    assert_eq!(fs::read_to_string(&target).unwrap(), "new desired\n");
    fs::write(
        source.join("profile.toml"),
        "[[rules]]\nenable = [\"second\"]\n",
    )
    .unwrap();
    successful(&home, &["apply"]);
    assert!(!target.exists());
}

#[cfg(windows)]
#[test]
fn apply_rolls_back_earlier_updates_when_a_later_target_is_busy() {
    use std::os::windows::fs::OpenOptionsExt;
    let home = temporary_home().unwrap();
    let source = deployment_fixture(&home);
    successful(&home, &["apply"]);
    let state = home.path().join(".local/state/rosamta/state.json");
    let before = fs::read(&state).unwrap();
    fs::write(source.join("contents"), "second\n").unwrap();
    let _blocked = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(home.path().join("z.txt"))
        .unwrap();
    let output = run(&home, &["apply", "--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["result"]["items"][0]["status"], "reverted", "{value}");
    assert_eq!(value["result"]["items"][1]["status"], "failed");
    assert_eq!(
        fs::read_to_string(home.path().join("a.txt")).unwrap(),
        "first\n"
    );
    assert_eq!(
        fs::read_to_string(home.path().join("z.txt")).unwrap(),
        "first\n"
    );
    assert_eq!(fs::read(&state).unwrap(), before);
}

#[test]
fn add_modify_and_forget_preserve_the_target_and_existing_manifest_format() {
    let home = temporary_home().unwrap();
    let source = home.path().join(".local/share/rosamta");
    fs::create_dir_all(&source).unwrap();
    let manifest = source.join("manifest.toml");
    let original = b"\xef\xbb\xbf# Keep this comment\r\n[builtin.ghostty]\r\nownership = \"full\" # Keep this too\r\n";
    fs::write(&manifest, original).unwrap();
    let target = home.path().join(".gitconfig");
    fs::write(&target, b"binary\xff").unwrap();
    successful(&home, &["add", "~/.gitconfig"]);
    assert!(fs::read(&manifest).unwrap().starts_with(original));
    assert_eq!(
        fs::read(source.join("files/.gitconfig")).unwrap(),
        b"binary\xff"
    );
    successful(&home, &["modify", "gitconfig", "--takeover", "true"]);
    successful(&home, &["modify", "gitconfig", "--reset", "takeover"]);
    fs::write(
        source.join("profile.toml"),
        "[[rules]]\nenable = [\"gitconfig\"]\n",
    )
    .unwrap();
    successful(&home, &["apply"]);
    successful(&home, &["forget", "gitconfig"]);
    successful(&home, &["apply"]);
    assert_eq!(fs::read(&target).unwrap(), b"binary\xff");
    assert!(fs::read(&manifest).unwrap().starts_with(original));
    let state: Value = serde_json::from_slice(
        &fs::read(home.path().join(".local/state/rosamta/state.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(state["outputs"], json!([]));
}

#[test]
fn init_merges_local_settings_and_prepares_an_empty_source() {
    let home = temporary_home().unwrap();
    let config = home.path().join(".config/rosamta");
    fs::create_dir_all(&config).unwrap();
    fs::write(
        config.join("local.toml"),
        "# Local settings\ntags = [\"work\"] # Keep tags\n[values.font]\nsize = 12 # Keep size\n",
    )
    .unwrap();
    let result = successful(
        &home,
        &[
            "init",
            "--source",
            "~/dotfiles",
            "--tag",
            "work",
            "--tag",
            "laptop",
            "--set",
            "font.family=JetBrains Mono",
            "--set",
            "font.size=16",
            "--format",
            "json",
        ],
    );
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["result"]["source_created"], true);
    assert_eq!(value["result"]["apply"], Value::Null);
    assert_eq!(
        fs::read(home.path().join("dotfiles/manifest.toml")).unwrap(),
        b""
    );
    assert!(!home.path().join(".local/state/rosamta/state.json").exists());
    let local = fs::read_to_string(config.join("local.toml")).unwrap();
    assert!(
        local.contains("# Local settings")
            && local.contains("# Keep tags")
            && local.contains("size = 16 # Keep size"),
        "{local}"
    );
    let value = successful(
        &home,
        &["query", "value", "font.family", "--format", "json"],
    );
    let value: Value = serde_json::from_slice(&value.stdout).unwrap();
    assert_eq!(value["result"]["value"], "JetBrains Mono");
    let before = fs::read(config.join("local.toml")).unwrap();
    let existing = run(
        &home,
        &[
            "init",
            "https://example.invalid/test",
            "--source",
            "~/dotfiles",
            "--no-apply",
        ],
    );
    assert_eq!(existing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&existing.stderr).contains("already exists"));
    assert_eq!(fs::read(config.join("local.toml")).unwrap(), before);
}
