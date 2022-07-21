use std::{char, collections::HashMap, fmt::Display, rc::Rc};

use crate::types::MalType;
use assert_matches::assert_matches;

const DEBUG: bool = false;
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

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    SpliceUnquote,
    OpenSquare,
    OpenParen,
    OpenBrace,
    CloseSquare,
    CloseParen,
    CloseBrace,
    Keyword(String),
    Quote,
    QuasiQuote,
    Unquote,
    Deref,
    WithMeta,
    String(String),
    Atom(String),
    Number(isize),
    EndOfFile,
}

#[derive(Debug)]
pub(crate) struct Reader {
    position: usize,
    tokens: Vec<Token>,
}

impl Reader {
    pub fn read_str(input: String) -> Result<Self, MalError> {
        let tokens = Self::tokenize(&input);
        if DEBUG {
            eprintln!("{:#?}", tokens);
        }
        match tokens {
            Ok(t) => Ok(Self {
                position: 0,
                tokens: t,
            }),
            Err(e) => Err(e),
        }
    }

    fn is_special_char(c: char) -> bool {
        match c {
            '[' | ']' | '{' | '}' | '(' | ')' | '\'' | '`' | '~' | '^' | '@' => true,
            _ => false,
        }
    }

    fn read_string(chars: &Vec<char>, start: usize) -> Result<(usize, String), MalError> {
        if start >= chars.len() {
            return Err(MalError::UnterminatedToken('"', start - 1, start));
        }

        let mut end = start;
        let mut result = String::with_capacity(chars.len() - start);

        let mut escape_next = false;
        let mut current: char;
        let mut end_found = false;
        loop {
            current = chars[end];
            match (current, escape_next) {
                ('\\', true) => {
                    result.push('\\');
                    escape_next = false;
                }
                ('\\', false) => {
                    escape_next = true;
                }
                ('"', false) => {
                    end_found = true;
                    break;
                }
                ('"', true) => {
                    result.push('"');
                    escape_next = false;
                }
                ('n', true) => {
                    result.push('\n');
                    escape_next = false;
                }
                (c, _) => {
                    result.push(c);
                }
            }
            end += 1;
            if end >= chars.len() {
                break;
            }
        }

        if !end_found {
            return Err(MalError::UnterminatedToken('"', start - 1, end));
        }
        Ok((end, result))
    }

