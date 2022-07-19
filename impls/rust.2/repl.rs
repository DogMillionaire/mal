use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    env::Env,
    printer::Printer,
    reader::{MalError, Reader},
    types::{self, MalType},
};

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

pub struct Repl {
    env: Rc<RefCell<Env>>,
}

impl Repl {
    pub fn new(bindings: Option<Vec<Rc<MalType>>>, expressions: Option<Vec<Rc<MalType>>>) -> Self {
        Self {
            env: Env::new(bindings, expressions, None),
        }
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
        //debug!(&ast);
        match ast.clone().as_ref() {
            MalType::List(l) => {
                if l.is_empty() {
                    Ok(ast)
                } else {
                    Self::apply(ast, env)
                }
            }
            _ => Self::eval_ast(ast, env),
        }
    }

    fn print(input: Rc<MalType>) -> String {
        Printer::pr_str(input.as_ref(), true)
    }

    fn execute(
        func: &types::MalFunc,
        param_values: Vec<Rc<MalType>>,
    ) -> Result<Rc<MalType>, MalError> {
        if func.parameters().len() > param_values.len() {
            return Err(MalError::IncorrectParamCount(
                func.name(),
                func.parameters().len(),
                param_values.len(),
            ));
        }

        let exec_env = Env::new(
            Some(func.parameters().to_vec()),
            Some(param_values.clone()),
            Some(func.env()),
        );

        func.body()(exec_env, func.body_ast(), param_values)
    }

    fn apply(ast: Rc<MalType>, env: Rc<RefCell<Env>>) -> Result<Rc<MalType>, MalError> {
        //debug!(&ast);

        //eval_ast(ast, env.clone())?

        match ast.clone().as_ref() {
            MalType::List(l) => match l[0].clone().as_ref() {
                MalType::Symbol(s) if s == "def!" => {
                    let value = Self::eval(l[2].clone(), env.clone())?;
                    env.borrow_mut()
                        .set(l[1].clone().try_into_symbol()?, value.clone());
                    Ok(value)
                }
                MalType::Symbol(s) if s == "let*" => {
                    let new_env = Env::new(None, None, Some(env));

                    let bindings_list = l[1].clone().try_into_list()?;

                    let bindings = bindings_list.chunks_exact(2);

                    for binding in bindings {
                        let key = binding[0].clone();
                        let value = Self::eval(binding[1].clone(), new_env.clone())?;

                        {
                            let mut mut_env = new_env.borrow_mut();
                            mut_env.set(key.to_string(), value.clone());
                        }
                    }

                    Self::eval(l[2].clone(), new_env)
                }
                MalType::Symbol(s) if s == "if" => {
                    let condition = l[1].clone();

                    let c_value = Self::eval(condition, env.clone())?;

                    match c_value.as_ref() {
                        MalType::Nil | MalType::False => {
                            if l.len() < 4 {
                                return Ok(Rc::new(MalType::Nil));
                            }
                            let false_value = l[3].clone();
                            Self::eval(false_value, env)
                        }
                        _ => {
                            let true_value = l[2].clone();
                            Self::eval(true_value, env)
                        }
                    }
                }
                MalType::Symbol(s) if s == "fn*" => {
                    let func_parameters = l[1].clone();
                    let func_body = l[2].clone();

                    let body = |env: Rc<RefCell<Env>>,
                                body_ast: Rc<MalType>,
                                _params: Vec<Rc<MalType>>|
                     -> Result<Rc<MalType>, MalError> {
                        Self::eval(body_ast, env)
                    };

                    let mal = types::MalFunc::new(
                        None,
                        func_parameters.try_into_list()?,
                        body,
                        env,
                        func_body.clone(),
                    );

                    // let func_env = Env::new(
                    //     Some(func_parameters),
                    //     Some(func_parameters_values),
                    //     Some(env),
                    // );

                    Ok(Rc::new(MalType::Func(mal)))
                }
                MalType::Symbol(s) if s == "do" => {
                    let mut value: Rc<MalType> = Rc::new(MalType::Nil);
                    for i in 1..l.len() {
                        value = Self::eval_ast(l[i].clone(), env.clone())?;
                    }
                    Ok(value)
                }
                MalType::Func(func) => {
                    let params = l[1..l.len()].to_vec();
                    Self::execute(func, params)
                }
                _ => {
                    let func_ast = Self::eval_ast(ast, env.clone())?;
                    Self::apply(func_ast, env)
                }
            },
            _ => Err(MalError::ParseError(
                "eval_ast did not return a list!".to_string(),
            )),
        }
    }

    fn read(input: String) -> Result<Rc<MalType>, MalError> {
        let mut reader = Reader::read_str(input)?;
        let result = reader.read_form();
        debug!(&result);
        result
    }

    fn eval_ast(ast: Rc<MalType>, env: Rc<RefCell<Env>>) -> Result<Rc<MalType>, MalError> {
        //debug!(&ast);
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
