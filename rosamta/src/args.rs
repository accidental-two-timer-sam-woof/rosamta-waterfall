use clap::{CommandFactory, error::ErrorKind};
use rosamta_core::name::FieldRef;

use crate::cli::{Cli, Command, InitArgs, ModifyArgs, Property};

pub(crate) fn validate(command: &Command) -> Result<(), clap::Error> {
    let result = match command {
        Command::Init(args) => validate_init(args),
        Command::Modify(args) => validate_modify(args),
        _ => Ok(()),
    };
    result.map_err(|message| Cli::command().error(ErrorKind::InvalidValue, message))
}

fn validate_init(args: &InitArgs) -> Result<(), String> {
    if args.repo.is_none() && args.choices.is_requested() {
        return Err("drift handling options require a repository for init".into());
    }

    for assignment in &args.set {
        let valid_field = assignment
            .split_once('=')
            .is_some_and(|(field, _)| field.parse::<FieldRef>().is_ok());
        if !valid_field {
            return Err("expected --set MODULE.FIELD=VALUE".into());
        }
    }
    Ok(())
}

fn validate_modify(args: &ModifyArgs) -> Result<(), String> {
    for property in &args.reset {
        let is_set = match property {
            Property::Source => args.template.is_some() || args.copy.is_some(),
            Property::Target => args
                .target
                .iter()
                .any(|value| target_platform(value).is_none()),
            Property::TargetLinux => has_platform_target(args, "linux"),
            Property::TargetMacos => has_platform_target(args, "macos"),
            Property::TargetWindows => has_platform_target(args, "windows"),
            Property::Ownership => args.ownership.is_some(),
            Property::Permissions => args.permissions.is_some(),
            Property::Takeover => args.takeover.is_some(),
        };
        if is_set {
            let name = clap::ValueEnum::to_possible_value(property)
                .map(|value| value.get_name().to_owned())
                .unwrap_or_default();
            return Err(format!("cannot set and reset {name} in the same command"));
        }
    }
    Ok(())
}

fn has_platform_target(args: &ModifyArgs, platform: &str) -> bool {
    args.target
        .iter()
        .any(|value| target_platform(value) == Some(platform))
}

fn target_platform(value: &std::ffi::OsStr) -> Option<&'static str> {
    let bytes = value.as_encoded_bytes();
    let separator = bytes.iter().position(|&byte| byte == b'=')?;
    match &bytes[..separator] {
        b"linux" => Some("linux"),
        b"macos" => Some("macos"),
        b"windows" => Some("windows"),
        _ => None,
    }
}
