use std::fmt::Display;

use crate::types::MalType;

pub struct Printer {}

impl Display for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalType::Nil => f.write_str("nil"),
            MalType::List(l) => {
                let values: Vec<_> = l.iter().map(|v| format!("{}", v)).collect();
                f.write_str(&format!("({})", values.join(" ")))
            }
            MalType::Symbol(name) => f.write_str(name),
            MalType::Number(n) => f.write_str(&format!("{}", n)),
            MalType::String(s) => f.write_str(&format!("\"{}\"", s)),
            MalType::Vector(v) => {
                let values: Vec<_> = v.iter().map(|v| format!("{}", v)).collect();
                f.write_str(&format!("[{}]", values.join(" ")))
            }
            MalType::Keyword(kw) => f.write_str(&format!(":{}", kw)),
            MalType::Hashmap(h) => {
                let values: Vec<_> = h.iter().map(|v| format!("{} {}", v.0, v.1)).collect();
                f.write_str(&format!("{{{}}}", values.join(" ")))
            }
            MalType::Func(func) => f.write_str(&format!("#<function:{}>", func.name())),
            MalType::True => f.write_str("true"),
            MalType::False => f.write_str("false"),
        }
    }
}

impl Printer {
    pub fn pr_str(data: &MalType, print_readonly: bool) -> String {
        let mut formatted = format!("{}", data);
        if print_readonly {
            formatted = formatted.replace("\\", "\\\\");
            formatted = formatted.replace("\n", "\\n");
            // Ignore leading and trailing spaces
            let mut add_quotes = false;
            if formatted.starts_with('"') && formatted.ends_with('"') {
                add_quotes = true;
                formatted = formatted
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .to_string();

                formatted = formatted.replace('"', "\\\"");
            }
            if add_quotes {
                formatted = format!("\"{}\"", formatted);
            }
        }

        formatted
    }
}
