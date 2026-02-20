# coda
coda is an experimental interpreted scripting language.

> [!WARNING]
> coda is a work in progress, and is not yet ready for production use. coda is unstable and not battle-tested.
> i am also not good at rust in the slightest. here be dragons!

## usage
> [!TIP]
> run `cargo run -- <command> --help` to see a detailed view on any of the commands.
> you can also run `cargo run -- --help` to see a detailed view on all commands.

run coda by using `cargo run -- [args]`. you can either use `cargo run -- run --file <file_path>` or `cargo run -- repl`.

## embedding
the *coda runtime* has **no dependencies**, meaning you can use it in any project without worrying about compatibility issues.

embedding the *coda runtime* into your own projects is as easy and simple as:
- *creating an interpreter* using `coda_runtime::runtime::interpreter::Interpreter::new`
- *getting the source code* in any way you want
- *scanning the source code* into tokens using `coda_runtime::frontend::lexer::scan`
- *parsing the tokens* into an *ast* using `coda_runtime::frontend::parser::parse`
- *running the ast* using `coda_runtime::runtime::interpreter::Interpreter::run`

this code sample is directly taken from the cli `run` subcommand!
```rust
use coda_runtime::{
    frontend::{lexer, parser},
    runtime::interpreter::Interpreter,
};

let mut interpreter = Interpreter::new();

let file = "examples/test.coda";
let source = std::fs::read_to_string(&file)?;

let tokens = lexer::scan(&source)?;
let ast = parser::parse(tokens)?;

interpreter.run(ast);
```

## features
- let/const variables
- importing from **standard library** and other files
- string addition
- compound assignment
- functions
  - anonymous functions
  - closures
- if statements
- while loops
- arrays
  - nested arrays

## standard library
### math
- **constants**
  - *pi*
  - *e*
- **functions**
  - *sqrt* - gets the square root of a number
### io
- **functions**
  - *print* - prints a string to the console