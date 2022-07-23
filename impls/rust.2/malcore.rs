use std::fs::File;
use std::{cell::RefCell, collections::HashMap, io::Read, rc::Rc};

use crate::env::Env;
use crate::malerror::MalError;
use crate::printer::Printer;
use crate::reader::Reader;
use crate::types::{MalFunc, MalType};

#[allow(dead_code)]
pub struct MalCore {
    ns: HashMap<String, MalType>,
}

#[allow(dead_code)]
impl MalCore {
    pub fn add_to_env(env: Rc<RefCell<Env>>) -> Self {
        let instance = Self { ns: HashMap::new() };

        Self::add_numeric_func(env.clone(), "+", &|a, b| a + b);
        Self::add_numeric_func(env.clone(), "/", &|a, b| a / b);
        Self::add_numeric_func(env.clone(), "*", &|a, b| a * b);
        Self::add_numeric_func(env.clone(), "-", &|a, b| a - b);

        Self::add_param_list_func(env.clone(), "pr-str", &|a| {
            Self::print_str(a, " ", true, false, true)
        });
        Self::add_param_list_func(env.clone(), "str", &|a| {
            Self::print_str(a, "", false, false, true)
        });
        Self::add_param_list_func(env.clone(), "prn", &|a| {
            Self::print_str(a, " ", true, true, false)
        });
        Self::add_param_list_func(env.clone(), "println", &|a| {
            Self::print_str(a, " ", false, true, false)
        });

        Self::add_param_list_func(env.clone(), "list", &|a| Ok(Rc::new(MalType::List(a))));

        Self::add_unary_func(env.clone(), "list?", &|a| {
            let result = match a.is_list() {
                true => MalType::True,
                false => MalType::False,
            };
            Ok(Rc::new(result))
        });
        Self::add_unary_func(env.clone(), "empty?", &|a| {
            let result = match a.try_into_list()?.is_empty() {
                true => MalType::True,
                false => MalType::False,
            };
            Ok(Rc::new(result))
        });
        Self::add_unary_func(env.clone(), "count", &|a| {
            Ok(Rc::new(MalType::Number(
                a.try_into_list().unwrap_or_default().len() as isize,
            )))
        });
        Self::add_binary_func(env.clone(), "<", &|a, b| {
            let lhs = a.try_into_number()?;
            let rhs = b.try_into_number()?;
            if lhs < rhs {
                Ok(Rc::new(MalType::True))
            } else {
                Ok(Rc::new(MalType::False))
            }
        });
        Self::add_binary_func(env.clone(), "<=", &|a, b| {
            let lhs = a.try_into_number()?;
            let rhs = b.try_into_number()?;
            if lhs <= rhs {
                Ok(Rc::new(MalType::True))
            } else {
                Ok(Rc::new(MalType::False))
            }
        });
        Self::add_binary_func(env.clone(), ">", &|a, b| {
            let lhs = a.try_into_number()?;
            let rhs = b.try_into_number()?;
            if lhs > rhs {
                Ok(Rc::new(MalType::True))
            } else {
                Ok(Rc::new(MalType::False))
            }
        });
        Self::add_binary_func(env.clone(), ">=", &|a, b| {
            let lhs = a.try_into_number()?;
            let rhs = b.try_into_number()?;
            if lhs >= rhs {
                Ok(Rc::new(MalType::True))
            } else {
                Ok(Rc::new(MalType::False))
            }
        });
        Self::add_binary_func(env.clone(), "=", &|a, b| {
            if a == b {
                Ok(Rc::new(MalType::True))
            } else {
                Ok(Rc::new(MalType::False))
            }
        });
        Self::add_unary_func(env.clone(), "read-string", &|str| {
            let input = str.try_into_string()?;
            Reader::read_str(input)?.read_form()
        });
        Self::add_unary_func(env.clone(), "slurp", &|str| {
            let filename = str.try_into_string()?;

            let mut file = File::open(&filename).map_err(|_| MalError::FileNotFound(filename))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| MalError::InternalError(format!("{}", e)))?;

            Ok(MalType::new_string(content))
        });

