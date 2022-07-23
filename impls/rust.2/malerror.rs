use std::fmt::Display;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum MalError {
    UnterminatedToken(char, usize, usize),
    UnterminatedList,
    InvalidNumber(String, usize),
    UnbalancedHashmap,
    SymbolNotFound(String),
    InvalidType(String, String),
    ParseError(String),
    IncorrectParamCount(String, usize, usize),
    FileNotFound(String),
    InternalError(String),
}

impl Display for MalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            MalError::UnterminatedToken(char, start, end) => write!(
                f,
                "end of input found while looking for token '{}' start: {}, end: {}",
                char, start, end
            ),
            MalError::InvalidNumber(number, location) => {
                write!(
                    f,
                    "Failed to parse number '{}' at location {}",
                    number, location
                )
            }
            MalError::UnterminatedList => {
                write!(f, "end of input found while looking for end of list")
            }
            MalError::UnbalancedHashmap => {
                write!(f, "Number of keys and values does not match for hashmap")
            }
            MalError::SymbolNotFound(s) => write!(f, "Symbol '{}' not found", s),
            MalError::InvalidType(expected, actual) => write!(
                f,
                "Invalid type. Expected: {}, Actual: {}",
                expected, actual
            ),
            MalError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            MalError::IncorrectParamCount(name, expected, actual) => write!(
                f,
                "Function {} expected {} parameters, called with {} parameters",
                name, expected, actual
            ),
            &MalError::FileNotFound(file) => write!(f, "File '{}' not found", file),
            &MalError::InternalError(error) => write!(f, "Internal Error: '{}'", error),
        }
    }
}
