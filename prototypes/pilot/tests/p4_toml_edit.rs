//! P4: format-preserving edits of the user configuration with toml_edit.
//!
//! One test per row of the probe table in 05 §2.2 (04 §4 row 1, IF-010,
//! FR-MAN-029, FR-MAN-030). Each edit test spells out the whole expected
//! file, so any byte that changes outside the target lines fails the test.

use serde::Deserialize;
use toml_edit::{DocumentMut, Item, Table, Value, value};

const MANIFEST: &str = r#"# rosamta manifest（pilot 用）
# 每個 [[output]] 宣告一個輸出

[source]
root = "~/.local/share/rosamta"   # 來源目錄

# 終端機配色
[[output]]
name = "alacritty-theme"
template = "templates/alacritty.toml.j2"
target = "~/.config/alacritty/theme.toml"
when = { os = ["linux", "macos"], tags = ["desktop"] }

# Neovim 配色，tab 縮排
[[output]]
	name  =  "nvim-colors"	# 行尾註解
	template = "templates/nvim.lua.j2"
	target = "~/.config/nvim/colors/rosamta.lua"

[[output]]
  name = "bashrc"     # 兩格縮排
  source = "files/bashrc"
  target = "~/.bashrc"
  mode = "0644"
"#;

const LOCAL: &str = r##"# 本機設定（不進版本控制）
host = "laptop"
tags = ["desktop", "work"]   # 套用 Profile 的標籤

[theme]
foreground = "#d0d0d0"  # 只在這台主機覆寫
"##;

fn crlf(s: &str) -> String {
    s.replace('\n', "\r\n")
}

fn parse(s: &str) -> DocumentMut {
    s.parse().expect("fixture parses")
}

fn outputs(doc: &mut DocumentMut) -> &mut toml_edit::ArrayOfTables {
    doc["output"].as_array_of_tables_mut().unwrap()
}

fn output(doc: &mut DocumentMut, index: usize) -> &mut Table {
    outputs(doc).get_mut(index).unwrap()
}

/// Replaces `old` in `text`, asserting it occurs exactly once.
fn edit(text: &str, old: &str, new: &str) -> String {
    assert_eq!(text.matches(old).count(), 1, "{old:?} must occur once");
    text.replacen(old, new, 1)
}

/// Sets `item` to `new`, keeping the old value's surrounding whitespace and
/// trailing comment (its decor).
fn set_keep_decor(item: &mut Item, new: impl Into<Value>) {
    let old = item.as_value().unwrap().decor().clone();
    let mut new = new.into();
    *new.decor_mut() = old;
    *item = Item::Value(new);
}

#[test]
fn lf_round_trip_is_byte_identical() {
    assert_eq!(parse(MANIFEST).to_string(), MANIFEST);
    assert_eq!(parse(LOCAL).to_string(), LOCAL);
}

#[test]
fn crlf_round_trip_turns_every_line_to_lf() {
    let input = crlf(MANIFEST);
    let output = parse(&input).to_string();
    assert_ne!(output, input);
    assert_eq!(output, MANIFEST);
}

#[test]
fn crlf_converted_to_lf_and_back_is_byte_identical() {
    let input = crlf(MANIFEST);
    let lf = input.replace("\r\n", "\n");
    assert_eq!(crlf(&parse(&lf).to_string()), input);

    // An edit through the same conversion touches only the target line.
    let mut doc = parse(&lf);
    set_keep_decor(&mut output(&mut doc, 2)["target"], "~/.bashrc.d/main");
    let expected = edit(&input, "\"~/.bashrc\"", "\"~/.bashrc.d/main\"");
    assert_eq!(crlf(&doc.to_string()), expected);
}

#[test]
fn plain_assignment_resets_the_value_decor() {
    let mut doc = parse(MANIFEST);
    output(&mut doc, 1)["name"] = value("nvim");
    let expected = edit(
        MANIFEST,
        "\tname  =  \"nvim-colors\"\t# 行尾註解",
        "\tname  = \"nvim\"",
    );
    assert_eq!(doc.to_string(), expected);
}

#[test]
fn assignment_keeping_decor_changes_only_the_value() {
    let mut doc = parse(MANIFEST);
    set_keep_decor(&mut output(&mut doc, 1)["name"], "nvim");
    let expected = edit(MANIFEST, "\"nvim-colors\"", "\"nvim\"");
    assert_eq!(doc.to_string(), expected);
}

#[test]
fn inline_table_key_edit_changes_only_that_line() {
    let mut doc = parse(MANIFEST);
    let when = output(&mut doc, 0)["when"].as_inline_table_mut().unwrap();
    let tags = when.get_mut("tags").unwrap();
    let decor = tags.decor().clone();
    let mut new: Value = toml_edit::Array::from_iter(["desktop", "laptop"]).into();
    *new.decor_mut() = decor;
    *tags = new;
    let expected = edit(
        MANIFEST,
        "tags = [\"desktop\"] }",
        "tags = [\"desktop\", \"laptop\"] }",
    );
    assert_eq!(doc.to_string(), expected);
}

