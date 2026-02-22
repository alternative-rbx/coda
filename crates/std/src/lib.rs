use coda_runtime::env::Env;

pub mod io;
pub mod math;

pub type StdRegisterFn = fn(&mut Env);

pub fn get_module(path: &str) -> Option<StdRegisterFn> {
    match path {
        "std.math" => Some(math::register),
        "std.io" => Some(io::register),
        _ => None,
    }
}

pub fn std_loader(
    path: &str,
    env: &mut Env,
) -> Result<bool, Box<dyn std::error::Error>> {
    if path.starts_with("std.") {
        match path {
            "std.math" => {
                math::register(env);

                return Ok(true);
            }

            "std.io" => {
                io::register(env);

                return Ok(true);
            }

            _ => {
                return Err(format!("unknown std module `{path}`").into());
            }
        }
    }

    Ok(false)
}