    fn tokenize(input: &String) -> Result<Vec<Token>, MalError> {
        let mut tokens: Vec<Token> = vec![];

        let mut idx = 0;
        let chars: Vec<_> = input.chars().collect();

        loop {
            if idx >= input.len() {
                break;
            }
            let ch = chars[idx];

            let token = match ch {
                '~' => {
                    if chars[idx + 1] == '@' {
                        idx += 1;
                        Token::SpliceUnquote
                    } else {
                        Token::Unquote
                    }
                }
                ' ' | '\t' | ',' | '\n' => {
                    // Skip whitespace
                    idx += 1;
                    continue;
                }
                '(' => Token::OpenParen,
                ')' => Token::CloseParen,
                '[' => Token::OpenSquare,
                ']' => Token::CloseSquare,
                '{' => Token::OpenBrace,
                '}' => Token::CloseBrace,
                '\'' => Token::Quote,
                '`' => Token::QuasiQuote,
                '@' => Token::Deref,
                '^' => Token::WithMeta,
                '"' => {
                    let (end, string) = Self::read_string(&chars, idx + 1)?;
                    idx = end;
                    Token::String(string)
                }
                ';' => {
                    let (end, _) =
                        Self::read_until(&chars, idx, &|current, _| current == '\n', false)?;
                    idx = end;
                    continue;
                }
                '/' => Token::Atom("/".to_string()),
                '*' => {
                    if (idx < chars.len() - 1) && chars[idx + 1] == '*' {
                        idx += 1;
                        Token::Atom("**".to_string())
                    } else if (idx < chars.len() - 1) && chars[idx + 1] == 'A' {
                        idx += 5;
                        Token::Atom("*ARGV*".to_string())
                    } else {
                        Token::Atom("*".to_string())
                    }
                }
                '+' => Token::Atom("+".to_string()),
                '-' => {
                    // Negative number
                    if (idx < chars.len() - 1) && chars[idx + 1].is_ascii_digit() {
                        let (end, string) =
                            Self::read_until(&chars, idx + 1, &|c, _| !c.is_ascii_digit(), false)?;

                        idx = end;

                        Token::Number(match string.parse::<isize>() {
                            Ok(it) => -it,
                            Err(_) => return Err(MalError::InvalidNumber(string, idx)),
                        })
                    // Dash at the start of an atom
                    } else if (idx < chars.len() - 1) && !chars[idx + 1].is_ascii_whitespace() {
                        let (end, string) =
                            Self::read_until(&chars, idx, &|c, _| Self::is_special_char(c), false)?;

                        idx = end - 1;
                        Token::Atom(string)
                    } else {
                        Token::Atom("-".to_string())
                    }
                }
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    let (end, string) =
                        Self::read_until(&chars, idx, &|c, _| !c.is_ascii_digit(), false)?;

                    idx = end - 1;

                    Token::Number(match string.parse::<isize>() {
                        Ok(it) => it,
                        Err(_) => return Err(MalError::InvalidNumber(string, idx)),
                    })
                }
                '\\' => {
                    idx += 1;
                    continue;
                }
                ':' => {
                    let (end, string) = Self::read_until(
                        &chars,
                        idx + 1,
                        &|c, _| Self::is_special_char(c) || c.is_ascii_whitespace(),
                        false,
                    )?;
                    idx = end - 1;
                    Token::Keyword(string)
                }
                _ => {
                    let (end, string) = Self::read_until(
                        &chars,
                        idx,
                        &|c, _| Self::is_special_char(c) || c.is_ascii_whitespace(),
                        false,
                    )?;
                    idx = end - 1;
                    Token::Atom(string)
                }
            };
            if DEBUG {
                eprintln!("Adding token: {:?}", token);
            }
            tokens.push(token);

            idx += 1;
        }

