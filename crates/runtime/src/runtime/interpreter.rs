use crate::{
    frontend::token::TokenKind,
    runtime::{ast::*, env::Env, value::*},
};
use std::{cell::RefCell, collections::HashMap, fmt::Write, fs, rc::Rc};

pub struct Module {
    pub exports: HashMap<String, Value>,
}

pub struct Interpreter {
    pub env: Rc<RefCell<Env>>,
}

pub enum RuntimeControl {
    Return(Value),
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Env::new(None))),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for stmt in statements {
            let _ = self.execute(stmt);
        }
    }

    pub fn execute(&mut self, stmt: Stmt) -> Result<Option<RuntimeControl>, Box<dyn std::error::Error>> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val = self.evaluate(value)?;
                
                self.env.borrow_mut().define(name, val);
                
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

            Stmt::Function { name, params, body, .. } => {
                let function = Value::Function(Function {
                    name: name.clone(),
                    params,
                    body,
                    closure: self.env.clone(),
                });
                
                self.env.borrow_mut().define(name, function);
                
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
        if path.starts_with("std") {
            let modules = crate::std::modules();

            if let Some((_, register_fn)) = modules.iter().find(|(name, _)| *name == path) {
                register_fn(&mut self.env.borrow_mut());
            } else {
                return Err(format!("unknown std module `{path}`").into());
            }
        } else {
            let src = fs::read_to_string(path)?;
            let tokens = crate::frontend::lexer::scan(&src)?;
            let stmts = crate::frontend::parser::parse(tokens)?;

            let mut module_env = Env::new(None);
            
            for stmt in stmts {
                match &stmt {
                    Stmt::Let { name, is_exported, .. } | Stmt::Function { name, is_exported, .. } => {
                        if *is_exported {
                            self.execute_stmt_into(&stmt, &mut module_env)?;
                        } else {
                            self.execute_stmt_into(&stmt, &mut module_env)?;
                        }
                    }
                    
                    _ => {
                        self.execute_stmt_into(&stmt, &mut module_env)?;
                    }
                }
            }
            
            for (name, val) in module_env.values.into_iter() {
                self.env.borrow_mut().define(name, val);
            }
        }

        Ok(())
    }
    
    fn execute_stmt_into(&mut self, stmt: &Stmt, env: &mut Env) -> Result<(), Box<dyn std::error::Error>> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val = self.evaluate_into(value.clone(), env)?;
                
                env.define(name.clone(), val);
            }
            
            Stmt::Function { name, params, body, .. } => {
                let func = Value::Function(Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: Rc::new(RefCell::new(env.clone())),
                });
                
                env.define(name.clone(), func);
            }
            
            _ => {}
        }

        Ok(())
    }
    
    fn evaluate_into(&mut self, expr: Expr, env: &mut Env) -> Result<Value, Box<dyn std::error::Error>> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                ValueLiteral::Number(n) => Value::Number(n),
                ValueLiteral::String(s) => Value::String(s),
                ValueLiteral::Bool(b) => Value::Bool(b),
                ValueLiteral::Null => Value::Null,
            }),
            
            Expr::Binary { left, operator, right } => {
                let l = self.evaluate_into(*left, env)?;
                let r = self.evaluate_into(*right, env)?;
                
                unimplemented!()
            }
            
            Expr::Variable(name) => Ok(env.get(&name).ok_or_else(|| format!("undefined variable `{name}`"))?),
            
            _ => unimplemented!(),
        }
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
                        let call_env = Rc::new(RefCell::new(Env::new(Some(func.closure.clone()))));
                        
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

    pub fn run(&mut self, statements: Vec<Stmt>) {
        self.interpret(statements);
    }
}
