use anyhow::Result;
use clap::Parser;

use tmbliss::{Args, TMBliss};

fn main() -> Result<()> {
    let args = Args::parse();
    let command = args.command;

    TMBliss::run(command)?;
    Ok(())
}