        Ok(tokens)
    }

    fn read_until(
        chars: &Vec<char>,
        start: usize,
        is_end: &dyn Fn(char, char) -> bool,
        must_find_end: bool,
    ) -> Result<(usize, String), MalError> {
        let mut idx = start;

        if idx >= chars.len() {
            return Err(MalError::UnterminatedToken(
                chars[start - 1],
                start - 1,
                idx,
            ));
        }
        let mut found_end = false;

        loop {
            if idx > 0 && is_end(chars[idx], chars[idx - 1]) {
                found_end = true;
                break;
            }
            idx += 1;
            if idx >= chars.len() {
                break;
            }
        }

        if !found_end && must_find_end {
            return Err(MalError::UnterminatedToken(chars[start], start, idx));
        }

        let mut result = String::with_capacity(idx - start);

        (start..idx).for_each(|i| result.push(chars[i]));

        Ok((idx, result))
    }

    pub fn read_form(&mut self) -> Result<Rc<MalType>, MalError> {
        let next_token = self.peek();
        if DEBUG {
            eprintln!("read_form: {:?}", next_token);
        }
        match next_token {
            Token::OpenParen => self.read_list(),
            &Token::OpenSquare => self.read_vector(),
            &Token::OpenBrace => self.read_hashmap(),
            Token::Quote => self.read_macro("quote".to_string()),
            Token::QuasiQuote => self.read_macro("quasiquote".to_string()),
            Token::Unquote => self.read_macro("unquote".to_string()),
            Token::SpliceUnquote => self.read_macro("splice-unquote".to_string()),
            Token::Deref => self.read_macro("deref".to_string()),
            Token::WithMeta => {
                let mut types: Vec<Rc<MalType>> = vec![];

                types.push(Rc::new(MalType::Symbol("with-meta".to_string())));
                self.next();
                let hashmap = self.read_form()?;
                assert_matches!(
                    hashmap.as_ref(),
                    MalType::Hashmap(_),
                    "First element after with-meta should be a hashmap"
                );
                let vector = self.read_form()?;

                assert_matches!(
                    vector.as_ref(),
                    MalType::Vector(_),
                    "First element after with-meta should be a vector"
                );
                types.push(vector);
                types.push(hashmap);

                Ok(Rc::new(MalType::List(types)))
            }
            Token::Keyword(name) => {
                let result = MalType::Keyword(name.to_string());
                self.next();
                Ok(Rc::new(result))
            }
            _ => self.read_atom(),
        }
    }

    fn read_macro(&mut self, symbol: String) -> Result<Rc<MalType>, MalError> {
        let mut types: Vec<Rc<MalType>> = vec![];

        types.push(Rc::new(MalType::Symbol(symbol)));
        self.next();
        types.push(self.read_form()?);

        Ok(Rc::new(MalType::List(types)))
    }

    fn read_hashmap(&mut self) -> Result<Rc<MalType>, MalError> {
        let tokens = self.read_token_list(&Token::OpenBrace, &Token::CloseBrace)?;

        let mut hashmap: HashMap<Rc<MalType>, Rc<MalType>> = HashMap::new();

        if tokens.len() % 2 != 0 {
            return Err(MalError::UnbalancedHashmap);
        }

        for chunk in tokens.chunks_exact(2) {
            hashmap.insert(chunk[0].clone(), chunk[1].clone());
        }

        Ok(Rc::new(MalType::Hashmap(hashmap)))
    }

    fn read_atom(&mut self) -> Result<Rc<MalType>, MalError> {
        match self.next() {
            Token::String(s) => Ok(Rc::new(MalType::String(s.to_string()))),
            Token::Atom(s) if s == "nil" => Ok(Rc::new(MalType::Nil)),
            Token::Atom(s) if s == "true" => Ok(Rc::new(MalType::True)),
            Token::Atom(s) if s == "false" => Ok(Rc::new(MalType::False)),
            Token::Atom(s) => Ok(Rc::new(MalType::Symbol(s.to_string()))),
            Token::Number(n) => Ok(Rc::new(MalType::Number(*n))),
            _ => Ok(Rc::new(MalType::Nil)),
        }
    }

    fn read_vector(&mut self) -> Result<Rc<MalType>, MalError> {
        Ok(Rc::new(MalType::Vector(self.read_token_list(
            &Token::OpenSquare,
            &Token::CloseSquare,
        )?)))
    }

    fn read_list(&mut self) -> Result<Rc<MalType>, MalError> {
        Ok(Rc::new(MalType::List(
            self.read_token_list(&Token::OpenParen, &Token::CloseParen)?,
        )))
    }

    fn read_token_list(
        &mut self,
        start_token: &Token,
        end_token: &Token,
    ) -> Result<Vec<Rc<MalType>>, MalError> {
        let mut tokens: Vec<Rc<MalType>> = vec![];
        // Skip the open OpenParen
        assert_eq!(start_token, self.next());
        loop {
            if DEBUG {
                eprintln!("Next token: {:?}", self.peek());
            }
            match self.peek() {
                Token::EndOfFile => {
                    return Err(MalError::UnterminatedList);
                }
                token => {
                    if token == end_token {
                        self.next();
                        break;
                    }
                    tokens.push(self.read_form()?)
                }
            }
        }
        Ok(tokens)
    }

    pub fn next(&mut self) -> &Token {
        if self.position >= self.tokens.len() {
            return &Token::EndOfFile;
        }
        let token = &self.tokens[self.position];
        self.position += 1;
        token
    }

    pub fn peek(&self) -> &Token {
        if self.position >= self.tokens.len() {
            return &Token::EndOfFile;
        }
        &self.tokens[self.position]
    }
}

#[cfg(test)]
mod tests {
    use crate::{reader::Reader, types::MalType};
    use assert_matches::assert_matches;

    #[test]
    fn parse_list() {
        let mut reader = Reader::read_str("(1 2 3)".to_string()).unwrap();

        let result = reader.read_form().unwrap();

        assert_matches!(result.as_ref(), &MalType::List(_));
    }

