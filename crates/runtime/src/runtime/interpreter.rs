use crate::{
    frontend::token::TokenKind,
    runtime::{ast::*, value::*},
    env::Env,
};
use std::{cell::RefCell, collections::{HashMap, HashSet}, fmt::Write, rc::Rc};

pub type ModuleLoader = fn(&str, &mut Env) -> Result<bool, Box<dyn std::error::Error>>;

pub struct Module {
    pub exports: HashMap<String, Value>,
}

pub struct Interpreter {
    pub env: Rc<RefCell<Env>>,
    pub base_path: std::path::PathBuf,
    pub module_loader: Option<ModuleLoader>,
    pub loaded_modules: HashSet<String>,
}

pub enum RuntimeControl {
    Return(Value),
}

impl Interpreter {
    pub fn new(
        env: Env,
        base_path: std::path::PathBuf,
        module_loader: Option<ModuleLoader>,
    ) -> Self {
        Self {
            env: Rc::new(RefCell::new(env)),
            base_path,
            module_loader,
            loaded_modules: HashSet::new(),
        }
    }

    pub fn interpret(
        &mut self,
        statements: Vec<Stmt>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    pub fn execute(&mut self, stmt: Stmt) -> Result<Option<RuntimeControl>, Box<dyn std::error::Error>> {
        match stmt {
            Stmt::Let { name, value, is_exported, .. } => {
                let val = self.evaluate(value)?;

                if is_exported {
                    self.env.borrow_mut().define_export(name, val);
                } else {
                    self.env.borrow_mut().define(name, val);
                }

                Ok(None)
            }

            Stmt::Expr(expr) => {
                let _ = self.evaluate(expr)?;
                Ok(None)
            }

            Stmt::Block(statements) => {
                self.execute_block(statements, None)?;

                Ok(None)
            }

            Stmt::Function { name, params, body, is_exported, .. } => {
                let function = Value::Function(Function {
                    name: name.clone(),
                    params,
                    body,
                    closure: self.env.clone(),
                });

                if is_exported {
                    self.env.borrow_mut().define_export(name, function);
                } else {
                    self.env.borrow_mut().define(name, function);
                }

                Ok(None)
            }

            Stmt::Return(expr) => {
                let value = match expr {
                    Some(e) => self.evaluate(e)?,
                    None => Value::Null,
                };

                Ok(Some(RuntimeControl::Return(value)))
            }

            Stmt::If { condition, then_branch, else_branch } => {
                let cond = self.evaluate(condition)?;
                let cond_bool = cond.as_bool();

                if cond_bool {
                    self.execute_block(then_branch, None)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute_block(else_branch, None)?;
                }

                Ok(None)
            }

            Stmt::While { condition, body } => {
                while self.evaluate(condition.clone())?.as_bool() {
                    if let Some(ctrl) = self.execute_block(body.clone(), None)? {
                        if let RuntimeControl::Return(_) = ctrl {
                            return Ok(Some(ctrl));
                        }
                    }
                }

                Ok(None)
            }

            Stmt::Import(module_path) => {
                self.execute_import(&module_path)?;

                Ok(None)
            }
        }
    }

    fn execute_block(&mut self, statements: Vec<Stmt>, env: Option<Rc<RefCell<Env>>>) -> Result<Option<RuntimeControl>, Box<dyn std::error::Error>> {
        let previous = self.env.clone();
        let env_to_use = env.unwrap_or_else(|| previous.clone());

        self.env = env_to_use;

        for stmt in statements {
            if let Some(RuntimeControl::Return(val)) = self.execute(stmt)? {
                self.env = previous;

                return Ok(Some(RuntimeControl::Return(val)));
            }
        }

        self.env = previous;

        Ok(None)
    }

    fn execute_import(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Prevent duplicate imports
        if self.loaded_modules.contains(path) {
            return Ok(());
        }

        // Try external module loader (std etc.)
        if let Some(loader) = self.module_loader {
            let handled = loader(path, &mut self.env.borrow_mut())?;
            if handled {
                self.loaded_modules.insert(path.to_string());
                return Ok(());
            }
        }

        // Fallback: user file import
        let full_path = if path.starts_with("./") || path.starts_with("../") {
            self.base_path.join(path)
        } else {
            std::path::PathBuf::from(path)
        };

        let src = std::fs::read_to_string(&full_path)?;

        let tokens = crate::frontend::lexer::scan(&src)?;
        let stmts = crate::frontend::parser::parse(tokens)?;

        let module_env = Rc::new(RefCell::new(Env::new_with_parent(None)));

        for stmt in stmts {
            if let Some(ctrl) =
                self.execute_block(vec![stmt], Some(module_env.clone()))?
            {
                if let RuntimeControl::Return(_) = ctrl {
                    break;
                }
            }
        }

        let values = module_env.borrow().values.clone();

        for (name, val) in values {
            self.env.borrow_mut().define(name, val);
        }

        self.loaded_modules.insert(path.to_string());

        Ok(())
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Value, Box<dyn std::error::Error>> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                ValueLiteral::Number(n) => Value::Number(n),
                ValueLiteral::String(s) => Value::String(s),
                ValueLiteral::Bool(b) => Value::Bool(b),
                ValueLiteral::Null => Value::Null,
            }),

