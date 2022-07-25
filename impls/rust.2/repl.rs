use std::{cell::RefCell, rc::Rc};

use indexmap::IndexMap;
use log::debug;

use crate::{
    env::{Env, MalEnv},
    malerror::MalError,
    printer::Printer,
    reader::Reader,
    types::{self, MalType},
};

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
        let eval_result = Self::eval2(read_result, self.env.clone())?;
        Ok(Self::print(eval_result))
    }

    pub fn eval2(ast: Rc<MalType>, env: Rc<RefCell<Env>>) -> Result<Rc<MalType>, MalError> {
        // while true:
        let mut current_ast = ast;
        let mut current_env = env;
        loop {
            debug!("eval: {}", current_ast);
            // println!("eval={:?}", current_ast);
            //     if not list?(ast): return eval_ast(ast, env)
            if !current_ast.is_list() {
                return Self::eval_ast(current_ast, current_env.clone());
            }
            //     ast = macroexpand(ast, env)
            current_ast = Self::macroexpand(current_ast, current_env.clone())?;
            //     if not list?(ast): return eval_ast(ast, env)
            let ast_list = match current_ast.try_into_list() {
                Ok(list) => list,
                Err(_) => return Self::eval_ast(current_ast, current_env.clone()),
            };

            //     if empty?(ast): return ast
            if ast_list.is_empty() {
                return Ok(current_ast);
            }
            //     switch ast[0]:
            match ast_list[0].as_ref() {
                //     'def!:        return env.set(ast[1], EVAL(ast[2], env))
                MalType::Symbol(s) if s == "def!" => {
                    let val = Self::eval2(ast_list[2].clone(), current_env.clone())?;
                    current_env
                        .borrow_mut()
                        .set(ast_list[1].try_into_symbol().unwrap(), val.clone());
                    return Ok(val);
                }
                //     'let*:        env = ...; ast = ast[2] // TCO
                MalType::Symbol(s) if s == "let*" => {
                    let new_env = Env::new_with_outer(None, None, current_env.clone());

                    let bindings_list = ast_list[1].clone().try_into_list()?;

                    let bindings = bindings_list.chunks_exact(2);

                    for binding in bindings {
                        let key = binding[0].clone();
                        let value = Self::eval2(binding[1].clone(), new_env.clone())?;

                        {
                            let mut mut_env = new_env.as_ref().borrow_mut();
                            mut_env.set(key.to_string(), value.clone());
                        }
                    }

                    current_ast = ast_list[2].clone();
                    current_env = new_env;
                }
                //     'quote:       return ast[1]
                MalType::Symbol(s) if s == "quote" => {
                    return Ok(ast_list[1].clone());
                }
                //     'quasiquote:  ast = quasiquote(ast[1]) // TCO
                MalType::Symbol(s) if s == "quasiquote" => {
                    current_ast = Self::quasiquote(ast_list[1].clone(), current_env.clone())?;
                }
                MalType::Symbol(s) if s == "quasiquoteexpand" => {
                    return Self::quasiquote(ast_list[1].clone(), current_env.clone());
                }
                //     'defmacro!:   return ... // like def!, but set macro property
                MalType::Symbol(s) if s == "defmacro!" => {
                    let val = Self::eval2(ast_list[2].clone(), current_env.clone())?;

                    let new_func = val.try_into_func()?.clone();
                    new_func.set_is_macro();

                    current_env.borrow_mut().set(
                        ast_list[1].try_into_symbol().unwrap(),
                        Rc::new(MalType::Func(new_func, None)),
                    );
                    return Ok(val);
                }
                //     'macroexpand: return macroexpand(ast[1], env)
                MalType::Symbol(s) if s == "macroexpand" => {
                    return Self::macroexpand(ast_list[1].clone(), current_env.clone());
                }
                //     'try*:        return ... // try/catch native and malval exceptions
                MalType::Symbol(s) if s == "try*" => {
                    match Self::eval2(ast_list[1].clone(), current_env.clone()) {
                        Ok(ast) => current_ast = ast,
                        Err(e) if e.is_exception() => {
                            if ast_list.len() < 3 {
                                // No catch block
                                return Err(e);
                            }

                            let catch = ast_list[2].clone().try_into_list()?;
                            let exception_symbol = catch[1].try_into_symbol()?;
                            let catch_body = catch[2].clone();

                            let catch_env = Env::new_with_outer(
                                Some(vec![MalType::symbol(exception_symbol)]),
                                Some(vec![e
                                    .as_exception()
                                    .expect("Expected MalError::Exception")]),
                                current_env.clone(),
                            );
                            current_ast = Self::eval2(catch_body, catch_env.clone())?;
                        }
                        Err(e) => return Err(e),
                    }
                }
                //     'do:          ast = eval_ast(ast[1..-1], env)[-1] // TCO
                MalType::Symbol(s) if s == "do" => {
                    let args: Vec<_> = ast_list[1..ast_list.len()]
                        .iter()
                        .map(|v| v.clone())
                        .collect();
                    let val = Self::eval_ast(MalType::new_list(args), current_env.clone())?;
                    current_ast = val
                        .try_into_list()?
                        .last()
                        .unwrap_or(&Rc::new(MalType::Nil))
                        .clone();
                }
                //     'if:          EVAL(ast[1], env) ? ast = ast[2] : ast = ast[3] // TCO
                MalType::Symbol(s) if s == "if" => {
                    let cond = Self::eval2(ast_list[1].clone(), current_env.clone())?;
                    if !cond.is_false() && !cond.is_nil() {
                        current_ast = ast_list[2].clone();
                    } else {
                        if ast_list.len() < 4 {
                            return Ok(Rc::new(MalType::Nil));
                        }
                        current_ast = ast_list[3].clone();
                    }
                }
                //     'fn*:         return new MalFunc(...)
                MalType::Symbol(s) if s == "fn*" => {
                    let func_parameters = ast_list[1].clone();
                    let func_body = ast_list[2].clone();

                    let mal = types::MalFunc::new_with_closure(
                        None,
                        func_parameters.try_into_list()?,
                        &|env,
                          body: Rc<MalType>,
                          params: Vec<Rc<MalType>>,
                          param_values: Vec<Rc<MalType>>| {
                            Self::eval2(
                                body.clone(),
                                Env::new_with_outer(
                                    Some(params.clone()),
                                    Some(param_values.clone()),
                                    env,
                                ),
                            )
                        },
                        current_env,
                        func_body,
                    );

                    return Ok(Rc::new(MalType::Func(mal, None)));
                }
                //     _default_:    f, args = eval_ast(ast, env)
                _ => {
                    let func_args = Self::eval_ast(current_ast.clone(), current_env.clone())?
                        .try_into_list()?;

                    let f = func_args.first().unwrap().clone();
                    let args: Vec<_> = func_args.iter().skip(1).map(|v| v.clone()).collect();

                    //                     if malfunc?(f): ast = f.fn; env = ... // TCO
                    if func_args[0].is_func() {
                        let func = func_args[0].try_into_func().unwrap();

                        if let Some(f) = func.body() {
                            return f.as_ref()(
                                func.env(),
                                func.body_ast(),
                                func.parameters(),
                                args,
                            );
                        }

                        current_env = Env::new_with_outer(
                            Some(func.parameters().clone()),
                            Some(args),
                            current_env.clone(),
                        );
                        current_ast = func.body_ast();
                    } else if f.is_func() {
                        return f.try_into_func()?.apply(args);
                        //                     else:           return apply(f, args)
                        //return Self::apply(f.clone(), args);
                    } else {
                        return Ok(current_ast);
                    }
                }
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
                let mut func_list = vec![];
                func_args.iter().for_each(|a| func_list.push(a.clone()));
                let apply_result = func.try_into_func()?.apply(func_args)?;
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
            MalType::List(l, _) if l.len() > 0 => {
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
            MalType::List(list, _) if !list.is_empty() => {
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
            MalType::List(list, _) => {
                if Self::is_list_starting_with_symbol(ast, "unquote") {
                    // If ast is a list starting with the "unquote" symbol, return its second element.
                    return Ok(list[1].clone());
                } else {
                    // If ast is a list failing previous test, the result will be a list populated by the following process.
                    let mut current_result: Rc<MalType> = MalType::new_list(vec![]);
                    // The result is initially an empty list. Iterate over each element elt of ast in reverse order:
                    for elt in list.iter().rev() {
                        if Self::is_list_starting_with_symbol(elt.clone(), "splice-unquote") {
                            // If elt is a list starting with the "splice-unquote" symbol,
                            // replace the current result with a list containing:
                            //  the "concat" symbol, the second element of elt, then the previous result.

                            let elt_list = elt.try_into_list()?;

                            current_result = MalType::new_list(vec![
                                MalType::symbol("concat".to_string()),
                                elt_list[1].clone(),
                                current_result,
                            ]);
                        } else {
                            // Else replace the current result with a list containing:
                            // the "cons" symbol, the result of calling quasiquote with elt as argument,
                            // then the previous result.
                            current_result = MalType::new_list(vec![
                                MalType::symbol("cons".to_string()),
                                Self::quasiquote(elt.clone(), env.clone())?,
                                current_result,
                            ]);
                        }
                    }

                    return Ok(current_result);
                }
            }
            MalType::Vector(v, _) => {
                if v.is_empty() {
                    return Ok(MalType::new_list(vec![
                        MalType::symbol("vec".to_string()),
                        MalType::new_list(vec![]),
                    ]));
                }
                let mut result_list = vec![MalType::symbol("vec".to_string())];
                let quasicote_result = Self::quasiquote(MalType::new_list(v.clone()), env.clone())?;

                let mut results: Vec<Rc<MalType>> = vec![];
                quasicote_result
                    .get_as_vec()?
                    .iter()
                    .for_each(|v| results.push(v.clone()));

                result_list.push(MalType::new_list(results));

                return Ok(MalType::new_list(result_list));
            }
            MalType::Symbol(_) | MalType::Hashmap(_, _) => {
                // If ast is a map or a symbol, return a list containing: the "quote" symbol, then ast.
                return Ok(MalType::new_list(vec![
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
        debug!("eval_ast: {}", ast);
        match ast.as_ref() {
            MalType::Symbol(name) => env.borrow().get(name.to_string()),
            MalType::List(list, _) => {
                let mut new_ast: Vec<Rc<MalType>> = Vec::with_capacity(list.len());
                for value in list {
                    let new_value = Self::eval2(value.clone(), env.clone())?;
                    new_ast.push(new_value);
                }
                Ok(MalType::new_list(new_ast))
            }
            MalType::Vector(vector, _) => {
                let mut new_ast: Vec<Rc<MalType>> = Vec::with_capacity(vector.len());
                for value in vector {
                    let new_value = Self::eval2(value.clone(), env.clone())?;
                    new_ast.push(new_value);
                }
                Ok(MalType::new_vector(new_ast))
            }
            MalType::Hashmap(hashmap, _) => {
                let mut new_ast: IndexMap<Rc<MalType>, Rc<MalType>> =
                    IndexMap::with_capacity(hashmap.len());

                for (key, value) in hashmap {
                    let new_value = Self::eval2(value.clone(), env.clone())?;
                    new_ast.insert(key.clone(), new_value);
                }
                Ok(MalType::new_hashmap(new_ast))
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

        let eval_result = Repl::eval2(ast, repl.env()).expect("Expected evaluation to succeed");

        assert_matches!(eval_result.as_ref(), MalType::List(_, _));
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
