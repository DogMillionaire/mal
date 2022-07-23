use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    env::{Env, MalEnv},
    printer::{Printer},
    reader::{Reader},
    types::{self, MalFn, MalType}, malerror::MalError,
};

#[allow(unused_must_use)]
#[allow(unused_macros)]
#[cfg(debug_assertions)]
macro_rules! debug {
    ($x:expr) => {
        dbg!($x)
    };
}

#[allow(unused_macros)]
#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($x:expr) => {
        std::convert::identity($x)
    };
}

#[allow(dead_code)]
pub struct Repl {
    env: Rc<RefCell<Env>>,
}

#[allow(dead_code)]
impl Repl {
    pub fn new(bindings: Option<Vec<Rc<MalType>>>, expressions: Option<Vec<Rc<MalType>>>) -> Self {
        let env = Env::new_root(bindings, expressions);
        env.as_ref().borrow_mut().set_root(Some(env.clone()));
        Self { env }
    }

    pub fn env(&self) -> Rc<RefCell<Env>> {
        self.env.clone()
    }

    pub fn rep(&mut self, input: String) -> Result<String, MalError> {
        let read_result = Self::read(input)?;
        let eval_result = Self::eval(read_result, self.env.clone())?;
        Ok(Self::print(eval_result))
    }

    fn eval(ast: Rc<MalType>, env: Rc<RefCell<Env>>) -> Result<Rc<MalType>, MalError> {
        // println!("eval={}", printer::Printer::pr_str(ast.as_ref(), true));
        let mut current_ast = ast.clone();
        let mut current_env = env.clone();
        loop {
            current_ast = Self::macroexpand(current_ast.clone(), current_env.clone())?;
            match current_ast.clone().as_ref() {
                MalType::List(l) => {
                    if l.is_empty() {
                        return Ok(current_ast);
                    } else {
                        match current_ast.clone().as_ref() {
                            MalType::List(l) if l.len() > 0 => match l[0].clone().as_ref() {
                                MalType::Symbol(s) if s == "def!" => {
                                    let value = Self::eval(l[2].clone(), current_env.clone())?;
                                    current_env
                                        .as_ref()
                                        .borrow_mut()
                                        .set(l[1].clone().try_into_symbol()?, value.clone());
                                    return Ok(value);
                                }
                                MalType::Symbol(s) if s == "defmacro!" => {
                                    let value = Self::eval(l[2].clone(), current_env.clone())?;
            
                                    value.clone().as_func().unwrap().set_is_macro();
            
                                    current_env
                                        .as_ref()
                                        .borrow_mut()
                                        .set(l[1].clone().try_into_symbol()?, value.clone());
                                    return Ok(value);
                                }
                                MalType::Symbol(s) if s == "let*" => {
                                    let new_env = Env::new_with_outer(None, None, current_env.clone());
            
                                    let bindings_list = l[1].clone().try_into_list()?;
            
                                    let bindings = bindings_list.chunks_exact(2);
            
                                    for binding in bindings {
                                        let key = binding[0].clone();
                                        let value = Self::eval(binding[1].clone(), new_env.clone())?;
            
                                        {
                                            let mut mut_env = new_env.as_ref().borrow_mut();
                                            mut_env.set(key.to_string(), value.clone());
                                        }
                                    }
            
                                    current_ast = l[2].clone();
                                    current_env = new_env;
                                }
                                MalType::Symbol(s) if s == "if" => {
                                    let condition = l[1].clone();
            
                                    let c_value = Self::eval(condition, current_env.clone())?;
            
                                    match c_value.as_ref() {
                                        MalType::Nil | MalType::False => {
                                            if l.len() < 4 {
                                                return Ok(Rc::new(MalType::Nil));
                                            }
                                            let false_value = l[3].clone();
                                            current_ast = false_value;
                                        }
                                        _ => {
                                            let true_value = l[2].clone();
                                            current_ast = true_value;
                                        }
                                    }
                                }
                                MalType::Symbol(s) if s == "fn*" => {
                                    let func_parameters = l[1].clone();
                                    let func_body = l[2].clone();
            
                                    let mal = types::MalFunc::new(
                                        None,
                                        func_parameters.try_into_list()?,
                                        current_env,
                                        func_body,
                                    );
            
                                    return Ok(Rc::new(MalType::Func(mal)));
                                }
                                MalType::Symbol(s) if s == "do" => {
                                    for i in 1..l.len() - 1 {
                                        Self::eval(l[i].clone(), current_env.clone())?;
                                    }
                                    current_ast = l.last().expect("Expected non empty list for 'do'").clone();
                                }
                                MalType::Symbol(s) if s == "eval" => {
                                    let symbol_to_eval = l[1].clone();
                                    let params = vec![Rc::new(MalType::Symbol("input".to_string()))];
            
                                    let body: &MalFn =
                                        &|env: Rc<RefCell<Env>>,
                                          _body: Rc<MalType>,
                                          _params: Vec<Rc<MalType>>,
                                          param_values: Vec<Rc<MalType>>| {
                                            let input = param_values[0].clone();
            
                                            let a = Self::eval(input, env.clone())?;
                                            let root_env = env.borrow().get_root().unwrap();
                                            Self::eval(a, root_env)
                                        };
            
                                    let mal = types::MalFunc::new_with_closure(
                                        Some("eval-closure".to_string()),
                                        params,
                                        body,
                                        current_env.clone(),
                                        Rc::new(MalType::Nil),
                                    );
            
                                    current_ast =
                                        MalType::list(vec![Rc::new(MalType::Func(mal)), symbol_to_eval]);
                                }
                                MalType::Symbol(s) if s == "swap!" => {
                                    let atom_symbol = l[1].clone().try_into_symbol()?;
                                    let atom_type = current_env.borrow().get(atom_symbol)?;
                                    let atom = atom_type.try_into_atom()?;
                                    let func = l[2].clone();
            
                                    let atom_value = atom.borrow().clone();
            
                                    let mut func_ast = vec![func, atom_value];
                                    l[3..].iter().for_each(|f| func_ast.push(f.clone()));
            
                                    let new_value = Self::eval(MalType::list(func_ast), current_env.clone())?;
                                    atom.replace(new_value.clone());
            
                                    return Ok(new_value);
                                }
                                MalType::Symbol(s) if s == "quote" => {
                                    return Ok(l[1].clone());
                                }
                                MalType::Symbol(s) if s == "quasiquote" => {
                                    current_ast = Self::quasiquote(l[1].clone(), current_env.clone())?;
                                }
                                MalType::Symbol(s) if s == "quasiquoteexpand" => {
                                    return Self::quasiquote(l[1].clone(), current_env.clone());
                                }
                                MalType::Symbol(s) if s == "macroexpand" => {
                                    return Self::macroexpand(l[1].clone(), current_env.clone());
                                }
                                MalType::Symbol(s) if s == "try*" => {
                                    let body = l[1].clone();

                                    match Self::eval(body, env.clone()){
                                        Ok(ast) => current_ast = ast,
                                        Err(e) if e.is_exception() => {
                                            if l.len() < 2 {
                                                // No catch block
                                                return Err(e);
                                            }

                                            let catch = l[2].clone().try_into_list()?;

                                            let catch_symbol = catch[0].try_into_symbol()?;
                                            let _exception_symbol = catch[1].try_into_symbol()?;
                                            let catch_body = catch[2].clone();

                                            let catch_bindings = vec![
                                                MalType::symbol(catch_symbol)
                                            ];
                                            let catch_expressions = vec![
                                                e.as_exception().unwrap()
                                            ];

                                            let catch_env = Env::new_with_outer(Some(catch_bindings), Some(catch_expressions), env.clone());
                                            current_ast = Self::eval(catch_body, catch_env)?;
                                        },
                                        Err(e) => return Err(e) 
                                    }
                                }
                                MalType::Symbol(s) if s == "apply" => {
                                    if l.len() < 2 {
                                        return Err(MalError::ParseError(
                                            "Apply needs at least 2 arguments".to_string(),
                                        ));
                                    }

                                    let func = l[1].clone();
                                    let mut func_ast: Vec<Rc<MalType>> = vec![func];

                                    l[2..l.len()-1].iter().for_each(|v| func_ast.push(v.clone()));
                                    let arg_vec = l[l.len()-1].get_as_vec()?;

                                    arg_vec.iter().for_each(|v| func_ast.push(v.clone()));

                                    current_ast = MalType::list(func_ast);
                                }
                                MalType::Symbol(s) if s == "map" => {
                                    if l.len() < 2 {
                                        return Err(MalError::ParseError(
                                            "Map needs at least 2 arguments".to_string(),
                                        ));
                                    }

                                    let func = l[1].clone();
                                    let args = l[2].get_as_vec()?;

                                    let mut results: Vec<Rc<MalType>> = vec![];
                                    for arg in args {
                                        let func_ast = MalType::list(vec![
                                            func.clone(),
                                            arg
                                        ]);
                                        let mapped_val = Self::eval(func_ast, current_env.clone())?;
                                        results.push(mapped_val);
                                    }

                                    return Ok(MalType::list(results));
                                }
                                MalType::Func(func) => {
                                    let args = l[1..l.len()].to_vec();
            
                                    // Built-in func
                                    if let Some(f) = func.body() {
                                        return f(
                                            func.env(),
                                            func.body_ast(),
                                            func.parameters().to_vec(),
                                            args,
                                        );
                                    }
            
                                    let exec_env = Env::new_with_outer(
                                        Some(func.parameters().to_vec()),
                                        Some(args),
                                        func.env(),
                                    );
            
                                    // def!/defmacro! function
                                    current_ast = func.body_ast();
                                    current_env = exec_env;
                                }
                                MalType::Number(_) | MalType::String(_) | MalType::True | MalType::False => {
                                    // A list of some data type
                                    return Ok(current_ast);
                                }
                                _ => {
                                    current_ast = Self::eval_ast(current_ast, current_env.clone())?;
                                }
                            },
                            _ => {
                                return Self::eval(current_ast, current_env);
                            }
                        }
                    }
                }
                _ => return Self::eval_ast(current_ast, current_env),
            }
        }
    }

