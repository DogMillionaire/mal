extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod core;
mod env;
mod printer;
mod reader;
mod types;

use std::rc::Rc;
use types::{MalError, MalResult, MalValue};

fn read(input: String) -> Result<Option<MalValue>, MalError> {
    return reader::Reader::read_str(input.to_string());
}

fn to_int(value: MalValue) -> Result<i32, MalError> {
    match value {
        MalValue::MalInteger(int) => {
            return Ok(int);
        }
        _ => {
            return Err(types::MalError::EvalError(String::from(
                "value is not an integer",
            )));
        }
    }
}

pub fn exec_fn(func: MalValue, args: Vec<MalValue>, env: Rc<env::Environment>) -> MalResult {
    match func {
        MalValue::MalFunction(f, _) => f(args),
        MalValue::MalFunc {
            eval,
            ref ast,
            ref env,
            ref params,
            ..
        } => {
            let a = &**ast;
            let fn_env = env::Environment::new(Some(env.clone()), params.as_vec(), Some(args));
            Ok(eval(a.clone(), Rc::new(fn_env))?)
        }
        _ => Err(MalError::EvalError(
            "Attempt to call non-function".to_string(),
        )),
    }
}

fn apply(ast: MalValue, env: Rc<env::Environment>) -> MalResult {
    match ast {
        MalValue::MalList(list) => {
            let evaluated_result = eval_ast(types::MalValue::MalList(list), env.clone())?;
            match evaluated_result {
                MalValue::MalList(list) => {
                    let func = list.first().unwrap();
                    let args = list.clone().split_off(1);

                    return exec_fn(func.clone(), args, env.clone());
                }
                _ => {
                    todo!("Handle eval_ast returning something that is not a list!");
                }
            }
        }
        _ => {
            return Err(MalError::EvalError(String::from(
                "apply called on non-list",
            )));
        }
    }
}

fn run_let(env: Rc<env::Environment>, bindings: MalValue, e: MalValue) -> MalResult {
    let new_env = Rc::new(env::Environment::new(Some(env.clone()), None, None));

    if let Some(binding_list) = bindings.as_vec() {
        assert!(binding_list.len() % 2 == 0);
        let mut key = binding_list.first().unwrap();

        for i in 0..binding_list.len() {
            if i % 2 == 0 {
                key = binding_list.get(i).unwrap();
            } else {
                new_env.set(
                    key.clone(),
                    eval(binding_list.get(i).unwrap().clone(), new_env.clone())?,
                );
            }
        }

        return eval(e, new_env);
    } else {
        return Err(MalError::EvalError(String::from("Missing bindings!")));
    }
}

fn handle_fn(ast: Vec<MalValue>, env: Rc<env::Environment>) -> MalResult {
    let func = MalValue::MalFunc {
        eval: eval,
        ast: Rc::new(ast[2].clone()),
        env: env,
        params: Rc::new(ast[1].clone()),
    };
    return Ok(func);
}

fn eval(ast: MalValue, env: Rc<env::Environment>) -> MalResult {
    match ast.clone() {
        MalValue::MalList(list) => {
            if list.len() > 0 {
                match list.first().unwrap() {
                    MalValue::MalSymbol(ref s) => {
                        if s == "fn*" {
                            return handle_fn(list, env);
                        } else if s == "do" {
                            for token in &list[1..list.len() - 1] {
                                eval(token.clone(), env.clone())?;
                            }

                            return eval(list.last().unwrap().clone(), env);
                        } else if s == "if" {
                            assert!(list.len() >= 3);
                            let test = list[1].clone();
                            let true_expr = list[2].clone();
                            let test_result = eval(test, env.clone())?;
                            if test_result.is_truthy() {
                                return eval(true_expr, env.clone());
                            } else if (list.len() == 4) {
                                return eval(list[3].clone(), env.clone());
                            }
                            return Ok(MalValue::MalNil);
                        } else if s == "def!" {
                            assert_eq!(3, list.len());
                            let key = list[1].clone();
                            let value = eval(list[2].clone(), env.clone())?;
                            env.set(key, value.clone())?;
                            return Ok(value);
                        } else if s == "let*" {
                            assert_eq!(3, list.len());
                            let bindings = list[1].clone();
                            let e = list[2].clone();
                            return run_let(env, bindings, e);
                        } else {
                            return apply(ast, env);
                        }
                    }
                    _ => {
                        return apply(ast, env);
                    }
                }
            }
            return Ok(ast);
        }
        _ => {
            return eval_ast(ast, env);
        }
    }
}

fn eval_ast(ast: MalValue, env: Rc<env::Environment>) -> Result<MalValue, MalError> {
    match ast {
        MalValue::MalSymbol(_) => {
            if let Ok(func) = env.get(ast.clone()) {
                return Ok(func);
            }
            return Err(types::MalError::EvalError(format!(
                "Symbol '{}' not found",
                ast.inspect(false)
            )));
        }
        MalValue::MalList(list) => {
            let mut result = Vec::<types::MalValue>::new();

            for token in list {
                result.push(eval(token, env.clone())?);
            }

            return Ok(types::MalValue::MalList(result));
        }
        MalValue::MalVector(vector) => {
            let mut result = Vec::<types::MalValue>::new();

            for token in vector {
                result.push(eval(token, env.clone())?);
            }

            return Ok(types::MalValue::MalVector(result));
        }
        MalValue::MalHashmap(keys, values) => {
            let mut result = Vec::<types::MalValue>::new();

            for token in values {
                result.push(eval(token, env.clone())?);
            }

            return Ok(types::MalValue::MalHashmap(keys, result));
        }
        _ => {
            return Ok(ast);
        }
    }
}

fn print(input: MalValue) -> String {
    return crate::printer::pr_str(input, true);
}

fn rep(input: String, env: &Rc<env::Environment>) -> String {
    let ast = read(input);
    match ast {
        Err(e) => {
            return format!("Error! {:?}", e);
        }
        Ok(v) => match v {
            None => return String::default(),
            Some(a) => match eval(a, env.clone()) {
                Ok(result) => {
                    return print(result);
                }
                Err(e) => {
                    return format!("Error! {:?}", e);
                }
            },
        },
    }
}

fn main() {
    let mut rl = Editor::<()>::new();
    if rl.load_history(".mal-history").is_err() {
        println!("No previous history.");
    }

    let env = Rc::new(env::Environment::new(None, None, None));

    for (symbol, func) in crate::core::ns() {
        env.set(MalValue::MalSymbol(symbol.to_string()), func);
    }

    rep("(def! not (fn* (a) (if a false true)))".to_string(), &env);

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let result = rep(line.trim_end().to_string(), &env);
                print!("{}", result);
                println!();
            }
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
