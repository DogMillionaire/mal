mod env;
mod malcore;
mod malerror;
mod printer;
mod reader;
mod repl;
mod types;

use crate::repl::Repl;
use crate::types::MalType;

#[allow(unused_macros)]
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

fn main() {
    let mut rl = rustyline::Editor::<()>::new();
    let _result = rl.load_history("history.txt");

    let mut repl = Repl::new(None, None);

    malcore::MalCore::add_to_env(repl.env());
    repl.rep("(def! not (fn* (a) (if a false true)))".to_string())
        .expect("Fail to parse def! not");

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