    fn print(input: Rc<MalType>) -> String {
        Printer::pr_str(input.as_ref(), true)
    }

    fn macroexpand(ast: Rc<MalType>, env: MalEnv) -> Result<Rc<MalType>, MalError> {
        // println!(
        //     "\t==>macroexpand={}",
        //     printer::Printer::pr_str(ast.as_ref(), true)
        // );
        let mut current_ast = ast.clone();
        loop {
            if let Some((func, func_args)) = Self::get_macro_call(current_ast.clone(), env.clone())
            {
                let mut func_list = vec![func];
                func_args.iter().for_each(|a| func_list.push(a.clone()));
                let apply_result = Self::eval(MalType::list(func_list), env.clone())?;
                current_ast = apply_result;
            } else {
                break;
            }
        }

        // println!(
        //     "\t<==macroexpand={}",
        //     printer::Printer::pr_str(current_ast.as_ref(), true)
        // );
        return Ok(current_ast.clone());
    }

    fn get_macro_symbol(symbol: Rc<MalType>, env: MalEnv) -> Option<Rc<MalType>> {
        if let Ok(symbol) = symbol.try_into_symbol() {
            if let Ok(s) = env.borrow().get(symbol) {
                if let Ok(func) = s.try_into_func() {
                    if func.is_macro() {
                        return Some(s);
                    }
                    return None;
                }
            }
        }

        return None;
    }

