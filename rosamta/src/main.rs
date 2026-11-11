mod args;
mod cli;
mod output;
mod run;
mod term;

use std::{io, process::ExitCode};

use clap::{Parser, error::ErrorKind};

fn main() -> ExitCode {
    let cli = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return print_parse_result(&error),
    };

    if let Err(error) = args::validate(&cli.command) {
        return print_parse_result(&error);
    }

    run::run(cli, io::stdout().lock(), io::stderr().lock())
}

fn print_parse_result(error: &clap::Error) -> ExitCode {
    match error.print() {
        Ok(())
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            ExitCode::SUCCESS
        }
        _ => ExitCode::FAILURE,
    }
}
