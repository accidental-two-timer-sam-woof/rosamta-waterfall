//! P5: MiniJinja static references and error messages.
//!
//! One test per MiniJinja row of the probe table in 05 §2.3 (04 §4 row 2,
//! FR-QRY-002–004, USE-003, FR-PLN-005), plus the trailing-newline setting
//! that §2.3 relies on and the template metadata comment of §5.9.

use std::collections::BTreeSet;

use minijinja::syntax::SyntaxConfig;
use minijinja::{Environment, Error, ErrorKind, UndefinedBehavior, Value, context};

const NAME: &str = "alacritty.toml.j2";

/// Configured as §2.3 decides: empty environment, strict undefined, and only
/// the tests and functions registered here.
fn env() -> Environment<'static> {
    let mut env = bare_env();
    env.add_test("none", minijinja::tests::is_none);
    env.add_function("hex_to_rgb", |hex: String| -> Result<String, Error> {
        let v = u32::from_str_radix(hex.trim_start_matches('#'), 16)
            .map_err(|e| Error::new(ErrorKind::InvalidOperation, e.to_string()))?;
        Ok(format!("{},{},{}", v >> 16, (v >> 8) & 0xff, v & 0xff))
    });
    env
}

/// Empty environment with strict undefined and nothing registered.
fn bare_env() -> Environment<'static> {
    let mut env = Environment::empty();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env
}

fn refs(env: &Environment, name: &str) -> BTreeSet<String> {
    env.get_template(name)
        .unwrap()
        .undeclared_variables(true)
        .into_iter()
        .collect()
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn theme() -> Value {
    context! {
        foreground => "#d0d0d0",
        opacity => (),
        cursor => "#ffffff",
        selection => "#444444",
    }
}

/// Renders `source` as template `NAME` with a context that has `theme` and
/// `font` but no `shellrc`.
fn render(source: &str) -> Result<String, Error> {
    let mut env = env();
    env.add_template_owned(NAME, source.to_string())?;
    env.get_template(NAME)?.render(
        context! { os => "linux", theme => theme(), font => context! { family => "Iosevka" } },
    )
}

const MAIN: &str = r#"[colors.primary]
foreground = "{{ theme.foreground }}"
{% if theme.opacity is not none %}opacity = {{ theme.opacity }}
{% endif %}{% if os == "windows" %}shell = "{{ shellrc.windows_shell }}"
{% endif %}font = "{{ font.family }}"
{% for key in ["cursor", "selection"] %}{{ key }} = "{{ theme[key] }}"
{% endfor %}cursor_rgb = "{{ hex_to_rgb(theme['cursor']) }}"
"#;

#[test]
fn references_cover_untaken_branches_and_fall_back_to_the_module() {
    let mut env = env();
    env.add_template(NAME, MAIN).unwrap();
    assert_eq!(
        refs(&env, NAME),
        set(&[
            "font.family",
            "hex_to_rgb",
            "os",
            "shellrc.windows_shell",
            "theme",
            "theme.foreground",
            "theme.opacity",
        ])
    );
    // The windows branch is not taken, so the missing `shellrc` never errors.
    assert!(render(MAIN).is_ok());
}

#[test]
fn references_do_not_follow_include() {
    let mut env = env();
    env.add_template("part.j2", "size = {{ font.size }}\n")
        .unwrap();
    env.add_template(NAME, "{{ theme.foreground }}\n{% include \"part.j2\" %}")
        .unwrap();
    assert_eq!(refs(&env, NAME), set(&["theme.foreground"]));
}

fn expect_err(source: &str) -> Error {
    render(source).expect_err("must fail")
}

/// The full message as rosamta would show it: kind, detail, name and line.
fn message(err: &Error) -> String {
    format!("{err:#}")
}

#[test]
fn misspelled_field_names_template_line_and_path() {
    let err = expect_err("a\nb = \"{{ theme.forground }}\"\n");
    assert_eq!(err.kind(), ErrorKind::UndefinedError);
    assert_eq!(err.name(), Some(NAME));
    assert_eq!(err.line(), Some(2));
    assert!(
        message(&err).contains("theme.forground"),
        "{}",
        message(&err)
    );
}

#[test]
fn missing_module_names_template_line_and_name() {
    let err = expect_err("a\n\n{{ colors.red }}\n");
    assert_eq!((err.name(), err.line()), (Some(NAME), Some(3)));
    assert!(message(&err).contains("colors"), "{}", message(&err));
}

#[test]
fn missing_function_names_template_line_and_name() {
    let err = expect_err("{{ rgb_to_hex(theme.foreground) }}\n");
    assert_eq!((err.name(), err.line()), (Some(NAME), Some(1)));
    assert!(message(&err).contains("rgb_to_hex"), "{}", message(&err));
}

#[test]
fn missing_field_in_condition_fails_under_strict() {
    let err = expect_err("{% if theme.blink %}x{% endif %}\n");
    assert_eq!(err.kind(), ErrorKind::UndefinedError);
    assert_eq!(err.line(), Some(1));
}

#[test]
fn unclosed_if_reports_the_opening_line() {
    let err = expect_err("a\n{% if theme.opacity is not none %}\nb\nc\n");
    assert_eq!(err.kind(), ErrorKind::SyntaxError);
    assert_eq!(err.line(), Some(2));
}

#[test]
fn unterminated_string_reports_the_last_line() {
    // The string starts on line 2; the error points at line 4, the last line.
    let err = expect_err("a\n{{ \"oops }}\nb\nc\n");
    assert_eq!(err.kind(), ErrorKind::SyntaxError);
    assert_eq!(err.line(), Some(4));
}

#[test]
fn none_optional_field_is_skipped() {
    let out = render("{% if theme.opacity is not none %}opacity = 1\n{% endif %}end\n").unwrap();
    assert_eq!(out, "end");
}

#[test]
fn none_test_must_be_registered_in_the_empty_environment() {
    let mut env = bare_env();
    env.add_template(NAME, "{% if x is not none %}x{% endif %}")
        .unwrap();
    let err = env
        .get_template(NAME)
        .unwrap()
        .render(context! { x => () })
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::UnknownTest);
}

