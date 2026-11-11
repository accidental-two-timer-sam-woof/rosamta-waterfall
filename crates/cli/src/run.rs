use std::{env, io::Write, process::ExitCode};

use rosamta_core::{
    apply,
    config::{
        load::Config,
        schema::{self, Kind},
    },
    diag::{DiagCode, Diagnostics},
    query::QueryService,
    resolve,
};
use rosamta_platform::{SystemFs, host};
use serde_json::Value;

use crate::{
    args,
    cli::{ChoicesArgs, Cli, Command, Format, InitArgs, Query, Schema},
    output::{json, plan, text},
};

pub(crate) fn run(cli: Cli, mut stdout: impl Write, mut stderr: impl Write) -> ExitCode {
    let command = cli.command.name();
    let mut diagnostics = Diagnostics::default();
    let mut exit_code = None;
    let result = match &cli.command {
        Command::Schema(schema) => describe(schema)
            .map_err(|error| diagnostics.error(DiagCode::InvalidConfig, error))
            .ok(),
        Command::List | Command::Query(_) => inspect(&cli.command, &mut diagnostics),
        Command::Apply(args) => deploy(args, &mut diagnostics, &mut exit_code),
        Command::Add { .. } | Command::Modify(_) | Command::Forget { .. } => {
            manage(&cli.command, &mut diagnostics)
        }
        Command::Diff(args) | Command::Status(args) => preview(
            args,
            matches!(cli.command, Command::Diff(_)),
            &mut diagnostics,
        ),
        Command::Init(options) => initialize(options, &mut diagnostics, &mut exit_code),
    };
    let code = exit_code.unwrap_or_else(|| u8::from(result.is_none() || diagnostics.has_errors()));

    let output = match cli.format {
        Format::Text => {
            let print = result
                .as_ref()
                .filter(|(_, display)| !display.is_empty())
                .map_or(Ok(()), |(_, display)| writeln!(stdout, "{display}"));
            print.and_then(|()| text::write(&mut stderr, &diagnostics))
        }
        Format::Json => json::write(
            &mut stdout,
            &mut stderr,
            command,
            code,
            result.as_ref().map(|(value, _)| value),
            &diagnostics,
        ),
    };
    if let Err(error) = output {
        let _ = writeln!(stderr, "error: failed to write command output: {error}");
        return ExitCode::FAILURE;
    }

    ExitCode::from(code)
}

fn initialize(
    options: &InitArgs,
    diagnostics: &mut Diagnostics,
    code: &mut Option<u8>,
) -> Option<(Value, String)> {
    use rosamta_core::{
        init::{self, Request},
        path::HomePath,
    };
    use serde_json::json;
    let host = report(host::collect(), diagnostics)?;
    let fs = report(SystemFs::open(&host.home), diagnostics)?;
    let cwd = report(env::current_dir(), diagnostics)?;
    let source = report(
        options
            .source
            .as_ref()
            .map(|path| HomePath::argument(path, &host, &cwd))
            .transpose(),
        diagnostics,
    )?;
    let apply = if options.repo.is_some() && !options.no_apply {
        Some(report(
            args::choices(&options.choices, &host, &cwd),
            diagnostics,
        )?)
    } else {
        None
    };
    let batch = report(batch_id(), diagnostics)?;
    let result = init::initialize(
        &fs,
        &fs,
        &host,
        Request {
            repo: options.repo.as_deref(),
            source,
            tags: &options.tag,
            values: &options.set,
            apply,
            batch: &batch,
        },
    )
    .map_err(|error| diagnostics.error(DiagCode::InitFailed, error))
    .ok()?;
    let mut lines = vec![format!("Source directory: {}", result.source)];
    lines.extend(
        result
            .changes
            .iter()
            .map(|(path, kind)| format!("{kind:<8} {path}")),
    );
    let apply = match result.apply {
        Some(Ok((analysis, result))) => {
            report_apply(&analysis, &result, diagnostics);
            *code = Some(result.exit_code());
            let (value, text) = plan::applied(&analysis.plan, &host, &result);
            if !text.is_empty() {
                lines.push(text);
            }
            Some(value)
        }
        Some(Err(errors)) => {
            for error in errors {
                diagnostics.error(DiagCode::PreflightFailed, error);
            }
            None
        }
        None => None,
    };
    Some((
        json!({"source": result.source.to_absolute(&host), "source_created": result.source_created, "apply": apply,
        "changes": result.changes.iter().map(|(path, kind)| json!({"path": path.to_absolute(&host), "kind": kind})).collect::<Vec<_>>() }),
        lines.join("\n"),
    ))
}

