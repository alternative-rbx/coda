#![allow(non_camel_case_types)]

use clap::Parser;

pub mod subcommands;

use subcommands::Commands;

#[derive(Parser)]
#[command(name = "coda", about = "an experimental scripting language", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        self.command.run()
    }
}