#[test]
fn adding_a_property_only_adds_a_line_without_table_indent() {
    let mut doc = parse(MANIFEST);
    output(&mut doc, 1)["mode"] = value("0600");
    let expected = edit(
        MANIFEST,
        "\ttarget = \"~/.config/nvim/colors/rosamta.lua\"\n",
        "\ttarget = \"~/.config/nvim/colors/rosamta.lua\"\nmode = \"0600\"\n",
    );
    assert_eq!(doc.to_string(), expected);
}

#[test]
fn adding_an_output_only_appends_lines() {
    let mut doc = parse(MANIFEST);
    let mut table = Table::new();
    table["name"] = value("tmux-theme");
    table["template"] = value("templates/tmux.conf.j2");
    table["target"] = value("~/.config/tmux/theme.conf");
    outputs(&mut doc).push(table);
    let expected = format!(
        "{MANIFEST}\n[[output]]\nname = \"tmux-theme\"\ntemplate = \"templates/tmux.conf.j2\"\ntarget = \"~/.config/tmux/theme.conf\"\n"
    );
    assert_eq!(doc.to_string(), expected);
}

#[test]
fn removing_an_output_takes_the_comment_above_it() {
    let mut doc = parse(MANIFEST);
    outputs(&mut doc).remove(1);
    let removed = "\n# Neovim 配色，tab 縮排\n[[output]]\n\tname  =  \"nvim-colors\"\t# 行尾註解\n\ttemplate = \"templates/nvim.lua.j2\"\n\ttarget = \"~/.config/nvim/colors/rosamta.lua\"\n";
    let expected = edit(MANIFEST, removed, "");
    assert_eq!(doc.to_string(), expected);
}

#[test]
fn local_tag_merge_changes_only_the_tag_line() {
    let mut doc = parse(LOCAL);
    let tags: Value = toml_edit::Array::from_iter(["desktop", "work", "gpu"]).into();
    set_keep_decor(&mut doc["tags"], tags);
    let expected = edit(
        LOCAL,
        "[\"desktop\", \"work\"]",
        "[\"desktop\", \"work\", \"gpu\"]",
    );
    assert_eq!(doc.to_string(), expected);
}

// ---- Reading through the same parser, and error positions (FR-MAN-029) ----

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    source: Source,
    output: Vec<Output>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    root: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    name: String,
    template: Option<String>,
    source: Option<String>,
    target: String,
    mode: Option<String>,
    when: Option<When>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct When {
    os: Option<Vec<String>>,
    tags: Option<Vec<String>>,
}

fn read(text: &str) -> Result<Manifest, String> {
    toml_edit::de::from_str::<Manifest>(text).map_err(|e| e.to_string())
}

/// 1-based line of the first occurrence of `needle` in `text`.
fn line_of(text: &str, needle: &str) -> usize {
    text[..text.find(needle).unwrap()].matches('\n').count() + 1
}

fn assert_error_at(text: &str, needle: &str, also: &[&str]) {
    let err = read(text).expect_err("must be rejected");
    let line = line_of(text, needle);
    assert!(
        err.contains(&format!("line {line}, column")),
        "expected line {line} in:\n{err}"
    );
    for part in also {
        assert!(err.contains(part), "expected {part:?} in:\n{err}");
    }
}

#[test]
fn fixture_reads_through_the_same_parser() {
    let m = read(MANIFEST).unwrap();
    assert_eq!(m.output.len(), 3);
    assert_eq!(m.output[1].name, "nvim-colors");
}

#[test]
fn syntax_error_has_line_and_column() {
    let text = edit(MANIFEST, "mode = \"0644\"", "mode = \"0644");
    assert_error_at(&text, "mode = \"0644", &[]);
}

#[test]
fn misspelled_property_names_the_valid_ones() {
    let text = edit(
        MANIFEST,
        "template = \"templates/nvim",
        "templat = \"templates/nvim",
    );
    assert_error_at(
        &text,
        "templat =",
        &["unknown field `templat`", "expected one of"],
    );
}

#[test]
fn duplicate_key_has_line() {
    let text = edit(
        MANIFEST,
        "  mode = \"0644\"\n",
        "  mode = \"0644\"\n  mode = \"0600\"\n",
    );
    assert_error_at(&text, "mode = \"0600\"", &["duplicate key"]);
}

#[test]
fn duplicate_table_has_line() {
    let text = format!("{MANIFEST}\n[source]\nroot = \"/tmp\"\n");
    assert_error_at(&text, "[source]\nroot = \"/tmp\"", &["duplicate key"]);
}

#[test]
fn wrong_type_has_line() {
    let text = edit(MANIFEST, "mode = \"0644\"", "mode = 644");
    assert_error_at(&text, "mode = 644", &["invalid type"]);
}

#[test]
fn multiline_string_keeps_crlf_unless_converted_first() {
    let text = "body = \"\"\"\nline one\nline two\n\"\"\"\n";
    let crlf_text = crlf(text);
    let doc = parse(&crlf_text);
    assert_eq!(doc["body"].as_str(), Some("line one\r\nline two\r\n"));
    let doc = parse(&crlf_text.replace("\r\n", "\n"));
    assert_eq!(doc["body"].as_str(), Some("line one\nline two\n"));
}