fn manage(command: &Command, diagnostics: &mut Diagnostics) -> Option<(Value, String)> {
    use rosamta_core::{manage::ManifestService, path::HomePath};
    use serde_json::json;
    let host = report(host::collect(), diagnostics)?;
    let fs = report(SystemFs::open(&host.home), diagnostics)?;
    let service = ManifestService {
        fs: &fs,
        host: &host,
    };
    let result = match command {
        Command::Add { file, name } => {
            let cwd = report(env::current_dir(), diagnostics)?;
            let target = report(HomePath::argument(file, &host, &cwd), diagnostics)?;
            service.add(target, name.clone())
        }
        Command::Modify(options) => service.modify(
            options.output.clone(),
            report(args::manifest_patch(options), diagnostics)?,
        ),
        Command::Forget { output } => service.forget(output.clone()),
        _ => return None,
    }
    .map_err(|error| diagnostics.error(DiagCode::ManageFailed, error))
    .ok()?;
    let mut lines: Vec<_> = result
        .changes
        .iter()
        .map(|(path, kind)| format!("{kind:<8} {path}"))
        .collect();
    if matches!(command, Command::Add { .. }) {
        lines.push(format!(
            "Enable {} in profile.toml to include it in the next plan.",
            result.output
        ));
    } else if lines.is_empty() {
        lines.push(format!("{} unchanged", result.output));
    }
    let value = json!({"output": result.output.to_string(), "unchanged": result.changes.is_empty(),
        "changes": result.changes.iter().map(|(path, kind)| json!({"path": path.to_absolute(&host), "kind": kind})).collect::<Vec<_>>()});
    Some((value, lines.join("\n")))
}

fn report<T>(result: Result<T, impl ToString>, diagnostics: &mut Diagnostics) -> Option<T> {
    result
        .map_err(|error| diagnostics.error(DiagCode::InvalidConfig, error.to_string()))
        .ok()
}

fn batch_id() -> Result<String, time::error::Format> {
    use time::{OffsetDateTime, macros::format_description};
    OffsetDateTime::now_utc().format(format_description!(
        "[year][month][day]T[hour][minute][second]Z"
    ))
}

fn deploy(
    options: &ChoicesArgs,
    diagnostics: &mut Diagnostics,
    code: &mut Option<u8>,
) -> Option<(Value, String)> {
    let host = report(host::collect(), diagnostics)?;
    let fs = report(SystemFs::open(&host.home), diagnostics)?;
    let cwd = report(env::current_dir(), diagnostics)?;
    let choices = report(args::choices(options, &host, &cwd), diagnostics)?;
    let batch = report(batch_id(), diagnostics)?;
    let (analysis, result) = apply::deploy(&fs, &host, &choices, &batch)
        .map_err(|errors| {
            for error in errors {
                diagnostics.error(DiagCode::PreflightFailed, error);
            }
        })
        .ok()?;
    *code = Some(result.exit_code());
    report_apply(&analysis, &result, diagnostics);
    Some(plan::applied(&analysis.plan, &host, &result))
}

fn report_apply(
    analysis: &apply::Analysis,
    result: &rosamta_core::exec::Report,
    diagnostics: &mut Diagnostics,
) {
    for warning in analysis.plan.warnings.iter().chain(&result.warnings) {
        diagnostics.warning(DiagCode::ApplyWarning, warning);
    }
    if let Some(failure) = &result.failure {
        let recovery = if result.restored {
            "; completed changes were rolled back"
        } else {
            ""
        };
        diagnostics.error(
            DiagCode::ExecuteFailed,
            format!("{}: {}{recovery}", failure.path, failure.message),
        );
    }
    for failure in &result.recovery {
        diagnostics.error(
            DiagCode::RecoveryFailed,
            format!("{}: {}", failure.path, failure.reason),
        );
    }
}

