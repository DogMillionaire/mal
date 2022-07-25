use std::fs::File;
use std::time::{self, SystemTime, UNIX_EPOCH};
use std::{cell::RefCell, collections::HashMap, io::Read, rc::Rc};

use indexmap::IndexMap;

use crate::env::{Env, MalEnv};
use crate::malerror::MalError;
use crate::printer::Printer;
use crate::reader::Reader;
use crate::repl::Repl;
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

        Self::add_unary_func_with_env(env.clone(), "eval", &|ast, env| Repl::eval2(ast, env));

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
        Self::add_param_list_func_with_env(env.clone(), "swap!", &|params, env| {
            let atom = params[0].clone();
            let atom_value = atom.try_into_atom()?;
            let func = params[1].clone();

            let mut func_ast = vec![func, atom_value.borrow().clone()];
            params[2..params.len()]
                .iter()
                .for_each(|p| func_ast.push(p.clone()));

            let new_value = Repl::eval2(MalType::list(func_ast), env.clone())?;
            atom_value.replace(new_value.clone());

            Ok(new_value)
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
                "Index {} out of range",
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

        Self::add_binary_func(env.clone(), "map", &|func, values| {
            let values_to_map = values.get_as_vec()?;
            let mut results = Vec::with_capacity(values_to_map.len());

            let func_to_apply = func.try_into_func()?;

            for value in values_to_map {
                results.push(func_to_apply.apply(vec![value.clone()])?);
            }

            return Ok(MalType::list(results));
        });
        Self::add_param_list_func(env.clone(), "apply", &|params| {
            let func = params[0].clone();
            let mut args = vec![];
            params[1..params.len() - 1]
                .iter()
                .for_each(|v| args.push(v.clone()));
            params
                .last()
                .unwrap()
                .get_as_vec()?
                .iter()
                .for_each(|v| args.push(v.clone()));

            func.try_into_func()?.apply(args)

            //Repl::eval2(MalType::list(args), env.clone())
        });

        Self::add_unary_func(env.clone(), "nil?", &|a| Ok(MalType::bool(a.is_nil())));
        Self::add_unary_func(env.clone(), "true?", &|a| Ok(MalType::bool(a.is_true())));
        Self::add_unary_func(env.clone(), "false?", &|a| Ok(MalType::bool(a.is_false())));
        Self::add_unary_func(env.clone(), "symbol?", &|a| {
            Ok(MalType::bool(a.is_symbol()))
        });

        Self::add_unary_func(env.clone(), "symbol", &|a| {
            Ok(MalType::symbol(a.try_into_string()?))
        });
        Self::add_unary_func(env.clone(), "keyword", &|a| match a.is_keyword() {
            true => Ok(a),
            false => Ok(Rc::new(MalType::Keyword(a.try_into_string()?))),
        });
        Self::add_unary_func(env.clone(), "keyword?", &|a| {
            Ok(MalType::bool(a.is_keyword()))
        });

        Self::add_param_list_func(env.clone(), "vector", &|vals| {
            Ok(Rc::new(MalType::Vector(vals)))
        });
        Self::add_unary_func(env.clone(), "vector?", &|a| {
            Ok(MalType::bool(a.is_vector()))
        });

        Self::add_unary_func(env.clone(), "sequential?", &|a| match a.get_as_vec() {
            Ok(_) => Ok(MalType::bool(true)),
            Err(_) => Ok(MalType::bool(false)),
        });

        Self::add_unary_func(env.clone(), "map?", &|a| Ok(MalType::bool(a.is_hashmap())));

        Self::add_param_list_func(env.clone(), "hash-map", &|vals| {
            if vals.len() % 2 != 0 {
                return Err(MalError::ParseError(
                    "hash-map must be provided an even number of args".to_string(),
                ));
            }

            let mut map: IndexMap<Rc<MalType>, Rc<MalType>> = IndexMap::new();
            for pairs in vals.chunks_exact(2) {
                map.insert(pairs[0].clone(), pairs[1].clone());
            }

            Ok(Rc::new(MalType::Hashmap(map)))
        });

        Self::add_param_list_func(env.clone(), "assoc", &|vals| {
            let map = vals[0].try_into_hashmap()?;

            let remaining_args: Vec<_> = vals[1..].iter().collect();
            if remaining_args.len() % 2 != 0 {
                return Err(MalError::ParseError(
                    "apply must be provided an even number of args".to_string(),
                ));
            }

            let mut new_map: IndexMap<Rc<MalType>, Rc<MalType>> = IndexMap::new();
            for (k, v) in map {
                new_map.insert(k, v);
            }

            for pairs in remaining_args.chunks_exact(2) {
                new_map.insert(pairs[0].clone(), pairs[1].clone());
            }

            return Ok(Rc::new(MalType::Hashmap(new_map)));
        });

        Self::add_param_list_func(env.clone(), "dissoc", &|vals| {
            let map = vals[0].try_into_hashmap()?;

            let mut new_map: IndexMap<Rc<MalType>, Rc<MalType>> = IndexMap::new();
            for (k, v) in map {
                new_map.insert(k, v);
            }

            for key in vals[1..].iter() {
                new_map.remove(key);
            }

            return Ok(Rc::new(MalType::Hashmap(new_map)));
        });

        Self::add_binary_func(env.clone(), "get", &|map, key| {
            if map.is_nil() {
                return Ok(Rc::new(MalType::Nil));
            }
            let m = map.try_into_hashmap()?;

            match m.get(&key) {
                Some(v) => Ok(v.clone()),
                None => Ok(Rc::new(MalType::Nil)),
            }
        });

        Self::add_binary_func(env.clone(), "contains?", &|map, key| {
            let m = map.try_into_hashmap()?;

            Ok(MalType::bool(m.contains_key(&key)))
        });

        Self::add_unary_func(env.clone(), "keys", &|a| {
            let map = a.try_into_hashmap()?;

            let keys: Vec<_> = map.keys().map(|v| v.clone()).collect();

            Ok(MalType::list(keys))
        });
        Self::add_unary_func(env.clone(), "vals", &|a| {
            let map = a.try_into_hashmap()?;

            let values: Vec<_> = map.values().map(|v| v.clone()).collect();

            Ok(MalType::list(values))
        });

        Self::add_unary_func(env.clone(), "readline", &|prompt| {
            let mut rl = rustyline::Editor::<()>::new();
            let readline = rl.readline(prompt.try_into_string()?.as_str());

            match readline {
                Ok(input) => Ok(MalType::new_string(input)),
                Err(_) => Ok(Rc::new(MalType::Nil)),
            }
        });

        // time-ms,
        Self::add_no_args(env.clone(), "time-ms", &|| {
            let ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Can't get system time")
                .as_millis() as isize;
            Ok(MalType::number(ms))
        });
        // meta,
        Self::add_unary_func(env.clone(), "meta", &|_val| {
            Err(MalError::Exception(MalType::new_string(
                "Not implemented".to_string(),
            )))
        });
        // with-meta,
        Self::add_binary_func(env.clone(), "with-meta", &|_val, _meta| {
            Err(MalError::Exception(MalType::new_string(
                "Not implemented".to_string(),
            )))
        });
        // fn?
        Self::add_unary_func(env.clone(), "fn?", &|f| Ok(MalType::bool(f.is_func())));
        // string?,
        Self::add_unary_func(env.clone(), "string?", &|f| {
            Ok(MalType::bool(f.is_string()))
        });
        // number?,
        Self::add_unary_func(env.clone(), "number?", &|f| {
            Ok(MalType::bool(f.is_number()))
        });
        // seq,
        Self::add_unary_func(env.clone(), "seq", &|val| match val.as_ref() {
            MalType::List(vec) | MalType::Vector(vec) => {
                if vec.is_empty() {
                    return Ok(Rc::new(MalType::Nil));
                }
                Ok(MalType::list(vec.clone()))
            }
            MalType::String(s) => {
                if s.is_empty() {
                    return Ok(Rc::new(MalType::Nil));
                }
                let chars: Vec<_> = s
                    .chars()
                    .map(|c| MalType::new_string(String::from(c)))
                    .collect();
                Ok(MalType::list(chars))
            }
            MalType::Nil => Ok(Rc::new(MalType::Nil)),
            _ => Err(MalError::Exception(MalType::new_string(
                "seq can only be called on List, Vector, String or Nil".to_string(),
            ))),
        });
        // conj
        Self::add_param_list_func(env.clone(), "conj", &|params| {
            let list_or_vector = params[0].clone();
            let to_add = params.iter().skip(1).map(|v| v.clone());

            match list_or_vector.as_ref() {
                MalType::List(list) => {
                    let mut new_list = vec![];
                    to_add.rev().for_each(|v| new_list.push(v.clone()));
                    list.iter().for_each(|v| new_list.push(v.clone()));
                    Ok(Rc::new(MalType::Vector(new_list)))
                }
                MalType::Vector(vector) => {
                    let mut new_vector = vec![];
                    vector.iter().for_each(|v| new_vector.push(v.clone()));
                    to_add.for_each(|v| new_vector.push(v.clone()));
                    Ok(Rc::new(MalType::Vector(new_vector)))
                }
                _ => Err(MalError::Exception(MalType::new_string(
                    "conj can only be called on List or Vector".to_string(),
                ))),
            }
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

    fn add_param_list_func_with_env(
        env: Rc<RefCell<Env>>,
        name: &str,
        func: &'static dyn Fn(Vec<Rc<MalType>>, MalEnv) -> Result<Rc<MalType>, MalError>,
    ) {
        let body = |env: Rc<RefCell<Env>>,
                    _body: Rc<MalType>,
                    _params: Vec<Rc<MalType>>,
                    param_values: Vec<Rc<MalType>>|
         -> Result<Rc<MalType>, MalError> { func(param_values, env) };

        let malfunc = Rc::new(MalType::Func(MalFunc::new_with_closure(
            Some(name.to_string()),
            vec![],
            body,
            env.clone(),
            Rc::new(MalType::Nil),
        )));

        env.borrow_mut().set(name.to_string(), malfunc);
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

    fn add_binary_func_with_env(
        env: Rc<RefCell<Env>>,
        name: &str,
        func: &'static dyn Fn(Rc<MalType>, Rc<MalType>, MalEnv) -> Result<Rc<MalType>, MalError>,
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
            let func_env = Env::new_with_outer(Some(params), Some(param_values), env.clone());
            let lhs = func_env.borrow().get("lhs".to_string())?;
            let rhs = func_env.borrow().get("rhs".to_string())?;
            func(lhs, rhs, env)
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

    fn add_unary_func_with_env(
        env: Rc<RefCell<Env>>,
        name: &str,
        func: &'static dyn Fn(Rc<MalType>, MalEnv) -> Result<Rc<MalType>, MalError>,
    ) {
        let params = vec![Rc::new(MalType::Symbol("a".to_string()))];

        let body = |env: Rc<RefCell<Env>>,
                    _body: Rc<MalType>,
                    params: Vec<Rc<MalType>>,
                    param_values: Vec<Rc<MalType>>|
         -> Result<Rc<MalType>, MalError> {
            let func_env = Env::new_with_outer(Some(params), Some(param_values), env.clone());
            let a = func_env.borrow().get("a".to_string())?;
            func(a, env)
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

    fn add_no_args(
        env: Rc<RefCell<Env>>,
        name: &str,
        func: &'static dyn Fn() -> Result<Rc<MalType>, MalError>,
    ) {
        let params = vec![];

        let body = |_env: Rc<RefCell<Env>>,
                    _body: Rc<MalType>,
                    _params: Vec<Rc<MalType>>,
                    _param_values: Vec<Rc<MalType>>|
         -> Result<Rc<MalType>, MalError> { func() };

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
