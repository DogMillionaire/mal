use std::hash::Hash;
use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;

use crate::env::Env;
use crate::malerror::MalError;

pub type MalMeta = Option<Rc<MalType>>;

pub type MalVal = Rc<MalType>;

pub enum MalType {
    Nil,
    List(Vec<Rc<MalType>>, MalMeta),
    Symbol(String),
    Number(isize),
    String(String),
    Vector(Vec<Rc<MalType>>, MalMeta),
    Keyword(String),
    Hashmap(IndexMap<Rc<MalType>, Rc<MalType>>, MalMeta),
    Func(MalFunc, MalMeta),
    True,
    False,
    Atom(RefCell<Rc<MalType>>),
}

/// Wrapper for a function
#[derive(Clone)]
pub struct MalFunc {
    name: String,
    parameters: Vec<Rc<MalType>>,
    body: Option<Rc<MalFn>>,
    env: Rc<RefCell<Env>>,
    body_ast: Rc<MalType>,
    is_macro: RefCell<bool>,
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
            body: Some(Rc::new(body)),
            env,
            body_ast,
            is_macro: RefCell::new(false),
        }
    }

    pub fn name(&self) -> String {
        if self.is_macro() {
            return String::from("macro");
        }
        self.name.clone()
    }

    pub fn parameters(&self) -> Vec<Rc<MalType>> {
        self.parameters.clone()
    }

    pub fn env(&self) -> Rc<RefCell<Env>> {
        self.env.clone()
    }

    pub fn body_ast(&self) -> Rc<MalType> {
        self.body_ast.clone()
    }

    pub fn body(&self) -> Option<&Rc<MalFn>> {
        self.body.as_ref()
    }

    pub fn is_macro(&self) -> bool {
        self.is_macro.borrow().clone()
    }

    pub fn set_is_macro(&self) {
        self.is_macro.replace(true);
    }

    pub fn apply(&self, args: Vec<Rc<MalType>>) -> Result<Rc<MalType>, MalError> {
        self.body.as_ref().expect("Body must be set")(
            self.env.clone(),
            self.body_ast.clone(),
            self.parameters.clone(),
            args,
        )
    }
}

#[allow(dead_code)]
impl MalType {
    pub fn type_name(&self) -> String {
        match self {
            MalType::Nil => String::from("MalType::Nil"),
            MalType::List(_, _) => String::from("MalType::List"),
            MalType::Symbol(_) => String::from("MalType::Symbol"),
            MalType::Number(_) => String::from("MalType::Number"),
            MalType::String(_) => String::from("MalType::String"),
            MalType::Vector(_, _) => String::from("MalType::Vector"),
            MalType::Keyword(_) => String::from("MalType::Keyword"),
            MalType::Hashmap(_, _) => String::from("MalType::Hashmap"),
            MalType::Func(_, _) => String::from("MalType::Func"),
            MalType::True => String::from("MalType::True"),
            MalType::False => String::from("MalType::False"),
            MalType::Atom(_) => String::from("MalType::Atom"),
        }
    }