    fn get_macro_call(ast: Rc<MalType>, env: MalEnv) -> Option<(Rc<MalType>, Vec<Rc<MalType>>)> {
        // This function takes arguments ast and env.
        // It returns true if ast is a list that contains a symbol as the first element
        //  and that symbol refers to a function in the env environment and that
        // function has the is_macro attribute set to true. Otherwise, it returns false.
        match ast.as_ref() {
            MalType::List(l) if l.len() > 0 => {
                if let Some(func) = Self::get_macro_symbol(l[0].clone(), env) {
                    let args: Vec<_> = l[1..].iter().map(|v| v.clone()).collect();
                    return Some((func, args));
                }
                return None;
            }
            _ => return None,
        };
    }

    fn is_list_starting_with_symbol(ast: Rc<MalType>, symbol: &str) -> bool {
        match ast.clone().as_ref() {
            MalType::List(list) if !list.is_empty() => {
                if let Ok(s) = list[0].try_into_symbol() {
                    return s == symbol;
                }
                false
            }
            _ => false,
        }
    }

    fn quasiquote(ast: Rc<MalType>, env: MalEnv) -> Result<Rc<MalType>, MalError> {
        match ast.clone().as_ref() {
            MalType::List(list) => {
                if Self::is_list_starting_with_symbol(ast, "unquote") {
                    // If ast is a list starting with the "unquote" symbol, return its second element.
                    return Ok(list[1].clone());
                } else {
                    // If ast is a list failing previous test, the result will be a list populated by the following process.
                    let mut current_result: Rc<MalType> = MalType::list(vec![]);
                    // The result is initially an empty list. Iterate over each element elt of ast in reverse order:
                    for elt in list.iter().rev() {
                        if Self::is_list_starting_with_symbol(elt.clone(), "splice-unquote") {
                            // If elt is a list starting with the "splice-unquote" symbol,
                            // replace the current result with a list containing:
                            //  the "concat" symbol, the second element of elt, then the previous result.

                            let elt_list = elt.try_into_list()?;

                            current_result = MalType::list(vec![
                                MalType::symbol("concat".to_string()),
                                elt_list[1].clone(),
                                current_result,
                            ]);
                        } else {
                            // Else replace the current result with a list containing:
                            // the "cons" symbol, the result of calling quasiquote with elt as argument,
                            // then the previous result.
                            current_result = MalType::list(vec![
                                MalType::symbol("cons".to_string()),
                                Self::quasiquote(elt.clone(), env.clone())?,
                                current_result,
                            ]);
                        }
                    }

                    return Ok(current_result);
                }
            }
            MalType::Vector(v) => {
                if v.is_empty() {
                    return Ok(MalType::list(vec![
                        MalType::symbol("vec".to_string()),
                        MalType::list(vec![]),
                    ]));
                }
                let mut result_list = vec![MalType::symbol("vec".to_string())];
                let quasicote_result = Self::quasiquote(MalType::list(v.clone()), env.clone())?;

                let mut results: Vec<Rc<MalType>> = vec![];
                quasicote_result
                    .get_as_vec()?
                    .iter()
                    .for_each(|v| results.push(v.clone()));

                result_list.push(MalType::list(results));

                return Ok(MalType::list(result_list));
            }
            MalType::Symbol(_) | MalType::Hashmap(_) => {
                // If ast is a map or a symbol, return a list containing: the "quote" symbol, then ast.
                return Ok(MalType::list(vec![
                    MalType::symbol("quote".to_string()),
                    ast.clone(),
                ]));
            }
            // Else return ast unchanged. Such forms are not affected by evaluation, so you may quote them as in the previous case if implementation is easier.
            _ => return Ok(ast),
        };
    }

