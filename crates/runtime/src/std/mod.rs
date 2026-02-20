use crate::runtime::env::Env;

pub mod io;
pub mod math;

pub type StdRegister = fn(&mut Env);

pub fn modules() -> Vec<(&'static str, StdRegister)> {
    vec![("std", register), ("std.math", math::register), ("std.io", io::register)]
}

pub fn register(env: &mut Env) {
    math::register(env);
    io::register(env);
}
