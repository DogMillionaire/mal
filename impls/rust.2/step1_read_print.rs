mod env;
mod printer;
mod reader;
mod repl;
mod types;

use std::rc::Rc;

use reader::MalError;

use crate::printer::Printer;
use crate::reader::Reader;
use crate::types::MalType;

fn read(input: String) -> Result<Rc<MalType>, MalError> {
    let mut reader = Reader::read_str(input)?;

    //eprintln!("Read reult: {:?}", result);
    reader.read_form()
}

fn eval(input: Rc<MalType>) -> Rc<MalType> {
    input
}

fn print(input: Rc<MalType>) -> String {
    Printer::pr_str(&input, true)
}

fn rep(input: String) -> Result<String, MalError> {
    let read_result = read(input)?;
    let eval_result = eval(read_result);
    Ok(print(eval_result))
}

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
