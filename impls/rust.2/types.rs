use std::{collections::HashMap, rc::Rc};

pub enum MalType {
    Nil,
    List(Vec<MalType>),
    Symbol(String),
    Number(isize),
    String(String),
    Vector(Vec<MalType>),
    Keyword(String),
    Hashmap(HashMap<MalType, MalType>),
    Func(String, Rc<dyn Fn(MalType, MalType) -> MalType>),
    True,
    False,
}

impl From<MalType> for String {
    fn from(mal_type: MalType) -> Self {
        match mal_type {
            MalType::String(s) => s,
            MalType::Symbol(sym) => sym,
            t => panic!("Can't convert {:?} into a String", t),
        }
    }
}

impl From<MalType> for isize {
    fn from(mal_type: MalType) -> Self {
        match mal_type {
            MalType::Number(n) => n,
            t => panic!("Can't convert {:?} into an isize", t),
        }
    }
}

impl From<MalType> for Vec<MalType> {
    fn from(mal_type: MalType) -> Self {
        match mal_type {
            MalType::List(l) => l,
            MalType::Vector(v) => v,
            t => panic!("Can't convert {:?} into an Vec<MalType>", t),
        }
    }
}

impl Eq for MalType {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Clone for MalType {
    fn clone(&self) -> Self {
        match self {
            Self::Nil => Self::Nil,
            Self::List(arg0) => Self::List(arg0.clone()),
            Self::Symbol(arg0) => Self::Symbol(arg0.clone()),
            Self::Number(arg0) => Self::Number(arg0.clone()),
            Self::String(arg0) => Self::String(arg0.clone()),
            Self::Vector(arg0) => Self::Vector(arg0.clone()),
            Self::Keyword(arg0) => Self::Keyword(arg0.clone()),
            Self::Hashmap(arg0) => Self::Hashmap(arg0.clone()),
            Self::Func(arg0, arg1) => Self::Func(arg0.clone(), arg1.clone()),
            Self::True => Self::True,
            Self::False => Self::False,
        }
    }
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Vector(l0), Self::Vector(r0)) => l0 == r0,
            (Self::Keyword(l0), Self::Keyword(r0)) => l0 == r0,
            (Self::Hashmap(l0), Self::Hashmap(r0)) => l0.len() == r0.len(),
            (Self::Func(l0, _), Self::Func(r0, _)) => l0 == r0,
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
            Self::Func(arg0, _) => f.debug_tuple("Func").field(arg0).finish(),
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
            MalType::Func(name, _) => name.hash(state),
            MalType::True => core::mem::discriminant(self).hash(state),
            MalType::False => core::mem::discriminant(self).hash(state),
        }
    }
}
