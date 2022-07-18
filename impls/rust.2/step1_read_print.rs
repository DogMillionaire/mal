mod env;
mod printer;
mod reader;
mod types;

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

fn eval(input: MalType) -> MalType {
    input
}

fn print(input: MalType) -> String {
    Printer::pr_str(input)
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
