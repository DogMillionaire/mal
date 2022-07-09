mod printer;
mod reader;
mod types;

use reader::TokenizeError;

use crate::printer::Printer;
use crate::reader::Reader;
use crate::types::MalType;

fn READ(input: String) -> Result<MalType, TokenizeError> {
    let mut reader = Reader::read_str(input)?;
    let result = reader.read_form();
    //eprintln!("Read reult: {:?}", result);
    result
}

fn EVAL(input: MalType) -> MalType {
    input
}

fn PRINT(input: MalType) -> String {
    Printer::pr_str(input)
}

fn rep(input: String) -> Result<String, TokenizeError> {
    let read_result = READ(input)?;
    let eval_result = EVAL(read_result);
    Ok(PRINT(eval_result))
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