            Expr::Variable(name) => Ok(self.env.borrow().get(&name).ok_or_else(|| format!("undefined variable `{name}`"))?),

            Expr::Binary { left, operator, right } => {
                let l = self.evaluate(*left)?;
                let r = self.evaluate(*right)?;

                match (&l, &r, operator.clone()) {
                    // numbers
                    (Value::Number(a), Value::Number(b), TokenKind::Plus) => Ok(Value::Number(a + b)),
                    (Value::Number(a), Value::Number(b), TokenKind::Minus) => Ok(Value::Number(a - b)),
                    (Value::Number(a), Value::Number(b), TokenKind::Star) => Ok(Value::Number(a * b)),
                    (Value::Number(a), Value::Number(b), TokenKind::Slash) => Ok(Value::Number(a / b)),

                    // compound assignment
                    (Value::Number(a), Value::Number(b), TokenKind::PlusEqual) => Ok(Value::Number(a + b)),
                    (Value::Number(a), Value::Number(b), TokenKind::MinusEqual) => Ok(Value::Number(a - b)),
                    (Value::Number(a), Value::Number(b), TokenKind::StarEqual) => Ok(Value::Number(a * b)),
                    (Value::Number(a), Value::Number(b), TokenKind::SlashEqual) => Ok(Value::Number(a / b)),

                    // strings
                    (Value::String(a), Value::String(b), TokenKind::Plus) => {
                        let mut s = String::with_capacity(a.len() + b.len());

                        s.push_str(a);
                        s.push_str(b);

                        Ok(Value::String(s))
                    }

                    (Value::String(a), Value::Number(b), TokenKind::Plus) => {
                        let mut s = String::with_capacity(a.len() + 16);

                        s.push_str(a);
                        write!(&mut s, "{}", b).unwrap();

                        Ok(Value::String(s))
                    }

                    (Value::Number(a), Value::String(b), TokenKind::Plus) => {
                        let mut s = String::with_capacity(a.to_string().len() + b.len());

                        s.push_str(a.to_string().as_str());
                        s.push_str(b);

                        Ok(Value::String(s))
                    }

                    // comparison operators
                    (Value::Number(a), Value::Number(b), TokenKind::Greater) => Ok(Value::Bool(a > b)),
                    (Value::Number(a), Value::Number(b), TokenKind::GreaterEqual) => Ok(Value::Bool(a >= b)),
                    (Value::Number(a), Value::Number(b), TokenKind::Less) => Ok(Value::Bool(a < b)),
                    (Value::Number(a), Value::Number(b), TokenKind::LessEqual) => Ok(Value::Bool(a <= b)),
                    (Value::Number(a), Value::Number(b), TokenKind::EqualEqual) => Ok(Value::Bool(a == b)),
                    (Value::Number(a), Value::Number(b), TokenKind::BangEqual) => Ok(Value::Bool(a != b)),
                    (Value::String(a), Value::String(b), TokenKind::EqualEqual) => Ok(Value::Bool(a == b)),

                    _ => Err(format!("unsupported operation: {l:?} {operator:?} {r:?}").into()),
                }
            }

            Expr::Call { callee, args } => {
                let callee_val = self.evaluate(*callee)?;
                let mut evaluated_args = Vec::with_capacity(args.len());

                for arg in args {
                    evaluated_args.push(self.evaluate(arg)?);
                }

                match callee_val {
                    Value::NativeFunction(f) => Ok(f(evaluated_args)),
                    Value::Function(func) => {
                        let call_env = Rc::new(RefCell::new(Env::new_with_parent(Some(func.closure.clone()))));

                        for (param, arg) in func.params.iter().zip(evaluated_args.into_iter()) {
                            call_env.borrow_mut().define(param.clone(), arg);
                        }

                        let result = match self.execute_block(func.body.clone(), Some(call_env))? {
                            Some(RuntimeControl::Return(val)) => val,
                            None => Value::Null,
                        };

                        Ok(result)
                    }

                    _ => panic!("can only call functions"),
                }
            }

            Expr::Assign { name, value } => {
                let val = self.evaluate(*value)?;

                self.env.borrow_mut().assign(&name, val)?;

                Ok(Value::Null)
            }

            Expr::Array(elements) => {
                let mut values = Vec::with_capacity(elements.len());

                for el in elements {
                    values.push(self.evaluate(el)?);
                }

                Ok(Value::Array(values))
            }

            Expr::Function { name, params, body } => {
                let func = Value::Function(Function {
                    name: name.clone(),
                    params,
                    body,
                    closure: self.env.clone(),
                });

                Ok(func)
            }

            expr => panic!("unimplemented expr {expr:?}"),
        }
    }

    pub fn run(
        &mut self,
        statements: Vec<Stmt>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.interpret(statements)
    }
}
