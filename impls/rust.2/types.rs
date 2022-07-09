use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::Hasher,
};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MalType {
    Nil,
    List(Vec<MalType>),
    Symbol(String),
    Number(isize),
    String(String),
    Vector(Vec<MalType>),
    Keyword(String),
    Hashmap(HashMap<MalType, MalType>),
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
        }
    }
}
