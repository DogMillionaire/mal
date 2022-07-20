use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{MalError, MalType};

#[allow(dead_code)]
pub struct Env {
    outer: Option<Rc<RefCell<Env>>>,
    data: HashMap<String, Rc<MalType>>,
}

#[allow(dead_code)]
pub type MalEnv = Rc<RefCell<Env>>;

#[allow(dead_code)]
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
                for i in 0..b.len() {
                    let binding_symbol = b[i].try_into_symbol().unwrap();
                    if binding_symbol == "&" {
                        let remaining_exprs: Vec<_> = e[i..e.len()].to_vec();

                        env.set(
                            b[i + 1].try_into_symbol().unwrap(),
                            Rc::new(MalType::List(remaining_exprs)),
                        );
                        break;
                    } else {
                        env.set(binding_symbol, e[i].clone());
                    }
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

    pub fn get(&self, key: String) -> Result<Rc<MalType>, MalError> {
        match self.data.get(&key) {
            Some(v) => Ok(v.clone()),
            None => {
                if let Some(outer) = &self.outer {
                    return outer.borrow().get(key.to_string());
                }
                Err(MalError::SymbolNotFound(key))
            }
        }
    }
}
