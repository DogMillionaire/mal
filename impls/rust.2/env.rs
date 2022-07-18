use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{MalError, MalType};

pub struct Env {
    outer: Option<Rc<RefCell<Env>>>,
    data: HashMap<String, MalType>,
}

impl Env {
    pub fn new(outer: Option<Rc<RefCell<Env>>>) -> Rc<RefCell<Env>> {
        Rc::new(RefCell::new(Env {
            outer,
            data: HashMap::new(),
        }))
    }

    pub fn set(&mut self, key: String, value: MalType) {
        self.data.insert(key, value);
    }

    // fn find(&self, key: String) -> Result<Rc<RefCell<&Env>>, MalError> {
    //     if self.data.contains_key(&key) {
    //         return Ok(Rc::new(RefCell::new(self)));
    //     } else if let Some(outer_env) = &self.outer {
    //         let env = outer_env;
    //         let result = env.borrow().find(key)?;
    //         return Ok(result.clone());
    //     }
    //     Err(MalError::SymbolNotFound(key))
    // }

    pub fn get(&self, key: String) -> Result<MalType, MalError> {
        match self.data.get(&key) {
            Some(v) => Ok(v.clone()),
            None => {
                if let Some(outer) = &self.outer {
                    return outer.borrow().get(key.to_string());
                }
                dbg!(&self.data);
                Err(MalError::SymbolNotFound(key))
            }
        }
    }
}
