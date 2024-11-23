use clap::Parser;
use set_luminance::{DisplayService, LuminancePacket};

#[derive(Debug, Parser)]
#[clap(version, about)]
pub struct Args {
    /// Display luminance, from 0 to 100.
    #[arg()]
    pub value: u8,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    DisplayService::try_new()?.set_luminance(&LuminancePacket::from(args.value))
}