    pub fn try_into_list(&self) -> Result<Vec<Rc<MalType>>, MalError> {
        match self {
            Self::List(v, _) => Ok(v.clone()),
            Self::Vector(v, _) => Ok(v.clone()),
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
        if let Self::Vector(v, _) = self {
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

    pub fn get_as_vec(&self) -> Result<Vec<Rc<MalType>>, MalError> {
        return match self {
            MalType::Vector(v, _) => Ok(v.clone()),
            MalType::List(l, _) => Ok(l.clone()),
            _ => Err(MalError::InvalidType(
                "MalType::Vector or MalType::List".to_string(),
                self.type_name(),
            )),
        };
    }

    fn compare_as_vec(this: &MalType, other: &MalType) -> bool {
        let (this_vec, other_vec) = match (this, other) {
            (MalType::List(l1, _), MalType::List(l2, _)) => (l1, l2),
            (MalType::List(l1, _), MalType::Vector(l2, _)) => (l1, l2),
            (MalType::Vector(l1, _), MalType::Vector(l2, _)) => (l1, l2),
            (MalType::Vector(l1, _), MalType::List(l2, _)) => (l1, l2),
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
        if let Self::Func(v, _) = self {
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

    pub fn new_list(values: Vec<Rc<MalType>>) -> Rc<MalType> {
        Rc::new(MalType::List(values, None))
    }

    pub fn symbol(symbol: String) -> Rc<MalType> {
        Rc::new(MalType::Symbol(symbol))
    }

    pub fn new_string(string: String) -> Rc<MalType> {
        Rc::new(MalType::String(string))
    }

    pub fn try_into_atom(&self) -> Result<&RefCell<Rc<MalType>>, MalError> {
        if let Self::Atom(v) = self {
            Ok(v)
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
        if let Self::Func(v, _) = self {
            Ok(v)
        } else {
            Err(MalError::InvalidType(
                String::from("MalType::Func"),
                self.type_name(),
            ))
        }
    }

    /// Returns `true` if the mal type is [`Func`].
    ///
    /// [`Func`]: MalType::Func
    #[must_use]
    pub fn is_func(&self) -> bool {
        matches!(self, Self::Func(..))
    }

    /// Returns `true` if the mal type is [`Symbol`].
    ///
    /// [`Symbol`]: MalType::Symbol
    #[must_use]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(..))
    }

    /// Returns `true` if the mal type is [`Nil`].
    ///
    /// [`Nil`]: MalType::Nil
    #[must_use]
    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    /// Returns `true` if the mal type is [`True`].
    ///
    /// [`True`]: MalType::True
    #[must_use]
    pub fn is_true(&self) -> bool {
        matches!(self, Self::True)
    }

    /// Returns `true` if the mal type is [`False`].
    ///
    /// [`False`]: MalType::False
    #[must_use]
    pub fn is_false(&self) -> bool {
        matches!(self, Self::False)
    }

    /// Returns `true` if the mal type is [`Keyword`].
    ///
    /// [`Keyword`]: MalType::Keyword
    #[must_use]
    pub fn is_keyword(&self) -> bool {
        matches!(self, Self::Keyword(..))
    }

    /// Returns `true` if the mal type is [`Vector`].
    ///
    /// [`Vector`]: MalType::Vector
    #[must_use]
    pub fn is_vector(&self) -> bool {
        matches!(self, Self::Vector(..))
    }

    /// Returns `true` if the mal type is [`Hashmap`].
    ///
    /// [`Hashmap`]: MalType::Hashmap
    #[must_use]
    pub fn is_hashmap(&self) -> bool {
        matches!(self, Self::Hashmap(..))
    }

    pub fn try_into_hashmap(&self) -> Result<IndexMap<Rc<MalType>, Rc<MalType>>, MalError> {
        if let Self::Hashmap(v, _) = self {
            Ok(v.clone())
        } else {
            Err(MalError::InvalidType(
                String::from("MalType::Hashmap"),
                self.type_name(),
            ))
        }
    }

    fn compare_hashmap(
        lhs: &IndexMap<Rc<MalType>, Rc<MalType>>,
        rhs: &IndexMap<Rc<MalType>, Rc<MalType>>,
    ) -> bool {
        if lhs.len() != rhs.len() {
            return false;
        }

        for key in lhs.keys() {
            let lvalue = lhs.get(key);
            let rvalue = rhs.get(key);

            match (lvalue, rvalue) {
                (Some(left), Some(right)) => {
                    if left != right {
                        return false;
                    }
                }
                _ => return false,
            }
        }

        return true;
    }

    /// Returns `true` if the mal type is [`Number`].
    ///
    /// [`Number`]: MalType::Number
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(..))
    }

    pub fn get_meta(&self) -> Option<Rc<MalType>> {
        match self {
            MalType::List(_, meta) => meta.clone(),
            MalType::Vector(_, meta) => meta.clone(),
            MalType::Hashmap(_, meta) => meta.clone(),
            MalType::Func(_, meta) => meta.clone(),
            _ => None,
        }
    }

    pub fn set_meta(&self, meta: Rc<MalType>) -> Result<Rc<MalType>, MalError> {
        match self {
            MalType::List(list, _) => Ok(Rc::new(MalType::List(list.clone(), Some(meta.clone())))),
            MalType::Vector(vec, _) => {
                Ok(Rc::new(MalType::Vector(vec.clone(), Some(meta.clone()))))
            }
            MalType::Hashmap(map, _) => {
                Ok(Rc::new(MalType::Hashmap(map.clone(), Some(meta.clone()))))
            }
            MalType::Func(func, _) => Ok(Rc::new(MalType::Func(func.clone(), Some(meta.clone())))),
            _ => Err(MalError::InvalidType(
                "MalType::List, MalType::Vector, MalType::Hashmap or MalType::Func".to_string(),
                self.type_name(),
            )),
        }
    }

    pub fn new_vector(values: Vec<Rc<MalType>>) -> Rc<MalType> {
        Rc::new(MalType::Vector(values.clone(), None))
    }

    pub fn new_hashmap(map: IndexMap<MalVal, MalVal>) -> MalVal {
        Rc::new(MalType::Hashmap(map.clone(), None))
    }
}

impl Eq for MalType {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(_, _), Self::List(_, _))
            | (Self::List(_, _), Self::Vector(_, _))
            | (Self::Vector(_, _), Self::Vector(_, _))
            | (Self::Vector(_, _), Self::List(_, _)) => MalType::compare_as_vec(self, other),
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Keyword(l0), Self::Keyword(r0)) => l0 == r0,
            (Self::Hashmap(l0, _), Self::Hashmap(r0, _)) => Self::compare_hashmap(l0, r0),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl std::fmt::Debug for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            Self::List(arg0, _) => f.debug_tuple("List").field(arg0).finish(),
            Self::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Vector(arg0, _) => f.debug_tuple("Vector").field(arg0).finish(),
            Self::Keyword(arg0) => f.debug_tuple("Keyword").field(arg0).finish(),
            Self::Hashmap(arg0, _) => f.debug_tuple("Hashmap").field(arg0).finish(),
            Self::Func(arg0, _) => f.debug_tuple("Func").field(arg0).finish(),
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
            MalType::List(l, _) => l.hash(state),
            MalType::Symbol(s) => s.hash(state),
            MalType::Number(n) => n.hash(state),
            MalType::String(s) => s.hash(state),
            MalType::Vector(v, _) => v.hash(state),
            MalType::Keyword(k) => k.hash(state),
            MalType::Hashmap(h, _) => {
                for entry in h {
                    entry.0.hash(state);
                    entry.1.hash(state);
                }
            }
            MalType::Func(func, _) => func.hash(state),
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
        let list = MalType::List(elements.clone(), None);
        let vector = MalType::Vector(elements, None);

        assert_eq!(list, vector);
    }
}
