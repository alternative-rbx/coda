use crate::runtime::env::Env;

pub mod consts;

pub fn register(env: &mut Env) {
    consts::register(env);
}
