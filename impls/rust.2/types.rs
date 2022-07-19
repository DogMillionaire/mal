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
    Atom(RefCell<Rc<MalType>>),
}

/// Wrapper for a function
pub struct MalFunc {
    name: String,
    parameters: Vec<Rc<MalType>>,
    body: Option<Box<MalFn>>,
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

pub type MalFn = dyn Fn(
    Rc<RefCell<Env>>,
    Rc<MalType>,
    Vec<Rc<MalType>>,
    Vec<Rc<MalType>>,
) -> Result<Rc<MalType>, MalError>;

impl MalFunc {
    pub fn new(
        name: Option<String>,
        parameters: Vec<Rc<MalType>>,
        env: Rc<RefCell<Env>>,
        body_ast: Rc<MalType>,
    ) -> Self {
        let name = name.unwrap_or(String::from("anonymous"));
        Self {
            name,
            parameters,
            body: None,
            env,
            body_ast,
        }
    }
    pub fn new_with_closure(
        name: Option<String>,
        parameters: Vec<Rc<MalType>>,
        body: impl Fn(
                Rc<RefCell<Env>>,
                Rc<MalType>,
                Vec<Rc<MalType>>,
                Vec<Rc<MalType>>,
            ) -> Result<Rc<MalType>, MalError>
            + 'static,
        env: Rc<RefCell<Env>>,
        body_ast: Rc<MalType>,
    ) -> Self {
        let name = name.unwrap_or(String::from("anonymous"));
        Self {
            name,
            parameters,
            body: Some(Box::new(body)),
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

    pub fn env(&self) -> Rc<RefCell<Env>> {
        self.env.clone()
    }

    pub fn body_ast(&self) -> Rc<MalType> {
        self.body_ast.clone()
    }

    pub fn body(&self) -> Option<&Box<MalFn>> {
        self.body.as_ref()
    }
}

#[allow(dead_code)]
impl MalType {
    pub fn type_name(&self) -> String {
        match self {
            MalType::Nil => String::from("MalType::Nil"),
            MalType::List(_) => String::from("MalType::List"),
            MalType::Symbol(_) => String::from("MalType::Symbol"),
            MalType::Number(_) => String::from("MalType::Number"),
            MalType::String(_) => String::from("MalType::String"),
            MalType::Vector(_) => String::from("MalType::Vector"),
            MalType::Keyword(_) => String::from("MalType::Keyword"),
            MalType::Hashmap(_) => String::from("MalType::Hashmap"),
            MalType::Func(_) => String::from("MalType::Func"),
            MalType::True => String::from("MalType::True"),
            MalType::False => String::from("MalType::False"),
            MalType::Atom(_) => String::from("MalType::Atom"),
        }
    }

    pub fn try_into_list(&self) -> Result<Vec<Rc<MalType>>, MalError> {
        match self {
            Self::List(v) => Ok(v.clone()),
            Self::Vector(v) => Ok(v.clone()),
            _ => Err(MalError::InvalidType(
                String::from("MalType::List"),
                self.type_name(),
            )),
        }
    }

    pub fn try_into_symbol(&self) -> Result<String, MalError> {
        if let Self::Symbol(v) = self {
            Ok(v.to_string())
        } else {
            Err(MalError::InvalidType(
                String::from("MalType::Symbol"),
                self.type_name(),
            ))
        }
    }

    pub fn try_into_number(&self) -> Result<isize, MalError> {
        if let Self::Number(v) = self {
            Ok(*v)
        } else {
            Err(MalError::InvalidType(
                String::from("MalType::Number"),
                self.type_name(),
            ))
        }
    }

    pub fn try_into_string(&self) -> Result<String, MalError> {
        if let Self::String(v) = self {
            Ok(v.to_string())
        } else {
            Err(MalError::InvalidType(
                String::from("MalType::String"),
                self.type_name(),
            ))
        }
    }

    pub fn try_into_vector(self) -> Result<Vec<Rc<MalType>>, MalError> {
        if let Self::Vector(v) = self {
            Ok(v)
        } else {
            Err(MalError::InvalidType(
                String::from("MalType::Vector"),
                self.type_name(),
            ))
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
            (MalType::List(l1), MalType::Vector(l2)) => (l1, l2),
            (MalType::Vector(l1), MalType::Vector(l2)) => (l1, l2),
            (MalType::Vector(l1), MalType::List(l2)) => (l1, l2),
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

        true
    }

    pub fn as_func(&self) -> Option<&MalFunc> {
        if let Self::Func(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the mal type is [`String`].
    ///
    /// [`String`]: MalType::String
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }

    pub fn number(number: isize) -> Rc<MalType> {
        Rc::new(MalType::Number(number))
    }

    pub fn list(values: Vec<Rc<MalType>>) -> Rc<MalType> {
        Rc::new(MalType::List(values))
    }

    pub fn symbol(symbol: String) -> Rc<MalType> {
        Rc::new(MalType::Symbol(symbol))
    }

    pub fn string(string: String) -> Rc<MalType> {
        Rc::new(MalType::String(string))
    }

    pub fn try_into_atom(&self) -> Result<RefCell<Rc<MalType>>, MalError> {
        if let Self::Atom(v) = self {
            Ok(v.clone())
        } else {
            Err(MalError::InvalidType(
                String::from("MalType::Atom"),
                self.type_name(),
            ))
        }
    }

    /// Returns `true` if the mal type is [`Atom`].
    ///
    /// [`Atom`]: MalType::Atom
    #[must_use]
    pub fn is_atom(&self) -> bool {
        matches!(self, Self::Atom(..))
    }

    pub fn bool(value: bool) -> Rc<MalType> {
        Rc::new(if value { MalType::True } else { MalType::False })
    }

    pub fn try_into_func(&self) -> Result<&MalFunc, MalError> {
        if let Self::Func(v) = self {
            Ok(v)
        } else {
            Err(MalError::InvalidType(
                String::from("MalType::Func"),
                self.type_name(),
            ))
        }
    }
}

impl Eq for MalType {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(_), Self::List(_))
            | (Self::List(_), Self::Vector(_))
            | (Self::Vector(_), Self::Vector(_))
            | (Self::Vector(_), Self::List(_)) => MalType::compare_as_vec(self, other),
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
            Self::Atom(v) => f.debug_tuple("Atom").field(v).finish(),
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
            MalType::Atom(v) => v.borrow().hash(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::types::MalType;

    #[test]
    fn match_vector_list() {
        let elements = vec![Rc::new(MalType::Number(1)), Rc::new(MalType::Number(2))];
        let list = MalType::List(elements.clone());
        let vector = MalType::Vector(elements);

        assert_eq!(list, vector);
    }
}
