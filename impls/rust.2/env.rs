use std::{collections::HashMap, rc::Rc};

use crate::{MalError, MalType};

pub struct Env {
    outer: Option<Rc<Env>>,
    data: HashMap<String, MalType>,
}

impl Env {
    pub fn new(outer: Option<Rc<Env>>) -> Self {
        Self {
            outer,
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: MalType) {
        self.data.insert(key, value);
    }

    pub fn find(&self, key: String) -> Result<Rc<&Env>, MalError> {
        if self.data.contains_key(&key) {
            return Ok(Rc::new(&self));
        } else if let Some(outer_env) = &self.outer {
            let env = outer_env;
            let result = env.find(key);
            return result.clone();
        }
        Err(MalError::SymbolNotFound(key))
    }

    pub fn get(&self, key: String) -> Result<MalType, MalError> {
        let env = self.find(key.clone())?;

        let data = &env.data;

        Ok(data.get(&key).unwrap().clone())
    }
}
