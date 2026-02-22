# coda-runtime
coda-runtime is the script interpreter for coda, an experimental scripting language.

you can see more information about coda in the [github repository](https://github.com/alternative-rbx/coda).

## embedding
> [!TIP]
> the coda runtime has zero dependencies, meaning you can use it in any project without worrying about compatibility issues.

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
    runtime::{interpreter::Interpreter},
    env::Env,
};
use coda_std::std_loader;

let env = Env::new();

let source_path = std::path::PathBuf::from(&self.file);
let base_path = source_path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();

let mut interpreter = Interpreter::new(env, base_path, Some(std_loader));

let source = std::fs::read_to_string(&self.file)?;
let tokens = lexer::scan(&source)?;
let ast = parser::parse(tokens)?;

interpreter.run(ast)?;
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