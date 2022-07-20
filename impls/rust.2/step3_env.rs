mod env;
mod malcore;
mod printer;
mod reader;
mod repl;
mod types;

use std::cell::RefCell;

use std::rc::Rc;

use env::Env;
use reader::MalError;

use crate::repl::Repl;
use crate::types::MalType;

fn add_func2(env: Rc<RefCell<Env>>, name: String, value: &'static dyn Fn(isize, isize) -> isize) {
    let params = vec![
        Rc::new(MalType::Symbol("a".to_string())),
        Rc::new(MalType::Symbol("b".to_string())),
    ];

    let body = |env: Rc<RefCell<Env>>,
                _body: Rc<MalType>,
                params: Vec<Rc<MalType>>,
                param_values: Vec<Rc<MalType>>|
     -> Result<Rc<MalType>, MalError> {
        let func_env = Env::new(Some(params), Some(param_values), Some(env));
        let a = func_env.borrow().get("a".to_string())?.try_into_number()?;
        let b = func_env.borrow().get("b".to_string())?.try_into_number()?;
        Ok(Rc::new(MalType::Number(value(a, b))))
    };

    let malfunc = types::MalFunc::new_with_closure(
        Some(name.clone()),
        params,
        body,
        env.clone(),
        Rc::new(MalType::Nil),
    );

    env.borrow_mut().set(name, Rc::new(MalType::Func(malfunc)))
}

fn main() {
    let mut rl = rustyline::Editor::<()>::new();
    let _result = rl.load_history("history.txt");

    let mut repl = Repl::new(None, None);

    add_func2(repl.env(), "+".to_string(), &|a, b| a + b);
    add_func2(repl.env(), "-".to_string(), &|a, b| a - b);
    add_func2(repl.env(), "/".to_string(), &|a, b| a / b);
    add_func2(repl.env(), "*".to_string(), &|a, b| a * b);

    loop {
        let readline = rl.readline("user> ");

        match readline {
            Ok(input) => match repl.rep(input.clone()) {
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
