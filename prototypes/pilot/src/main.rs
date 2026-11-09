//! `pilot` binary: the child process the kill tests terminate, plus a smoke
//! command for the release-build checks.
//!
//! - `pilot env` prints the platform and the identity the process runs as.
//! - `pilot p2-write <target> <size> <method>` replaces `target` forever,
//!   alternating payloads `A` and `B`, and prints `ready` after the first
//!   complete replacement.
//! - `pilot p1-swap <swap> <victim> <aside> <other>` replaces a directory
//!   once and prints `swapped` or `blocked <error>`.
//! - `pilot p1-churn <swap> <victim> <aside> <other> <stop>` keeps replacing
//!   a directory until `stop` exists, then prints the counts.

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use rosamta_pilot::{atomic, bound, payload, platform};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["env"] => {
            let identity = platform::identity().unwrap_or_else(|e| format!("error={e}"));
            println!(
                "os={} arch={} {identity}",
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            ExitCode::SUCCESS
        }
        ["p2-write", target, size, method] => {
            let (Ok(size), Some(method)) = (size.parse(), atomic::Method::parse(method)) else {
                return usage();
            };
            p2_write(PathBuf::from(target), size, method)
        }
        ["p1-swap", swap, victim, aside, other] => {
            let Some(swap) = bound::Swap::parse(swap) else {
                return usage();
            };
            match swap.apply(victim.as_ref(), aside.as_ref(), other.as_ref()) {
                Ok(()) => println!("swapped"),
                Err(e) => println!("blocked {e}"),
            }
            ExitCode::SUCCESS
        }
        ["p1-churn", swap, victim, aside, other, stop] => {
            let Some(swap) = bound::Swap::parse(swap) else {
                return usage();
            };
            let (cycles, blocked) = bound::churn(
                swap,
                victim.as_ref(),
                aside.as_ref(),
                other.as_ref(),
                stop.as_ref(),
            );
            println!("cycles={cycles} blocked={blocked}");
            ExitCode::SUCCESS
        }
        _ => usage(),
    }
}

fn p2_write(target: PathBuf, size: usize, method: atomic::Method) -> ExitCode {
    let contents = [payload::build(b'A', size), payload::build(b'B', size)];
    let mut stdout = std::io::stdout();
    for round in 0usize.. {
        if let Err(e) = atomic::replace(&target, &contents[round % 2], method) {
            eprintln!("replace failed in round {round}: {e}");
            return ExitCode::FAILURE;
        }
        if round == 0 {
            let _ = writeln!(stdout, "ready");
            let _ = stdout.flush();
        }
    }
    unreachable!()
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: pilot env | pilot p2-write <target> <size> <method>\n\
         \x20      | pilot p1-swap <swap> <victim> <aside> <other>\n\
         \x20      | pilot p1-churn <swap> <victim> <aside> <other> <stop>"
    );
    ExitCode::from(2)
}
