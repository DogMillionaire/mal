mod env;
mod malcore;
mod malerror;
mod printer;
mod reader;
mod repl;
mod types;

use std::cell::RefCell;

use std::rc::Rc;

use env::Env;
use indexmap::IndexMap;

use crate::malerror::MalError;
use crate::printer::Printer;
use crate::reader::Reader;
use crate::types::{MalFunc, MalType};

#[allow(unused_must_use)]
#[cfg(debug_assertions)]
macro_rules! debug {
    ($x:expr) => {
        dbg!($x)
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($x:expr) => {
        std::convert::identity($x)
    };
}

fn read(input: String) -> Result<Rc<MalType>, MalError> {
    let mut reader = Reader::read_str(input)?;
    let result = reader.read_form();
    debug!(&result);
    result
}

fn eval_ast(ast: Rc<MalType>, env: Rc<RefCell<Env>>) -> Result<Rc<MalType>, MalError> {
    debug!(&ast);
    match ast.as_ref() {
        MalType::Symbol(name) => env.borrow().get(name.to_string()),
        MalType::List(list, _) => {
            let mut new_ast: Vec<Rc<MalType>> = Vec::with_capacity(list.len());
            for value in list {
                let new_value = eval(value.clone(), env.clone())?;
                new_ast.push(new_value);
            }
            Ok(MalType::new_list(new_ast))
        }
        MalType::Vector(vector, _) => {
            let mut new_ast: Vec<Rc<MalType>> = Vec::with_capacity(vector.len());
            for value in vector {
                let new_value = eval(value.clone(), env.clone())?;
                new_ast.push(new_value);
            }
            Ok(MalType::new_vector(new_ast))
        }
        MalType::Hashmap(hashmap) => {
            let mut new_ast: IndexMap<Rc<MalType>, Rc<MalType>> =
                IndexMap::with_capacity(hashmap.len());

            for (key, value) in hashmap {
                let new_value = eval(value.clone(), env.clone())?;
                new_ast.insert(key.clone(), new_value);
            }
            Ok(Rc::new(MalType::Hashmap(new_ast)))
        }
        _ => Ok(ast),
    }
}

fn eval(ast: Rc<MalType>, env: Rc<RefCell<Env>>) -> Result<Rc<MalType>, MalError> {
    debug!(&ast);
    match ast.clone().as_ref() {
        MalType::List(l, _) => {
            if l.is_empty() {
                Ok(ast)
            } else {
                let func_ast = eval_ast(l[0].clone(), env.clone())?;
                let func = func_ast.as_func();
                let f = func.unwrap();

                let lhs = eval(l[1].clone(), env.clone())?;
                let rhs = eval(l[2].clone(), env)?;
                let param_values = vec![lhs, rhs];

                let exec_env = Env::new_with_outer(
                    Some(f.parameters().to_vec()),
                    Some(param_values.clone()),
                    f.env(),
                );

                {
                    let mut mut_env = exec_env.borrow_mut();
                    for (param, value) in f.parameters().iter().zip(param_values.iter()) {
                        mut_env.set(param.clone().try_into_symbol()?, value.clone());
                    }
                }

                f.body().unwrap()(exec_env, Rc::new(MalType::Nil), vec![], vec![])
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn print(input: Rc<MalType>) -> String {
    Printer::pr_str(&input, true)
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

    let body = |env: Rc<RefCell<Env>>,
                _body: Rc<MalType>,
                _params: Vec<Rc<MalType>>,
                _param_values: Vec<Rc<MalType>>|
     -> Result<Rc<MalType>, MalError> {
        let func_env = env.borrow();
        let a = func_env.get("a".to_string())?.try_into_number()?;
        let b = func_env.get("b".to_string())?.try_into_number()?;
        Ok(Rc::new(MalType::Number(func(a, b))))
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

fn rep(input: String) -> Result<String, MalError> {
    let env = env::Env::new_root(None, None);

    add_numeric_func(env.clone(), "+", &|a, b| a + b);
    add_numeric_func(env.clone(), "-", &|a, b| a - b);
    add_numeric_func(env.clone(), "/", &|a, b| a / b);
    add_numeric_func(env.clone(), "*", &|a, b| a * b);

    let read_result = read(input)?;
    let eval_result = eval(read_result, env)?;
    Ok(print(eval_result))
}

#[allow(dead_code)]
type NumericFn = Rc<dyn Fn(Rc<MalType>, Rc<MalType>) -> Rc<MalType>>;

fn main() {
    let mut rl = rustyline::Editor::<()>::new();
    let _result = rl.load_history("history.txt");

    loop {
        let readline = rl.readline("user> ");

        match readline {
            Ok(input) => match rep(input.clone()) {
                Ok(result) => {
                    println!("{}", result);
                    rl.add_history_entry(input);
                }
                Err(err) => eprintln!("ERROR: {}", err),
            },
            Err(_) => break,
        }
    }
    rl.save_history("history.txt").unwrap();
}