        Self::add_unary_func(env.clone(), "atom", &|atom| {
            Ok(Rc::new(MalType::Atom(RefCell::new(atom))))
        });
        Self::add_unary_func(env.clone(), "atom?", &|atom| {
            Ok(MalType::bool(atom.is_atom()))
        });
        Self::add_unary_func(env.clone(), "deref", &|atom| {
            let value = atom.try_into_atom()?;
            Ok(value.borrow().clone())
        });
        Self::add_binary_func(env.clone(), "reset!", &|val1, val2| {
            let atom = val1.try_into_atom()?;
            atom.replace(val2.clone());
            Ok(val2)
        });

        Self::add_binary_func(env.clone(), "cons", &|arg1, arg2| {
            let list = arg2.as_ref().get_as_vec()?;

            let mut new_list = Vec::with_capacity(list.len() + 1);
            new_list.push(arg1);
            list.iter().for_each(|v| new_list.push(v.clone()));

            Ok(MalType::list(new_list))
        });

        Self::add_param_list_func(env.clone(), "concat", &|args| {
            let mut new_list: Vec<Rc<MalType>> = Vec::new();
            for arg in args {
                let list = arg.get_as_vec()?;
                list.iter().for_each(|v| new_list.push(v.clone()));
            }

            Ok(MalType::list(new_list))
        });

        Self::add_unary_func(env.clone(), "vec", &|list| match list.as_ref() {
            MalType::Vector(_) => return Ok(list),
            MalType::List(l) => return Ok(Rc::new(MalType::Vector(l.clone()))),
            _ => {
                return Err(MalError::InvalidType(
                    "MalType::Vector or MalType::List".to_string(),
                    list.type_name(),
                ))
            }
        });

        Self::add_binary_func(env.clone(), "nth", &|a, b| {
            let list = a.get_as_vec()?;
            let index = b.try_into_number()? as usize;

            if index < list.len() {
                return Ok(list[index].clone());
            }

            Err(MalError::Exception(MalType::new_string(format!(
                "Invalid index {}",
                index
            ))))
        });

        Self::add_unary_func(env.clone(), "first", &|a| match a.as_ref() {
            MalType::List(v) | MalType::Vector(v) => {
                if v.is_empty() {
                    return Ok(Rc::new(MalType::Nil));
                }
                return Ok(v[0].clone());
            }
            MalType::Nil => Ok(Rc::new(MalType::Nil)),
            _ => Err(MalError::InvalidType(
                "MalType::List, MalType::Vector or MalType::Nil".to_string(),
                a.type_name(),
            )),
        });
        Self::add_unary_func(env.clone(), "rest", &|a| match a.as_ref() {
            MalType::List(v) | MalType::Vector(v) => {
                if v.is_empty() {
                    return Ok(MalType::list(vec![]));
                }
                let rest = v.iter().skip(1).map(|v| v.clone()).collect();
                return Ok(MalType::list(rest));
            }
            MalType::Nil => Ok(MalType::list(vec![])),
            _ => Err(MalError::InvalidType(
                "MalType::List, MalType::Vector or MalType::Nil".to_string(),
                a.type_name(),
            )),
        });

        Self::add_unary_func(env.clone(), "throw", &|value| {
            Err(MalError::Exception(value))
        });

        Self::add_unary_func(env.clone(), "nil?", &|a| Ok(MalType::bool(a.is_nil())));
        Self::add_unary_func(env.clone(), "true?", &|a| Ok(MalType::bool(a.is_true())));
        Self::add_unary_func(env.clone(), "false?", &|a| Ok(MalType::bool(a.is_false())));
        Self::add_unary_func(env.clone(), "symbol?", &|a| {
            Ok(MalType::bool(a.is_symbol()))
        });

