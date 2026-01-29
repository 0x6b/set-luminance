use std::{env, env::args, process::ExitCode};

use akarusa::{current_luminance, set_luminance};

const HELP: &str = "\
Controls luminance (brightness) of an external display over DDC

Usage: akarusa [VALUE]

Arguments:
  [VALUE]  Luminance level, 0-100. If omitted, prints current value.

Options:
  -h, --help     Print help
  -V, --version  Print version";

fn main() -> ExitCode {
    let result = match args().nth(1).as_deref() {
        Some("-h" | "--help") => {
            return {
                println!("{HELP}");
                ExitCode::SUCCESS
            };
        }
        Some("-V" | "--version") => {
            return {
                println!("{}", env!("CARGO_PKG_VERSION"));
                ExitCode::SUCCESS
            };
        }
        Some(v) => v
            .parse()
            .map_err(|_| format!("invalid value: {v}"))
            .and_then(set_luminance),
        None => current_luminance().map(|v| println!("{v}")),
    };

    result.map_or_else(
        |e| {
            eprintln!("{e}");
            ExitCode::FAILURE
        },
        |_| ExitCode::SUCCESS,
    )
}
