use crate::{runtime::{ast::Stmt, interpreter::Interpreter}, env::Env};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Array(Vec<Value>),

    NativeFunction(fn(Vec<Value>) -> Value),
    Function(Function),
}

impl Value {
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            _ => true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Env>>,
}

pub struct CodaFunction {
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Env>>,
}

impl CodaFunction {
    pub fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Value {
        let new_env = Rc::new(RefCell::new(Env::new_with_parent(Some(self.closure.clone()))));

        for (param, arg) in self.params.iter().zip(args) {
            new_env.borrow_mut().define(param.clone(), arg);
        }

        let previous = interpreter.env.clone();

        interpreter.env = new_env;

        for stmt in &self.body {
            let _ = interpreter.execute(stmt.clone());
        }

        interpreter.env = previous;

        Value::Null
    }
}
