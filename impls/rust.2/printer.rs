use std::{fmt::Display, rc::Rc};

use crate::types::MalType;

pub struct Printer {}

impl Display for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalType::Nil => f.write_str("nil"),
            MalType::List(l, _) => {
                let values: Vec<_> = l.iter().map(|v| format!("{}", v)).collect();
                f.write_str(&format!("({})", values.join(" ")))
            }
            MalType::Symbol(name) => f.write_str(name),
            MalType::Number(n) => f.write_str(&format!("{}", n)),
            MalType::String(s) => f.write_str(&format!("\"{}\"", s)),
            MalType::Vector(v, _) => {
                let values: Vec<_> = v.iter().map(|v| format!("{}", v)).collect();
                f.write_str(&format!("[{}]", values.join(" ")))
            }
            MalType::Keyword(kw) => f.write_str(&format!(":{}", kw)),
            MalType::Hashmap(h, _) => {
                let values: Vec<_> = h.iter().map(|v| format!("{} {}", v.0, v.1)).collect();
                f.write_str(&format!("{{{}}}", values.join(" ")))
            }
            MalType::Func(func, _) => f.write_str(&format!("#<function:{}>", func.name())),
            MalType::True => f.write_str("true"),
            MalType::False => f.write_str("false"),
            MalType::Atom(v) => f.write_str(&format!("Atom({:?})", v)),
        }
    }
}

impl Printer {
    fn print_seperated(
        list: &[Rc<MalType>],
        start_char: char,
        end_char: char,
        seperator: &str,
        print_readonly: bool,
    ) -> String {
        let values: Vec<_> = list
            .iter()
            .map(|v| Self::pr_str(v, print_readonly))
            .collect();
        format!("{}{}{}", start_char, values.join(seperator), end_char)
    }

    fn print_string(string: &String, print_readonly: bool) -> String {
        let mut formatted = string.to_string();
        if print_readonly {
            formatted = formatted.replace('\\', "\\\\");
            formatted = formatted.replace('\n', "\\n");
            formatted = formatted.replace('"', "\\\"");
            formatted = format!("\"{}\"", formatted);
        }

        formatted
    }

    pub fn pr_str(data: &MalType, print_readonly: bool) -> String {
        match data {
            MalType::Nil => String::from("nil"),
            MalType::List(l, _) => Self::print_seperated(l, '(', ')', " ", print_readonly),
            MalType::Symbol(s) => s.to_string(),
            MalType::Number(n) => format!("{}", n),
            MalType::String(s) => Self::print_string(s, print_readonly),
            MalType::Vector(v, _) => Self::print_seperated(v, '[', ']', " ", print_readonly),
            MalType::Keyword(kw) => format!(":{}", kw),
            MalType::Hashmap(h, _) => {
                let values: Vec<_> = h
                    .iter()
                    .map(|v| {
                        format!(
                            "{} {}",
                            Self::pr_str(v.0, print_readonly),
                            Self::pr_str(v.1, print_readonly)
                        )
                    })
                    .collect();
                format!("{}{}{}", '{', values.join(" "), '}')
            }
            MalType::Func(func, _) => format!("#<function:{}>", func.name()),
            MalType::True => String::from("true"),
            MalType::False => String::from("false"),
            MalType::Atom(a) => format!(
                "(atom {})",
                Self::pr_str(a.borrow().as_ref(), print_readonly)
            ),
        }
    }
}
