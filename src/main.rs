use clap::Parser;
use set_luminance::{current_luminance, set_luminance};
use std::process::ExitCode;

#[derive(Parser)]
#[command(about, version)]
struct Cli {
    /// Display luminance, from 0 to 100. If omitted, prints the current luminance.
    #[arg(value_name = "0-100")]
    value: Option<u16>,
}

enum Command {
    Get,
    Set(u16),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let command = match cli.value {
        Some(value) => Command::Set(value.min(100)),
        None => Command::Get,
    };

    run(command).map_or_else(
        |message| {
            eprintln!("{message}");
            ExitCode::FAILURE
        },
        |_| ExitCode::SUCCESS,
    )
}

fn run(command: Command) -> Result<(), String> {
    match command {
        Command::Get => current_luminance().map(|value| println!("{value}")),
        Command::Set(value) => set_luminance(value),
    }
}
