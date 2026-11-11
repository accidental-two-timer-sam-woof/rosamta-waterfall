use std::{io::Write, process::ExitCode};

use rosamta_core::{
    config::schema::{self, Kind},
    diag::{DiagCode, Diagnostics},
};
use serde_json::Value;

use crate::{
    cli::{Cli, Command, Format, Schema},
    output::{json, text},
};

pub(crate) fn run(cli: Cli, mut stdout: impl Write, mut stderr: impl Write) -> ExitCode {
    let command = cli.command.name();
    let mut diagnostics = Diagnostics::default();
    let result = match &cli.command {
        Command::Schema(schema) => describe(schema)
            .map_err(|error| diagnostics.error(DiagCode::InvalidConfig, error))
            .ok(),
        _ => {
            diagnostics.error(
                DiagCode::CommandUnavailable,
                format!("`{command}` is unavailable in this development build"),
            );
            None
        }
    };
    let code = u8::from(result.is_none());

    let output = match cli.format {
        Format::Text => {
            let print = result
                .as_ref()
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
