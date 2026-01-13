use std::process::ExitCode;

use clap::Parser;
use akarusa::{current_luminance, set_luminance};

#[derive(Parser)]
#[command(about, version)]
struct Cli {
    /// Display luminance, from 0 to 100. If omitted, prints the current luminance.
    #[arg(value_name = "0-100")]
    value: Option<u8>,
}

fn main() -> ExitCode {
    let result = match Cli::parse().value {
        Some(value) => set_luminance(value),
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
