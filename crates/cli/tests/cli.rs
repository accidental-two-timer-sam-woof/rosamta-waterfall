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
fn unavailable_commands_report_failure_without_writing_to_home() {
    let home = temporary_home().unwrap();
    let output = run(&home, &["apply"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error[cli.not-implemented]"));

    for args in [["--format", "json", "apply"], ["apply", "--format", "json"]] {
        let output = run(&home, &args);
        assert_eq!(output.status.code(), Some(1));
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            result,
            json!({"json_version": 1, "command": "apply", "exit_code": 1, "result": null})
        );
        let diagnostic: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(diagnostic["code"], "cli.not-implemented");
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
    assert!(String::from_utf8_lossy(&valid.stderr).contains("cli.not-implemented"));
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
