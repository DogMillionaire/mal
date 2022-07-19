use std::hash::Hash;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::env::Env;
use crate::reader::MalError;

pub enum MalType {
    Nil,
    List(Vec<Rc<MalType>>),
    Symbol(String),
    Number(isize),
    String(String),
    Vector(Vec<Rc<MalType>>),
    Keyword(String),
    Hashmap(HashMap<Rc<MalType>, Rc<MalType>>),
    Func(MalFunc),
    True,
    False,
}

/// Wrapper for a function
pub struct MalFunc {
    name: String,
    parameters: Vec<Rc<MalType>>,
    body: Box<
        dyn Fn(Rc<RefCell<Env>>, Rc<MalType>, Vec<Rc<MalType>>) -> Result<Rc<MalType>, MalError>,
    >,
    env: Rc<RefCell<Env>>,
    body_ast: Rc<MalType>,
}

impl std::fmt::Debug for MalFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MalFunc")
            .field("name", &self.name)
            .field("parameters", &self.parameters)
            .finish()
    }
}

impl Hash for MalFunc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.parameters.hash(state);
    }
}

impl MalFunc {
    pub fn new(
        name: Option<String>,
        parameters: Vec<Rc<MalType>>,
        body: impl Fn(Rc<RefCell<Env>>, Rc<MalType>, Vec<Rc<MalType>>) -> Result<Rc<MalType>, MalError>
            + 'static,
        env: Rc<RefCell<Env>>,
        body_ast: Rc<MalType>,
    ) -> Self {
        let name = name.unwrap_or(String::from("anonymous"));
        Self {
            name,
            parameters,
            body: Box::new(body),
            env,
            body_ast,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn parameters(&self) -> &[Rc<MalType>] {
        self.parameters.as_ref()
    }

    pub fn body(
        &self,
    ) -> &dyn Fn(Rc<RefCell<Env>>, Rc<MalType>, Vec<Rc<MalType>>) -> Result<Rc<MalType>, MalError>
    {
        self.body.as_ref()
    }

    pub fn env(&self) -> Rc<RefCell<Env>> {
        self.env.clone()
    }

    pub fn body_ast(&self) -> Rc<MalType> {
        self.body_ast.clone()
    }
}

impl MalType {
    pub fn try_into_list(&self) -> Result<Vec<Rc<MalType>>, MalError> {
        match self {
            Self::List(v) => Ok(v.clone()),
            Self::Vector(v) => Ok(v.clone()),
            _ => Err(MalError::InvalidType),
        }
    }

    pub fn try_into_symbol(&self) -> Result<String, MalError> {
        if let Self::Symbol(v) = self {
            Ok(v.to_string())
        } else {
            Err(MalError::InvalidType)
        }
    }

    pub fn try_into_number(&self) -> Result<isize, MalError> {
        if let Self::Number(v) = self {
            Ok(*v)
        } else {
            Err(MalError::InvalidType)
        }
    }

    pub fn try_into_string(&self) -> Result<String, MalError> {
        if let Self::String(v) = self {
            Ok(v.to_string())
        } else {
            Err(MalError::InvalidType)
        }
    }

    pub fn try_into_vector(self) -> Result<Vec<Rc<MalType>>, MalError> {
        if let Self::Vector(v) = self {
            Ok(v)
        } else {
            Err(MalError::InvalidType)
        }
    }

    /// Returns `true` if the mal type is [`List`].
    ///
    /// [`List`]: MalType::List
    #[must_use]
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(..))
    }

    fn compare_as_vec(this: &MalType, other: &MalType) -> bool {
        let (this_vec, other_vec) = match (this, other) {
            (MalType::List(l1), MalType::List(l2)) => (l1, l2),
            (MalType::Vector(l1), MalType::Vector(l2)) => (l1, l2),
            _ => unreachable!(),
        };

        if this_vec.len() != other_vec.len() {
            return false;
        }

        for (this_val, other_val) in this_vec.iter().zip(other_vec.iter()) {
            if this_val != other_val {
                return false;
            }
        }

        return true;
    }
}

impl Eq for MalType {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(_), Self::List(_)) | (Self::Vector(_), Self::Vector(_)) => {
                MalType::compare_as_vec(self, other)
            }
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Keyword(l0), Self::Keyword(r0)) => l0 == r0,
            (Self::Hashmap(l0), Self::Hashmap(r0)) => l0.len() == r0.len(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl std::fmt::Debug for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::List(arg0) => f.debug_tuple("List").field(arg0).finish(),
            Self::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Vector(arg0) => f.debug_tuple("Vector").field(arg0).finish(),
            Self::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
            Self::Hashmap(arg0) => f.debug_tuple("Hashmap").field(arg0).finish(),
            Self::Func(arg0) => f.debug_tuple("Func").field(arg0).finish(),
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
        }
    }
}

impl std::hash::Hash for MalType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            MalType::Nil => core::mem::discriminant(self).hash(state),
            MalType::List(l) => l.hash(state),
            MalType::Symbol(s) => s.hash(state),
            MalType::Number(n) => n.hash(state),
            MalType::String(s) => s.hash(state),
            MalType::Vector(v) => v.hash(state),
            MalType::Keyword(k) => k.hash(state),
            MalType::Hashmap(h) => {
                for entry in h {
                    entry.0.hash(state);
                    entry.1.hash(state);
                }
            }
            MalType::Func(func) => func.hash(state),
            MalType::True => core::mem::discriminant(self).hash(state),
            MalType::False => core::mem::discriminant(self).hash(state),
        }
    }
}
