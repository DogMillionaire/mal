#[derive(Debug)]
pub enum MalType {
    Nil,
    List(Vec<MalType>),
    Symbol(String),
    Number(isize),
    String(String),
    Vector(Vec<MalType>),
    Keyword(String),
}
