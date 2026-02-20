pub mod repl;
pub mod run;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    repl(repl::Arguments),
    run(run::Arguments),
}

impl Commands {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Commands::repl(args) => args.exec(),
            Commands::run(args) => args.exec(),
        }
    }
}
