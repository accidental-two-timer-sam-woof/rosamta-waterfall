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
//! - `pilot p6-lock try <dir> <name>` takes the lock once and prints
//!   `acquired` or `busy <error>`; `pilot p6-lock hold <dir> <name>` takes it,
//!   prints `held` and waits to be killed.
//! - `pilot p6-external <op> <path> <other>` does one thing another program
//!   might do to `path` and prints `ok` or `blocked <error>`.
//! - `pilot p6-exchange <dir> <a> <b>` exchanges two names forever and
//!   prints `ready` after the first exchange (Unix).

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use rosamta_pilot::{atomic, bound, ops, payload, platform};

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
        ["p6-lock", mode @ ("try" | "hold"), dir, name] => p6_lock(mode, dir, name),
        ["p6-external", op, path, other] => {
            match ops::external(op, path.as_ref(), other.as_ref()) {
                Ok(()) => println!("ok"),
                Err(e) => println!("blocked {}", ops::error_name(&e)),
            }
            ExitCode::SUCCESS
        }
        #[cfg(unix)]
        ["p6-exchange", dir, a, b] => p6_exchange(dir, a, b),
        _ => usage(),
    }
}

fn p6_lock(mode: &str, dir: &str, name: &str) -> ExitCode {
    let dir = match bound::Dir::open_root(dir.as_ref()) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("open {dir}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let lock = match ops::lock(&dir, name.as_ref()) {
        Ok(lock) => lock,
        Err(e) if ops::is_busy(&e) => {
            println!("busy {}", ops::error_name(&e));
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            println!("error {}", ops::error_name(&e));
            return ExitCode::FAILURE;
        }
    };
    if mode == "try" {
        println!("acquired");
        return ExitCode::SUCCESS;
    }
    println!("held");
    let _ = std::io::stdout().flush();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
        let _ = &lock;
    }
}

#[cfg(unix)]
fn p6_exchange(dir: &str, a: &str, b: &str) -> ExitCode {
    let dir = match bound::Dir::open_root(dir.as_ref()) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("open {dir}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut stdout = std::io::stdout();
    for round in 0u64.. {
        if let Err(e) = ops::exchange(&dir, a.as_ref(), b.as_ref()) {
            eprintln!("exchange failed in round {round}: {e}");
            return ExitCode::FAILURE;
        }
        if round == 0 {
            let _ = writeln!(stdout, "ready");
            let _ = stdout.flush();
        }
    }
    unreachable!()
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
         \x20      | pilot p1-churn <swap> <victim> <aside> <other> <stop>\n\
         \x20      | pilot p6-lock try|hold <dir> <name>\n\
         \x20      | pilot p6-external <op> <path> <other>\n\
         \x20      | pilot p6-exchange <dir> <a> <b>"
    );
    ExitCode::from(2)
}
