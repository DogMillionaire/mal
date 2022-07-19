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
                    .set(l[1].clone().try_into_symbol()?, value.clone());
                Ok(value)
            }
            MalType::Symbol(s) if s == "let*" => {
                let new_env = Env::new(None, None, Some(env));

                let bindings_list = l[1].clone().try_into_list()?;

                let bindings = bindings_list.chunks_exact(2);

                for binding in bindings {
                    let key = binding[0].clone();
                    let value = eval(binding[1].clone(), new_env.clone())?;

                    {
                        let mut mut_env = new_env.borrow_mut();
                        mut_env.set(key.to_string(), value.clone());
                    }
                }

                eval(l[2].clone(), new_env)
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
                apply(func_ast, env)
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
                apply(ast, env)
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn print(input: MalType) -> String {
    Printer::pr_str(input)
}

fn add_func(env: Rc<RefCell<Env>>, name: String, value: MalFn) {
    env.borrow_mut()
        .set(name.clone(), MalType::Func(name, value))
}

fn rep(input: String, env: Rc<RefCell<Env>>) -> Result<String, MalError> {
    add_func(
        env.clone(),
        "+".to_string(),
        Rc::new(&|a: MalType, b: MalType| {
            MalType::Number(a.try_into_number().unwrap() + b.try_into_number().unwrap())
        }),
    );
    add_func(
        env.clone(),
        "-".to_string(),
        Rc::new(&|a: MalType, b: MalType| {
            MalType::Number(a.try_into_number().unwrap() - b.try_into_number().unwrap())
        }),
    );
    add_func(
        env.clone(),
        "*".to_string(),
        Rc::new(&|a: MalType, b: MalType| {
            MalType::Number(a.try_into_number().unwrap() * b.try_into_number().unwrap())
        }),
    );
    add_func(
        env.clone(),
        "/".to_string(),
        Rc::new(&|a: MalType, b: MalType| {
            MalType::Number(a.try_into_number().unwrap() / b.try_into_number().unwrap())
        }),
    );

    let read_result = read(input)?;
    let eval_result = eval(read_result, env)?;
    Ok(print(eval_result))
}

type MalFn = Rc<dyn Fn(MalType, MalType) -> MalType>;

fn main() {
    let mut rl = rustyline::Editor::<()>::new();
    let _result = rl.load_history("history.txt");

    let env = Env::new(None, None, None);

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
