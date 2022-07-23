mod env;
mod malcore;
mod malerror;
mod printer;
mod reader;
mod repl;
mod types;

use std::rc::Rc;

use crate::repl::Repl;
use crate::types::MalType;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut rl = rustyline::Editor::<()>::new();
    let _result = rl.load_history("history.txt");

    let mut repl = Repl::new(None, None);

    malcore::MalCore::add_to_env(repl.env());
    repl.rep("(def! not (fn* (a) (if a false true)))".to_string())
        .expect("Fail to parse def! not");

    repl.rep(
        r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))"#
            .to_string(),
    )
    .expect("Fail to parse def! load-file");

    repl.rep(r#"(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw "odd number of forms to cond")) (cons 'cond (rest (rest xs)))))))"#.to_string()).expect("Failed to parse defmacro! cond");

    let mut arg_list: Vec<Rc<MalType>> = vec![];
    if args.len() >= 2 {
        arg_list = args[2..]
            .iter()
            .map(|a| MalType::string(a.to_string()))
            .collect();
    }

    repl.env()
        .borrow_mut()
        .set(String::from("*ARGV*"), MalType::list(arg_list));

    if let Some(file) = args.iter().nth(1) {
        match repl.rep(format!("(load-file \"{}\")", file)) {
            Ok(_) => {}
            Err(err) => eprintln!("ERROR: {}", err),
        }
        return;
    }

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
