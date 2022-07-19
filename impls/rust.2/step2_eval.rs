mod env;
mod printer;
mod reader;
mod repl;
mod types;

use std::collections::HashMap;
use std::rc::Rc;

use reader::MalError;

use crate::printer::Printer;
use crate::reader::Reader;
use crate::types::MalType;

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

fn eval_ast(ast: Rc<MalType>, env: &HashMap<String, NumericFn>) -> Result<Rc<MalType>, MalError> {
    debug!(&ast);
    match ast.as_ref() {
        MalType::Symbol(name) => match env.get(&name.to_string()) {
            Some(v) => Ok(Rc::new(MalType::Func(name.to_string(), v.clone()))),
            None => Err(MalError::SymbolNotFound(name.to_string())),
        },
        MalType::List(list) => {
            let mut new_ast: Vec<Rc<MalType>> = Vec::with_capacity(list.len());
            for value in list {
                let new_value = eval(value.clone(), env)?;
                new_ast.push(new_value);
            }
            Ok(Rc::new(MalType::List(new_ast)))
        }
        MalType::Vector(vector) => {
            let mut new_ast: Vec<Rc<MalType>> = Vec::with_capacity(vector.len());
            for value in vector {
                let new_value = eval(value.clone(), env)?;
                new_ast.push(new_value);
            }
            Ok(Rc::new(MalType::Vector(new_ast)))
        }
        MalType::Hashmap(hashmap) => {
            let mut new_ast: HashMap<Rc<MalType>, Rc<MalType>> =
                HashMap::with_capacity(hashmap.len());

            for (key, value) in hashmap {
                let new_value = eval(value.clone(), env)?;
                new_ast.insert(key.clone(), new_value);
            }
            Ok(Rc::new(MalType::Hashmap(new_ast)))
        }
        _ => Ok(ast),
    }
}

fn call_func(ast: Rc<MalType>) -> Result<Rc<MalType>, MalError> {
    match ast.as_ref() {
        MalType::List(l) => {
            assert!(
                l.len() >= 3,
                "Expected eval_ast to retun a list with at least 3 elements"
            );
            if let MalType::Func(name, func) = l[0].clone().as_ref() {
                debug!(format!(
                    "Executing func: {} with {:?} and {:?}",
                    name,
                    l[1].clone(),
                    l[2].clone()
                ));
                return Ok(func.as_ref()(l[1].clone(), l[2].clone()));
            }
            Err(MalError::ParseError(
                "First list element was not a function!".to_string(),
            ))
        }
        _ => Err(MalError::ParseError(
            "eval_ast did not return a list!".to_string(),
        )),
    }
}

fn eval(ast: Rc<MalType>, env: &HashMap<String, NumericFn>) -> Result<Rc<MalType>, MalError> {
    debug!(&ast);
    match ast.clone().as_ref() {
        MalType::List(l) => {
            if l.is_empty() {
                Ok(ast)
            } else {
                call_func(eval_ast(ast, env)?)
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn print(input: Rc<MalType>) -> String {
    Printer::pr_str(&input, true)
}

fn rep(input: String) -> Result<String, MalError> {
    let mut env: HashMap<String, NumericFn> = HashMap::new();

    env.insert(
        "+".to_string(),
        Rc::new(&|a: Rc<MalType>, b: Rc<MalType>| {
            Rc::new(MalType::Number(
                a.try_into_number().unwrap() + b.try_into_number().unwrap(),
            ))
        }),
    );
    env.insert(
        "-".to_string(),
        Rc::new(&|a: Rc<MalType>, b: Rc<MalType>| {
            Rc::new(MalType::Number(
                a.try_into_number().unwrap() - b.try_into_number().unwrap(),
            ))
        }),
    );
    env.insert(
        "/".to_string(),
        Rc::new(&|a: Rc<MalType>, b: Rc<MalType>| {
            Rc::new(MalType::Number(
                a.try_into_number().unwrap() / b.try_into_number().unwrap(),
            ))
        }),
    );
    env.insert(
        "*".to_string(),
        Rc::new(&|a: Rc<MalType>, b: Rc<MalType>| {
            Rc::new(MalType::Number(
                a.try_into_number().unwrap() * b.try_into_number().unwrap(),
            ))
        }),
    );

    let read_result = read(input)?;
    let eval_result = eval(read_result, &env)?;
    Ok(print(eval_result))
}

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
