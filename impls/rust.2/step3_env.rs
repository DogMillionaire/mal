mod env;
mod printer;
mod reader;
mod types;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

use env::Env;
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

fn read(input: String) -> Result<MalType, MalError> {
    let mut reader = Reader::read_str(input)?;
    let result = reader.read_form();
    debug!(&result);
    result
}

fn eval_ast(ast: MalType, env: Rc<RwLock<Env>>) -> Result<MalType, MalError> {
    debug!(&ast);
    match ast {
        MalType::Symbol(name) => env.read().unwrap().get(name),
        MalType::List(list) => {
            let mut new_ast: Vec<MalType> = Vec::with_capacity(list.len());
            for value in list {
                let new_value = eval(value.clone(), env.clone())?;
                new_ast.push(new_value);
            }
            Ok(MalType::List(new_ast))
        }
        MalType::Vector(vector) => {
            let mut new_ast: Vec<MalType> = Vec::with_capacity(vector.len());
            for value in vector {
                let new_value = eval(value.clone(), env.clone())?;
                new_ast.push(new_value);
            }
            Ok(MalType::Vector(new_ast))
        }
        MalType::Hashmap(hashmap) => {
            let mut new_ast: HashMap<MalType, MalType> = HashMap::with_capacity(hashmap.len());

            for (key, value) in hashmap {
                let new_value = eval(value.clone(), env.clone())?;
                new_ast.insert(key.clone(), new_value);
            }
            Ok(MalType::Hashmap(new_ast))
        }
        _ => Ok(ast),
    }
}

fn call_func(ast: MalType) -> Result<MalType, MalError> {
    match ast.clone() {
        MalType::List(l) => {
            assert!(
                l.len() >= 3,
                "Expected eval_ast to retun a list with at least 3 elements"
            );
            match l[0].clone() {
                MalType::Func(name, func) => {
                    debug!(format!(
                        "Executing func: {} with {:?} and {:?}",
                        name,
                        l[1].clone(),
                        l[2].clone()
                    ));
                    return Ok(func.as_ref()(l[1].clone(), l[2].clone()));
                }
                MalType::Symbol(s) => match s.as_str() {
                    "def!" => {}
                    "let*" => {}
                    _ => {}
                },
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

fn eval(ast: MalType, env: Rc<RwLock<Env>>) -> Result<MalType, MalError> {
    debug!(&ast);
    match ast.clone() {
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

fn print(input: MalType) -> String {
    Printer::pr_str(input)
}

fn add_func(env: Rc<RwLock<Env>>, name: String, value: NumericFn) {
    env.write()
        .unwrap()
        .set(name.clone(), MalType::Func(name.clone(), value))
}

fn rep(input: String) -> Result<String, MalError> {
    let env = Rc::new(RwLock::new(Env::new(None)));

    add_func(
        env.clone(),
        "+".to_string(),
        Rc::new(&|a, b| MalType::Number(isize::from(a) + isize::from(b))),
    );
    add_func(
        env.clone(),
        "-".to_string(),
        Rc::new(&|a, b| MalType::Number(isize::from(a) - isize::from(b))),
    );
    add_func(
        env.clone(),
        "*".to_string(),
        Rc::new(&|a, b| MalType::Number(isize::from(a) * isize::from(b))),
    );
    add_func(
        env.clone(),
        "/".to_string(),
        Rc::new(&|a, b| MalType::Number(isize::from(a) / isize::from(b))),
    );

    let read_result = read(input)?;
    let eval_result = eval(read_result, env)?;
    Ok(print(eval_result))
}

type NumericFn = Rc<dyn Fn(MalType, MalType) -> MalType>;

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