#[test]
fn include_outside_source_without_loader_is_not_found() {
    let err = expect_err("{% include \"../../secret\" %}");
    assert_eq!(err.kind(), ErrorKind::TemplateNotFound);
}

#[test]
fn include_outside_source_is_stopped_by_the_loader() {
    let mut env = env();
    env.set_loader(|name| {
        if name.split('/').any(|part| part == "..") {
            Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("outside the source directory: {name}"),
            ))
        } else {
            Ok(None::<String>)
        }
    });
    env.add_template(NAME, "a\n{% include \"../../secret\" %}")
        .unwrap();
    let err = env
        .get_template(NAME)
        .unwrap()
        .render(context! {})
        .unwrap_err();
    assert!(
        message(&err).contains("outside the source directory"),
        "{}",
        message(&err)
    );
}

#[test]
fn empty_environment_has_no_unregistered_builtins() {
    let filter = expect_err("{{ theme.foreground | upper }}");
    assert_eq!(filter.kind(), ErrorKind::UnknownFilter);
    let function = expect_err("{% for i in range(3) %}{% endfor %}");
    assert!(
        message(&function).contains("range"),
        "{}",
        message(&function)
    );
}

#[test]
fn trailing_newline_is_kept_only_when_configured() {
    let mut env = env();
    env.add_template("t", "x\n").unwrap();
    assert_eq!(
        env.get_template("t").unwrap().render(context! {}).unwrap(),
        "x"
    );

    env.set_syntax(
        SyntaxConfig::builder()
            .keep_trailing_newline(true)
            .build()
            .unwrap(),
    );
    env.add_template("t", "x\n").unwrap();
    assert_eq!(
        env.get_template("t").unwrap().render(context! {}).unwrap(),
        "x\n"
    );
}

/// 05 §5.9: template metadata written as a leading `{#- rosamta ... -#}`
/// comment renders as nothing, drops the newline after it, adds no
/// references, and leaves error line numbers pointing at the file's lines.
#[test]
fn metadata_comment_is_invisible_and_keeps_line_numbers() {
    let header = "{#- rosamta\nprogram = \"ghostty\"\nformat_version = \"1.1\"\n[requires]\ntheme = 1\n-#}\n";
    let mut env = env();
    env.set_syntax(
        SyntaxConfig::builder()
            .keep_trailing_newline(true)
            .build()
            .unwrap(),
    );

    env.add_template_owned("ok", format!("{header}fg = {{{{ theme.foreground }}}}\n"))
        .unwrap();
    assert_eq!(
        env.get_template("ok")
            .unwrap()
            .render(context! { theme => theme() })
            .unwrap(),
        "fg = #d0d0d0\n"
    );
    assert_eq!(refs(&env, "ok"), set(&["theme.foreground"]));

    // Line 7 of the file: six header lines, then the line below.
    env.add_template_owned("bad", format!("{header}x = {{{{ theme.forground }}}}\n"))
        .unwrap();
    let err = env
        .get_template("bad")
        .unwrap()
        .render(context! { theme => theme() })
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::UndefinedError);
    assert_eq!(err.line(), Some(7));
}