        instance
    }

    fn print_str(
        params: Vec<Rc<MalType>>,
        seperator: &str,
        print_readably: bool,
        output: bool,
        return_data: bool,
    ) -> Result<Rc<MalType>, MalError> {
        let data = params
            .iter()
            .map(|v| Printer::pr_str(v.as_ref(), print_readably))
            .collect::<Vec<String>>()
            .join(seperator);

        if output {
            println!("{}", data);
        }

        if return_data {
            Ok(Rc::new(MalType::String(data)))
        } else {
            Ok(Rc::new(MalType::Nil))
        }
    }

    fn add_param_list_func(
        env: Rc<RefCell<Env>>,
        name: &str,
        func: &'static dyn Fn(Vec<Rc<MalType>>) -> Result<Rc<MalType>, MalError>,
    ) {
        let body = |_env: Rc<RefCell<Env>>,
                    _body: Rc<MalType>,
                    _params: Vec<Rc<MalType>>,
                    param_values: Vec<Rc<MalType>>|
         -> Result<Rc<MalType>, MalError> { func(param_values) };

        let malfunc = Rc::new(MalType::Func(MalFunc::new_with_closure(
            Some(name.to_string()),
            vec![],
            body,
            env.clone(),
            Rc::new(MalType::Nil),
        )));

        env.borrow_mut().set(name.to_string(), malfunc);
    }

    fn add_binary_func(
        env: Rc<RefCell<Env>>,
        name: &str,
        func: &'static dyn Fn(Rc<MalType>, Rc<MalType>) -> Result<Rc<MalType>, MalError>,
    ) {
        let params = vec![
            Rc::new(MalType::Symbol("lhs".to_string())),
            Rc::new(MalType::Symbol("rhs".to_string())),
        ];

        let body = |env: Rc<RefCell<Env>>,
                    _body: Rc<MalType>,
                    params: Vec<Rc<MalType>>,
                    param_values: Vec<Rc<MalType>>|
         -> Result<Rc<MalType>, MalError> {
            let func_env = Env::new_with_outer(Some(params), Some(param_values), env);
            let lhs = func_env.borrow().get("lhs".to_string())?;
            let rhs = func_env.borrow().get("rhs".to_string())?;
            func(lhs, rhs)
        };

        let malfunc = Rc::new(MalType::Func(MalFunc::new_with_closure(
            Some(name.to_string()),
            params,
            body,
            env.clone(),
            Rc::new(MalType::Nil),
        )));

        env.borrow_mut().set(name.to_string(), malfunc);
    }

    fn add_unary_func(
        env: Rc<RefCell<Env>>,
        name: &str,
        func: &'static dyn Fn(Rc<MalType>) -> Result<Rc<MalType>, MalError>,
    ) {
        let params = vec![Rc::new(MalType::Symbol("a".to_string()))];

        let body = |env: Rc<RefCell<Env>>,
                    _body: Rc<MalType>,
                    params: Vec<Rc<MalType>>,
                    param_values: Vec<Rc<MalType>>|
         -> Result<Rc<MalType>, MalError> {
            let func_env = Env::new_with_outer(Some(params), Some(param_values), env);
            let a = func_env.borrow().get("a".to_string())?;
            func(a)
        };

        let malfunc = Rc::new(MalType::Func(MalFunc::new_with_closure(
            Some(name.to_string()),
            params,
            body,
            env.clone(),
            Rc::new(MalType::Nil),
        )));

        env.borrow_mut().set(name.to_string(), malfunc);
    }

    fn add_numeric_func(
        env: Rc<RefCell<Env>>,
        name: &str,
        func: &'static dyn Fn(isize, isize) -> isize,
    ) {
        let params = vec![
            Rc::new(MalType::Symbol("a".to_string())),
            Rc::new(MalType::Symbol("b".to_string())),
        ];

        let body = |_env: Rc<RefCell<Env>>,
                    _body: Rc<MalType>,
                    _params: Vec<Rc<MalType>>,
                    param_values: Vec<Rc<MalType>>|
         -> Result<Rc<MalType>, MalError> {
            let a = param_values[0].clone().try_into_number()?;
            let b = param_values[1].clone().try_into_number()?;
            Ok(Rc::new(MalType::Number(func(a, b))))
        };

        let func = MalFunc::new_with_closure(
            Some(name.to_string()),
            params,
            body,
            env.clone(),
            Rc::new(MalType::Nil),
        );

        let malfunc = Rc::new(MalType::Func(func));

        env.borrow_mut().set(name.to_string(), malfunc);
    }
}
