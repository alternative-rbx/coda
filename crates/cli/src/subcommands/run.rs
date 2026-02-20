use clap::Args;
use coda_runtime::{
    frontend::{lexer, parser},
    runtime::interpreter::Interpreter,
};
use std::{error::Error, time::Instant};

#[derive(Args)]
pub struct Arguments {
    #[arg(short, long)]
    pub file: String,
}

impl Arguments {
    pub fn exec(self) -> Result<(), Box<dyn Error>> {
        let start = Instant::now();

        let mut interpreter = Interpreter::new();
        let source = std::fs::read_to_string(&self.file)?;

        let tokens = lexer::scan(&source)?;
        let ast = parser::parse(tokens)?;

        interpreter.run(ast);

        let end = start.elapsed();

        println!("execution time: {:?}", end);

        Ok(())
    }
}
