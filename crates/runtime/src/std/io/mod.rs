use crate::runtime::{env::Env, value::Value};

pub fn register(env: &mut Env) {
    env.define(
        "print".to_string(),
        Value::NativeFunction(|args: Vec<Value>| {
            fn print_value(v: &Value) {
                match v {
                    Value::Number(n) => print!("{n}"),
                    Value::String(s) => print!("{s}"),
                    Value::Bool(b) => print!("{b}"),
                    Value::Null => print!("null"),

                    Value::Array(arr) => {
                        print!("[");

                        for (i, item) in arr.iter().enumerate() {
                            if i > 0 {
                                print!(", ");
                            }

                            print_value(item);
                        }

                        print!("]");
                    }

                    Value::Function(_) | Value::NativeFunction(_) => print!("<fn>"),
                }
            }

            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }

                print_value(a);
            }

            println!();

            Value::Null
        }),
    );
}
