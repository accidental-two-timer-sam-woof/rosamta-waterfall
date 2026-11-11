//! Diagnostics shared by the core services and the CLI (05 §4.1).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
}

impl Severity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagCode {
    CommandUnavailable,
    InvalidConfig,
}

impl DiagCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CommandUnavailable => "cli.not-implemented",
            Self::InvalidConfig => "config.invalid",
        }
    }
}

#[derive(Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagCode,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    pub fn error(&mut self, code: DiagCode, message: impl Into<String>) {
        self.0.push(Diagnostic {
            severity: Severity::Error,
            code,
            message: message.into(),
        });
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.0.iter()
    }
}
