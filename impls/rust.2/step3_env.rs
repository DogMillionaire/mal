mod env;
mod printer;
mod reader;
mod types;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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

fn eval_ast(ast: MalType, env: Rc<RefCell<Env>>) -> Result<MalType, MalError> {
    debug!(&ast);
    match ast {
        MalType::Symbol(name) => env.borrow().get(name),
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

fn apply(ast: MalType, env: Rc<RefCell<Env>>) -> Result<MalType, MalError> {
    debug!(&ast);

    //eval_ast(ast, env.clone())?

    match ast.clone() {
        MalType::List(l) => match l[0].clone() {
            MalType::Symbol(s) if s == "def!" => {
                let value = eval(l[2].clone(), env.clone())?;
                env.borrow_mut()
                    .set(String::from(l[1].clone()), value.clone());
                return Ok(value);
            }
            MalType::Symbol(s) if s == "let*" => {
                let clone_env = env.clone();
                let new_env = Env::new(Some(env));

                let bindings_list = Vec::<MalType>::from(l[1].clone());

                let bindings = bindings_list.chunks_exact(2);

                let mut keys: Vec<String> = Vec::with_capacity(bindings.len());
                let mut new_values: Vec<MalType> = Vec::with_capacity(bindings.len());

                for binding in bindings {
                    let key = binding[0].clone();
                    let value = eval(binding[1].clone(), new_env.clone())?;

                    keys.push(String::from(key));
                    new_values.push(value.clone());
                }

                {
                    let mut mut_env = clone_env.borrow_mut();
                    for (key, value) in keys.iter().zip(new_values.iter()) {
                        mut_env.set(key.to_string(), value.clone());
                    }
                }

                return eval(l[2].clone(), clone_env.clone());
            }
            MalType::Func(name, func) => {
                debug!(format!(
                    "Executing func: {} with {:?} and {:?}",
                    name,
                    l[1].clone(),
                    l[2].clone()
                ));
                return Ok(func.as_ref()(l[1].clone(), l[2].clone()));
            }
            _ => {
                let func_ast = eval_ast(ast, env.clone())?;
                return apply(func_ast, env.clone());
            }
        },
        _ => Err(MalError::ParseError(
            "eval_ast did not return a list!".to_string(),
        )),
    }
}

fn eval(ast: MalType, env: Rc<RefCell<Env>>) -> Result<MalType, MalError> {
    debug!(&ast);
    match ast.clone() {
        MalType::List(l) => {
            if l.is_empty() {
                Ok(ast)
            } else {
                apply(ast, env.clone())
            }
        }
        _ => eval_ast(ast, env.clone()),
    }
}

fn print(input: MalType) -> String {
    Printer::pr_str(input)
}

fn add_func(env: Rc<RefCell<Env>>, name: String, value: MalFn) {
    env.borrow_mut()
        .set(name.clone(), MalType::Func(name.clone(), value))
}

fn rep(input: String, env: Rc<RefCell<Env>>) -> Result<String, MalError> {
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

type MalFn = Rc<dyn Fn(MalType, MalType) -> MalType>;

fn main() {
    let mut rl = rustyline::Editor::<()>::new();
    let _result = rl.load_history("history.txt");

    let env = Env::new(None);

    loop {
        let readline = rl.readline("user> ");

        match readline {
            Ok(input) => match rep(input.clone(), env.clone()) {
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
