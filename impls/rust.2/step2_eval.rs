mod printer;
mod reader;
mod types;

use std::collections::HashMap;

use reader::MalError;

use crate::printer::Printer;
use crate::reader::Reader;
use crate::types::MalType;

fn read(input: String) -> Result<MalType, MalError> {
    let mut reader = Reader::read_str(input)?;
    let result = reader.read_form();
    //eprintln!("Read reult: {:?}", result);
    result
}

fn eval_ast(ast: MalType, env: &HashMap<String, NumericFn>) -> Result<MalType, MalError> {
    match ast {
        MalType::Symbol(name) => match env.get(&name) {
            Some(v) => Ok(MalType::Nil),
            None => Err(MalError::SymbolNotFound(name)),
        },
        MalType::List(list) => {
            let new_ast = list.iter().map(|e| eval(e.clone(), env).unwrap()).collect();
            Ok(MalType::List(new_ast))
        }
        _ => Ok(ast),
    }
}

fn eval(ast: MalType, env: &HashMap<String, NumericFn>) -> Result<MalType, MalError> {
    match ast.clone() {
        MalType::List(l) => {
            if l.is_empty() {
                Ok(ast)
            } else {
                let new_ast = eval_ast(ast, env)?;

                match new_ast.clone() {
                    MalType::List(l) => {
                        assert_eq!(
                            3,
                            l.len(),
                            "Expected eval_ast to retun a list with 3 elements"
                        );
                        let func = l[0].clone();
                        // Call func with rest of params
                    }
                    _ => panic!("eval_ast did not return a list!"),
                }

                Ok(new_ast.clone())
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn print(input: MalType) -> String {
    Printer::pr_str(input)
}

fn try_into_num(value: MalType) -> Result<isize, MalError> {
    match value {
        MalType::Number(n) => Ok(n),
        _ => Err(MalError::InvalidType),
    }
}

fn rep(input: String) -> Result<String, MalError> {
    let mut env: HashMap<String, NumericFn> = HashMap::new();

    env.insert("+".to_string(), &|a, b| {
        MalType::Number(try_into_num(a).unwrap() + try_into_num(b).unwrap())
    });

    let read_result = read(input)?;
    let eval_result = eval(read_result, &env)?;
    Ok(print(eval_result))
}

type NumericFn<'a> = &'a dyn Fn(MalType, MalType) -> MalType;

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
