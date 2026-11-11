use std::{io::Write, process::ExitCode};

use rosamta_core::diag::{DiagCode, Diagnostics};

use crate::{
    cli::{Cli, Format},
    output::{json, text},
};

pub(crate) fn run(cli: Cli, mut stdout: impl Write, mut stderr: impl Write) -> ExitCode {
    let command = cli.command.name();
    let mut diagnostics = Diagnostics::default();
    diagnostics.error(
        DiagCode::CommandUnavailable,
        format!("`{command}` is unavailable in this development build"),
    );

    let output = match cli.format {
        Format::Text => text::write(&mut stderr, &diagnostics),
        Format::Json => json::write(&mut stdout, &mut stderr, command, &diagnostics),
    };
    if let Err(error) = output {
        let _ = writeln!(stderr, "error: failed to write command output: {error}");
    }

    ExitCode::FAILURE
}