    #[test]
    fn parse_nested_list() {
        let mut reader = Reader::read_str("(()()))".to_string()).unwrap();

        let result = reader.read_form().unwrap();

        assert_matches!(result.as_ref(), MalType::List(l) => {
            assert_eq!(2, l.len());
            assert_matches!(l[0].as_ref(), MalType::List(_));
            assert_matches!(l[0].as_ref(), MalType::List(_));
        });
    }

    #[test]
    fn parse_string() {
        let mut reader = Reader::read_str("\"abc\"".to_string()).unwrap();

        let result = reader.read_form().unwrap();

        assert_matches!(result.as_ref(), MalType::String(s) => {
            assert_eq!("abc", s);
        });
    }

    #[test]
    fn parse_unterminated_string() {
        let reader = Reader::read_str("\"abc".to_string());

        assert_matches!(reader, Err(_));
    }

    #[test]
    fn parse_double_slash_in_string() {
        let reader = Reader::read_str("\"\\\\\"".to_string());

        assert_matches!(reader, Ok(_));
    }

    #[test]
    fn parse_escaped_doublequote_in_string() {
        let mut reader = Reader::read_str("\"abc\\\"def\"".to_string()).unwrap();

        let result = reader.read_form().unwrap();

        assert_matches!(result.as_ref(), MalType::String(s) => {
            assert_eq!("abc\\\"def", s);
        });
    }

    #[test]
    fn parse_keyword() {
        let mut reader = Reader::read_str(":kw".to_string()).unwrap();

        let result = reader.read_form().unwrap();

        assert_matches!(result.as_ref(), MalType::Keyword(k) => {
            assert_eq!("kw", k);
        });
    }

    #[test]
    fn parse_quote() {
        let mut reader = Reader::read_str("'1".to_string()).unwrap();

        let result = reader.read_form().unwrap();

        assert_matches!(result.as_ref(), MalType::List(l) => {
            assert_eq!(2, l.len());
            assert_matches!(l[0].as_ref(), &MalType::Symbol(_));
            assert_matches!(l[1].as_ref(), &MalType::Number(1));
        });
    }

    #[test]
    fn parse_def_call() {
        let mut reader = Reader::read_str("(def! x 3)".to_string()).unwrap();

        let result = reader.read_form().expect("Failed to parse");

        assert_matches!(result.as_ref(), MalType::List(l) => {
            assert_eq!(3, l.len(), "List should have 3 elements");
            assert_matches!(l[0].as_ref(), &MalType::Symbol(_));
            assert_matches!(l[1].as_ref(), &MalType::Symbol(_));
            assert_matches!(l[2].as_ref(), &MalType::Number(_));
        });
    }

    #[test]
    fn parse_nil() {
        let mut reader = Reader::read_str("nil".to_string()).unwrap();

        let result = reader.read_form().expect("Failed to parse");

        assert_matches!(result.as_ref(), &MalType::Nil);
    }

    #[test]
    fn parse_true() {
        let mut reader = Reader::read_str("true".to_string()).unwrap();

        let result = reader.read_form().expect("Failed to parse");

        assert_matches!(result.as_ref(), &MalType::True);
    }

    #[test]
    fn parse_false() {
        let mut reader = Reader::read_str("false".to_string()).unwrap();

        let result = reader.read_form().expect("Failed to parse");

        assert_matches!(result.as_ref(), &MalType::False);
    }

    #[test]
    fn parse_with_meta() {
        let mut reader = Reader::read_str(r#"^{"a" 1} [1 2 3]"#.to_string()).unwrap();

        let result = reader.read_form().expect("Failed to parse");

        assert_matches!(result.as_ref(), MalType::List(l) => {
            assert_eq!(3, l.len(), "List should have 3 elements");
            assert_matches!(l[0].as_ref(), &MalType::Symbol(_), "First list element should be a symbol");
            assert_matches!(l[1].as_ref(), &MalType::Vector(_), "First list element should be a vector");
            assert_matches!(l[2].as_ref(), &MalType::Hashmap(_), "Second list element should be a hashmap");
        });
    }

    //
}
