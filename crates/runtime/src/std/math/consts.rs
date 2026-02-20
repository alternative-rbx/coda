use crate::runtime::{env::Env, value::Value};

pub fn register(env: &mut Env) {
    env.define("pi".to_string(), Value::Number(std::f64::consts::PI));
    env.define("e".to_string(), Value::Number(std::f64::consts::E));

    env.define(
        "sqrt".to_string(),
        Value::NativeFunction(|args| if let Some(Value::Number(n)) = args.get(0) { Value::Number(n.sqrt()) } else { Value::Null }),
    );
}
