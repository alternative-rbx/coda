use clap::Args;
use coda_runtime::{
    frontend::{lexer, parser},
    runtime::interpreter::Interpreter,
};
use rustyline::{Editor, error::ReadlineError, history::DefaultHistory};

#[derive(Args)]
pub struct Arguments {}

impl Arguments {
    pub fn exec(self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rl = Editor::<(), DefaultHistory>::new()?;
        let mut interpreter = Interpreter::new();

        println!("coda repl (type ctrl+d to exit)");

        let mut buffer = String::new();

        loop {
            let prompt = if buffer.is_empty() { "> " } else { "... " };
            let readline = rl.readline(prompt);

            match readline {
                Ok(line) => {
                    if !buffer.is_empty() {
                        buffer.push('\n');
                    }
                    
                    buffer.push_str(&line);
                    
                    match lexer::scan(&buffer).and_then(|tokens| parser::parse(tokens)) {
                        Ok(ast) => {
                            interpreter.run(ast);
                            rl.add_history_entry(buffer.trim())?;
                            buffer.clear();
                        }
                        Err(_) => {
                            continue;
                        }
                    }
                }

                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    
                    buffer.clear();
                }

                Err(ReadlineError::Eof) => {
                    println!();
                    
                    break;
                }

                Err(err) => {
                    eprintln!("error: {err}");
                    
                    break;
                }
            }
        }

        Ok(())
    }
}