fn preview(
    options: &ChoicesArgs,
    diff: bool,
    diagnostics: &mut Diagnostics,
) -> Option<(Value, String)> {
    let host = report(host::collect(), diagnostics)?;
    let fs = report(SystemFs::open(&host.home), diagnostics)?;
    let cwd = report(env::current_dir(), diagnostics)?;
    let choices = report(args::choices(options, &host, &cwd), diagnostics)?;
    match apply::analyze(&fs, &host, &choices) {
        Ok(analysis) => {
            for warning in &analysis.plan.warnings {
                diagnostics.warning(DiagCode::ApplyWarning, warning);
            }
            Some(plan::preview(&analysis.plan, &host, diff))
        }
        Err(errors) => {
            for error in errors {
                diagnostics.error(DiagCode::PreflightFailed, error);
            }
            None
        }
    }
}

fn inspect(command: &Command, diagnostics: &mut Diagnostics) -> Option<(Value, String)> {
    let host = report(host::collect(), diagnostics)?;
    let fs = report(SystemFs::open(&host.home), diagnostics)?;
    let config = report(
        Config::load(&fs, &host).map_err(|errors| errors.join("\n")),
        diagnostics,
    )?;
    for name in config.enabled(&host).1 {
        diagnostics.warning(
            DiagCode::UnknownOutput,
            format!("profile enables unknown or removed output {name}"),
        );
    }
    let resolved = report(
        resolve::resolve(&config, &host).map_err(|errors| errors.join("\n")),
        diagnostics,
    )?;
    let query = QueryService {
        config: &config,
        resolved: &resolved,
        fs: &fs,
        host: &host,
    };
    let result = match command {
        Command::List => Ok(query.list()),
        Command::Query(Query::Refs { reference }) => query.refs(reference),
        Command::Query(Query::Output { key }) => {
            query.output(key, &report(env::current_dir(), diagnostics)?)
        }
        Command::Query(Query::Value { field }) => query.value(field),
        _ => return None,
    }
    .map_err(|error| diagnostics.error(DiagCode::QueryFailed, error))
    .ok()?;
    for error in result.errors {
        diagnostics.error(DiagCode::QueryFailed, error);
    }
    Some((result.value, result.lines.join("\n")))
}

fn describe(schema: &Schema) -> Result<(Value, String), String> {
    let kind = match schema {
        Schema::Manifest => Kind::Manifest,
        Schema::Module => Kind::Module,
        Schema::Profile => Kind::Profile,
        Schema::Local => Kind::Local,
        Schema::Builtin { module } => {
            let value = schema::builtin_fields(module.as_ref())?;
            let display = value["modules"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|module| {
                    let fields = module["fields"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|field| {
                            format!(
                                "  {} ({}, {}): {}  Range: {}  Default: {}",
                                field["name"].as_str().unwrap_or_default(),
                                field["type"].as_str().unwrap_or_default(),
                                if field["required"] == true {
                                    "required"
                                } else {
                                    "optional"
                                },
                                field["description"].as_str().unwrap_or_default(),
                                field["range"],
                                field["default"]
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("{}\n{fields}", module["name"].as_str().unwrap_or_default())
                })
                .collect::<Vec<_>>()
                .join("\n\n");
            return Ok((value, display));
        }
    };
    let schema = kind.describe();
    let mut lines = vec![format!("{} schema", schema.kind)];
    lines.extend(schema.keys.iter().map(|key| {
        format!(
            "  {} ({}, {}): {}{}",
            key.key,
            key.kind,
            if key.required { "required" } else { "optional" },
            key.description,
            key.default
                .map_or_else(String::new, |default| format!("  Default: {default}"))
        )
    }));
    lines.push(format!("\nExample:\n{}", schema.example.trim_end()));
    let value = serde_json::to_value(schema).map_err(|error| error.to_string())?;
    Ok((value, lines.join("\n")))
}
