use clap::Args;
use coda_runtime::{
    frontend::{lexer, parser},
    runtime::{interpreter::Interpreter},
    env::Env,
};
use coda_std::std_loader;
use std::{error::Error, time::Instant};

#[derive(Args)]
pub struct Arguments {
    #[arg(short, long)]
    pub file: String,
}

impl Arguments {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();
        let env = Env::new();

        let source_path = std::path::PathBuf::from(&self.file);
        let base_path = source_path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
        
        let mut interpreter = Interpreter::new(env, base_path, Some(std_loader));

        let source = std::fs::read_to_string(&self.file)?;
        let tokens = lexer::scan(&source)?;
        let ast = parser::parse(tokens)?;

        interpreter.run(ast)?;

        println!("execution time: {:?}", start.elapsed());

        Ok(())
    }
}
