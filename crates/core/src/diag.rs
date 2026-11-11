//! Diagnostics shared by the core services and the CLI (05 §4.1).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagCode {
    CommandUnavailable,
    InvalidConfig,
    UnknownOutput,
    QueryFailed,
    PreflightFailed,
    ApplyWarning,
    ExecuteFailed,
    RecoveryFailed,
}

impl DiagCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CommandUnavailable => "cli.not-implemented",
            Self::InvalidConfig => "config.invalid",
            Self::UnknownOutput => "config.unknown-output",
            Self::QueryFailed => "query.failed",
            Self::PreflightFailed => "preflight.failed",
            Self::ApplyWarning => "apply.warning",
            Self::ExecuteFailed => "exec.failed",
            Self::RecoveryFailed => "recovery.failed",
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

    pub fn warning(&mut self, code: DiagCode, message: impl Into<String>) {
        self.0.push(Diagnostic {
            severity: Severity::Warning,
            code,
            message: message.into(),
        });
    }

    pub fn has_errors(&self) -> bool {
        self.0
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}
