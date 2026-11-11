use std::io::{self, Write};

use rosamta_core::diag::{Diagnostic, Diagnostics};
use serde::Serialize;

#[derive(Serialize)]
struct FailedCommand<'a> {
    json_version: u8,
    command: &'a str,
    exit_code: u8,
    result: Option<()>,
}

#[derive(Serialize)]
struct JsonDiagnostic<'a> {
    severity: &'a str,
    code: &'a str,
    message: &'a str,
    location: Option<()>,
    related: [(); 0],
    subjects: [(); 0],
    hint: Option<()>,
}

impl<'a> From<&'a Diagnostic> for JsonDiagnostic<'a> {
    fn from(diagnostic: &'a Diagnostic) -> Self {
        Self {
            severity: diagnostic.severity.as_str(),
            code: diagnostic.code.as_str(),
            message: &diagnostic.message,
            location: None,
            related: [],
            subjects: [],
            hint: None,
        }
    }
}

pub(crate) fn write(
    stdout: &mut impl Write,
    stderr: &mut impl Write,
    command: &str,
    diagnostics: &Diagnostics,
) -> io::Result<()> {
    let result = FailedCommand {
        json_version: 1,
        command,
        exit_code: 1,
        result: None,
    };
    serde_json::to_writer(&mut *stdout, &result)?;
    writeln!(stdout)?;

    for diagnostic in diagnostics.iter() {
        serde_json::to_writer(&mut *stderr, &JsonDiagnostic::from(diagnostic))?;
        writeln!(stderr)?;
    }
    Ok(())
}
