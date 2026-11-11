use std::ffi::OsString;

use clap::{Args, ColorChoice, Parser, Subcommand, ValueEnum};
use rosamta_core::{
    name::{FieldRef, ModuleName, OutputName, Tag},
    query::RefQuery,
};

macro_rules! help {
    ($example:literal) => {
        concat!(
            "Exit status:\n  0  Help or version displayed\n",
            "  1  Invalid arguments or command unavailable in this development build\n\n",
            "Examples:\n  rosamta ",
            $example
        )
    };
}

#[derive(Debug, Parser)]
#[command(version, about = "Manage dotfiles across devices", color = ColorChoice::Never, propagate_version = true, after_long_help = help!("--help"))]
pub(crate) struct Cli {
    /// Set the output format.
    #[arg(long, global = true, value_enum, default_value = "text")]
    pub format: Format,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Format {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Set up a source directory and local settings.
    #[command(after_long_help = help!("init --help"))]
    Init(InitArgs),
    /// Preview changes with a content diff.
    #[command(after_long_help = help!("diff --help"))]
    Diff(ChoicesArgs),
    /// Summarize pending changes and drift.
    #[command(after_long_help = help!("status --help"))]
    Status(ChoicesArgs),
    /// Apply the deployment plan.
    #[command(after_long_help = help!("apply --help"))]
    Apply(ChoicesArgs),
    /// Inspect references, outputs or resolved field values.
    #[command(subcommand, after_long_help = help!("query refs --help"))]
    Query(Query),
    /// Add an existing file to the manifest.
    #[command(after_long_help = help!("add --help"))]
    Add {
        /// Path of the file to manage.
        #[arg(value_name = "FILE")]
        file: OsString,
        /// Override the generated output name.
        #[arg(long)]
        name: Option<OutputName>,
    },
    /// Change properties of a managed output.
    #[command(after_long_help = help!("modify --help"))]
    Modify(ModifyArgs),
    /// Stop managing an output while retaining its target file.
    #[command(after_long_help = help!("forget --help"))]
    Forget {
        /// Name of the output in the manifest.
        output: OutputName,
    },
    /// List declared outputs and their enablement.
    #[command(after_long_help = help!("list --help"))]
    List,
    /// Describe configuration formats or built-in modules.
    #[command(subcommand, after_long_help = help!("schema manifest --help"))]
    Schema(Schema),
}

impl Command {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Init(_) => "init",
            Self::Diff(_) => "diff",
            Self::Status(_) => "status",
            Self::Apply(_) => "apply",
            Self::Query(Query::Refs { .. }) => "query refs",
            Self::Query(Query::Output { .. }) => "query output",
            Self::Query(Query::Value { .. }) => "query value",
            Self::Add { .. } => "add",
            Self::Modify(_) => "modify",
            Self::Forget { .. } => "forget",
            Self::List => "list",
            Self::Schema(_) => "schema",
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct ChoicesArgs {
    /// Block application if any managed output has drifted.
    #[arg(long)]
    strict: bool,
    /// Replace a drifted target at this path. May be repeated.
    #[arg(long, value_name = "PATH")]
    overwrite: Vec<OsString>,
    /// Replace all drifted targets.
    #[arg(long, conflicts_with = "accept_all")]
    overwrite_all: bool,
    /// Accept the drift at this path. May be repeated.
    #[arg(long, value_name = "PATH")]
    accept: Vec<OsString>,
    /// Accept drift in all targets.
    #[arg(long)]
    accept_all: bool,
    /// Allow removal of a modified obsolete target. May be repeated.
    #[arg(long, value_name = "PATH")]
    delete_modified: Vec<OsString>,
    /// Allow removal of all modified obsolete targets.
    #[arg(long)]
    delete_modified_all: bool,
}

impl ChoicesArgs {
    pub(crate) fn is_requested(&self) -> bool {
        self.strict
            || !self.overwrite.is_empty()
            || self.overwrite_all
            || !self.accept.is_empty()
            || self.accept_all
            || !self.delete_modified.is_empty()
            || self.delete_modified_all
    }
}

#[derive(Debug, Args)]
pub(crate) struct InitArgs {
    /// HTTPS or SSH URL of the source repository.
    pub repo: Option<String>,
    /// Set a custom source directory.
    #[arg(long, value_name = "DIRECTORY")]
    source: Option<OsString>,
    /// Set a local tag. May be repeated.
    #[arg(long, value_name = "TAG")]
    tag: Vec<Tag>,
    /// Set a local field value. May be repeated.
    #[arg(long, value_name = "MODULE.FIELD=VALUE")]
    pub set: Vec<String>,
    /// Skip the initial apply.
    #[arg(long, requires = "repo")]
    no_apply: bool,
    #[command(flatten)]
    pub choices: ChoicesArgs,
}

#[derive(Debug, Args)]
#[group(skip)]
#[command(group = clap::ArgGroup::new("properties").required(true).multiple(true).args(["template", "copy", "target", "ownership", "permissions", "takeover", "reset"]))]
pub(crate) struct ModifyArgs {
    /// Name of the output in the manifest.
    #[arg(value_name = "OUTPUT")]
    output: OutputName,
    /// Set the template path relative to the source directory.
    #[arg(long, value_name = "PATH", conflicts_with = "copy")]
    pub template: Option<OsString>,
    /// Set the copy source path relative to the source directory.
    #[arg(long, value_name = "PATH")]
    pub copy: Option<OsString>,
    /// Set a target path or a platform-specific target. May be repeated.
    #[arg(long, value_name = "[PLATFORM=]PATH")]
    pub target: Vec<OsString>,
    /// Set the ownership policy.
    #[arg(long, value_enum)]
    pub ownership: Option<Ownership>,
    /// Set the Unix permissions, for example 0600.
    #[arg(long)]
    pub permissions: Option<String>,
    /// Allow or forbid taking over an existing file.
    #[arg(long, value_parser = ["true", "false"])]
    pub takeover: Option<String>,
    /// Restore a property to its default. May be repeated.
    #[arg(long, value_enum)]
    pub reset: Vec<Property>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Ownership {
    Full,
    CreateOnly,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Property {
    Source,
    Target,
    #[value(name = "target.linux")]
    TargetLinux,
    #[value(name = "target.macos")]
    TargetMacos,
    #[value(name = "target.windows")]
    TargetWindows,
    Ownership,
    Permissions,
    Takeover,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Query {
    /// Find outputs that reference a module or field.
    #[command(after_long_help = help!("query refs --help"))]
    Refs {
        /// Module name or fully qualified field reference.
        #[arg(value_name = "MODULE[.FIELD]")]
        reference: RefQuery,
    },
    /// Describe an output by name or target path.
    #[command(after_long_help = help!("query output --help"))]
    Output {
        /// Output name or target path to look up.
        #[arg(value_name = "NAME_OR_PATH")]
        key: OsString,
    },
    /// Describe the resolved value of a field and its origin.
    #[command(after_long_help = help!("query value --help"))]
    Value {
        /// Fully qualified field reference.
        #[arg(value_name = "MODULE.FIELD")]
        field: FieldRef,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum Schema {
    /// Describe manifest.toml.
    #[command(after_long_help = help!("schema manifest --help"))]
    Manifest,
    /// Describe a module declaration.
    #[command(after_long_help = help!("schema module --help"))]
    Module,
    /// Describe a profile declaration.
    #[command(after_long_help = help!("schema profile --help"))]
    Profile,
    /// Describe local.toml.
    #[command(after_long_help = help!("schema local --help"))]
    Local,
    /// Describe built-in modules, optionally restricted to one module.
    #[command(after_long_help = help!("schema builtin --help"))]
    Builtin {
        /// Restrict the description to this module.
        module: Option<ModuleName>,
    },
}
