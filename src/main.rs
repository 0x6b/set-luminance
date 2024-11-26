use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use log::info;
use set_luminance::DisplayService;

#[derive(Debug, Parser)]
#[clap(version)]
pub struct Args {
    /// Display luminance, from 0 to 100. If not provided, the current luminance is displayed.
    #[arg()]
    pub value: Option<u8>,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let service = DisplayService::try_new()?;

    match args.value {
        Some(value) => service.set_luminance(value)?,
        None => {
            info!("Current luminance: {}", service.get_luminance()?)
        }
    }

    Ok(())
}
