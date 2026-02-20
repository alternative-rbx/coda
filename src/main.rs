#![allow(non_camel_case_types, irrefutable_let_patterns)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use coda_cli::Cli;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if let Err(err) = cli.run() {
        eprintln!("error: {err}");

        std::process::exit(1);
    }

    Ok(())
}
