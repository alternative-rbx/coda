use crate::runtime::value::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Debug)]
pub struct Env {
    pub values: HashMap<String, Value>,
    pub parent: Option<Rc<RefCell<Env>>>,
}

impl Env {
    #[inline(always)]
    pub fn new(parent: Option<Rc<RefCell<Env>>>) -> Self {
        Self { values: HashMap::new(), parent }
    }

    pub fn assign(&mut self, name: &str, val: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), val);
            
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign(name, val)
        } else {
            Err(format!("undefined variable `{name}`"))
        }
    }

    #[inline(always)]
    pub fn define(&mut self, name: String, val: Value) {
        self.values.insert(name, val);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.values.get(name) {
            Some(v.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }
}
