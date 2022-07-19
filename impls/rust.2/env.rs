use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{MalError, MalType};

pub struct Env {
    outer: Option<Rc<RefCell<Env>>>,
    data: HashMap<String, Rc<MalType>>,
}

pub type MalEnv = Rc<RefCell<Env>>;

impl Env {
    pub fn new(
        bindings: Option<Vec<Rc<MalType>>>,
        exprs: Option<Vec<Rc<MalType>>>,
        outer: Option<MalEnv>,
    ) -> MalEnv {
        let mut env = Env {
            outer,
            data: HashMap::new(),
        };

        match (bindings, exprs) {
            (Some(b), Some(e)) => {
                for (binding, expression) in b.iter().zip(e.iter()) {
                    env.set(binding.try_into_symbol().unwrap(), expression.clone());
                }
            }
            (None, Some(_)) | (Some(_), None) => {
                panic!("Must pass both bindings and expressions");
            }
            (None, None) => { /*No-op*/ }
        }

        Rc::new(RefCell::new(env))
    }

    pub fn set(&mut self, key: String, value: Rc<MalType>) {
        self.data.insert(key, value.clone());
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

    pub fn get(&self, key: String) -> Result<Rc<MalType>, MalError> {
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