    fn read(input: String) -> Result<Rc<MalType>, MalError> {
        let mut reader = Reader::read_str(input)?;
        let result = reader.read_form();
        result
    }

    fn eval_ast(ast: Rc<MalType>, env: Rc<RefCell<Env>>) -> Result<Rc<MalType>, MalError> {
        match ast.as_ref() {
            MalType::Symbol(name) => env.borrow().get(name.to_string()),
            MalType::List(list) => {
                let mut new_ast: Vec<Rc<MalType>> = Vec::with_capacity(list.len());
                for value in list {
                    let new_value = Self::eval(value.clone(), env.clone())?;
                    new_ast.push(new_value);
                }
                Ok(Rc::new(MalType::List(new_ast)))
            }
            MalType::Vector(vector) => {
                let mut new_ast: Vec<Rc<MalType>> = Vec::with_capacity(vector.len());
                for value in vector {
                    let new_value = Self::eval(value.clone(), env.clone())?;
                    new_ast.push(new_value);
                }
                Ok(Rc::new(MalType::Vector(new_ast)))
            }
            MalType::Hashmap(hashmap) => {
                let mut new_ast: HashMap<Rc<MalType>, Rc<MalType>> =
                    HashMap::with_capacity(hashmap.len());

                for (key, value) in hashmap {
                    let new_value = Self::eval(value.clone(), env.clone())?;
                    new_ast.insert(key.clone(), new_value);
                }
                Ok(Rc::new(MalType::Hashmap(new_ast)))
            }
            _ => Ok(ast),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::Repl;
    use crate::{malcore::MalCore, types::MalType};

    #[test]
    fn eval_do() {
        let ast = Repl::read("(do (prn 101))".to_string()).expect("Expected read to succeed");
        let repl = Repl::new(None, None);
        MalCore::add_to_env(repl.env());

        let eval_result = Repl::eval(ast, repl.env()).expect("Expected evaluation to succeed");

        assert_matches!(eval_result.as_ref(), MalType::List(_l));
    }

    #[test]
    fn test_repl_do() {
        let mut repl = Repl::new(None, None);
        MalCore::add_to_env(repl.env());

        let result = repl
            .rep("(do (prn 101))".to_string())
            .expect("Evaluation should succeed");

        assert_eq!("101\nnil", result);
    }
}
