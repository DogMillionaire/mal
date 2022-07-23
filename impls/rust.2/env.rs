use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::malerror::MalError;
use crate::MalType;

#[allow(dead_code)]
pub struct Env {
    outer: Option<Rc<RefCell<Env>>>,
    data: HashMap<String, Rc<MalType>>,
    root: Option<Rc<RefCell<Env>>>,
}

#[allow(dead_code)]
pub type MalEnv = Rc<RefCell<Env>>;

impl std::fmt::Debug for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Env")
            .field("outer", &self.outer)
            .field("data", &self.data)
            .finish()
    }
}

#[allow(dead_code)]
impl Env {
    pub fn new_root(bindings: Option<Vec<Rc<MalType>>>, exprs: Option<Vec<Rc<MalType>>>) -> MalEnv {
        let new_env = Self::new(bindings, exprs, None);

        new_env.borrow_mut().set_root(Some(new_env.clone()));
        new_env
    }

    pub fn new_with_outer(
        bindings: Option<Vec<Rc<MalType>>>,
        exprs: Option<Vec<Rc<MalType>>>,
        outer: MalEnv,
    ) -> MalEnv {
        let new_env = Self::new(bindings, exprs, Some(outer.clone()));

        new_env.borrow_mut().set_root(outer.borrow().get_root());
        new_env
    }

    fn new(
        bindings: Option<Vec<Rc<MalType>>>,
        exprs: Option<Vec<Rc<MalType>>>,
        outer: Option<MalEnv>,
    ) -> MalEnv {
        let mut env = Env {
            outer,
            data: HashMap::new(),
            root: None,
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

    pub fn set_root(&mut self, root: Option<MalEnv>) {
        self.root = root;
    }

    pub fn get_root(&self) -> Option<MalEnv> {
        self.root.clone()
    }

    pub fn set(&mut self, key: String, value: Rc<MalType>) {
        self.data.insert(key, value);
    }

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
