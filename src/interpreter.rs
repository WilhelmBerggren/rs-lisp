#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinKind {
    Eager,
    SpecialForm,
}

#[derive(Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub func: fn(&[Expr], &mut Scope) -> Result<Expr, String>,
    pub kind: BuiltinKind,
}

impl BuiltinFunction {
    pub fn new(
        name: impl Into<String>,
        func: fn(&[Expr], &mut Scope) -> Result<Expr, String>,
        kind: BuiltinKind,
    ) -> Self {
        BuiltinFunction {
            name: name.into(),
            func,
            kind,
        }
    }
}

impl fmt::Debug for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BuiltinFunction {{ name: {} }}", self.name)
    }
}

impl PartialEq for BuiltinFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub parameters: Vec<String>,
    pub body: Box<Expr>,
    pub closure: Rc<Scope>,
}

impl Function {
    fn new(parameters: Vec<String>, body: Box<Expr>, closure: Rc<Scope>) -> Self {
        Function {
            parameters,
            body,
            closure,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Symbol(String),
    Number(f64),
    List(Vec<Expr>),
    Lambda(Vec<String>, Box<Expr>),
    Function(Rc<Function>),
    BuiltinFunction(BuiltinFunction),
}

impl Expr {
    pub fn symbol(s: impl Into<String>) -> Self {
        Expr::Symbol(s.into())
    }

    pub fn number(n: f64) -> Self {
        Expr::Number(n)
    }

    pub fn list(expressions: Vec<Expr>) -> Self {
        Expr::List(expressions)
    }

    pub fn lambda(parameters: Vec<String>, body: Expr) -> Self {
        Expr::Lambda(parameters, Box::new(body))
    }
}

use core::fmt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    variables: HashMap<String, Expr>,
    parent: Option<Rc<Scope>>,
}

impl Default for Scope {
    fn default() -> Self {
        Scope::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            variables: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<Scope>) -> Self {
        Scope {
            variables: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn set_variable(&mut self, name: String, value: Expr) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Expr> {
        match self.variables.get(name) {
            Some(value) => Some(value),
            None => match &self.parent {
                Some(parent) => parent.get_variable(name),
                None => None,
            },
        }
    }
}

fn apply_function(func: Expr, args: Vec<Expr>, scope: &mut Scope) -> Result<Expr, String> {
    match func {
        Expr::Function(func) => {
            if args.len() != func.parameters.len() {
                return Err("Argument count does not match parameter count".to_string());
            }

            let mut local_scope = Scope::with_parent(scope.clone().into());
            for (param, arg) in func.parameters.iter().zip(args) {
                local_scope.set_variable(param.clone(), arg);
            }

            eval(&func.body, &mut local_scope)
        }
        Expr::BuiltinFunction(builtin) => {
            let evaluated_args: Result<Vec<_>, _> =
                args.into_iter().map(|arg| eval(&arg, scope)).collect();

            // Execute builtin function
            (builtin.func)(&evaluated_args?, scope)
        }
        _ => Err("First argument to apply is not a function".to_string()),
    }
}

pub fn eval(expr: &Expr, scope: &mut Scope) -> Result<Expr, String> {
    match expr {
        Expr::List(list) => {
            if list.is_empty() {
                return Err("Cannot evaluate an empty list".to_string());
            }

            let first = &list[0];
            let evaluated_first = eval(first, scope)?;

            match evaluated_first {
                Expr::Lambda(parameters, body) => {
                    if list.len() != parameters.len() + 1 {
                        return Err("Argument count does not match parameter count".to_string());
                    }

                    let mut local_scope = Scope::with_parent(scope.clone().into());
                    for (param, arg) in parameters.iter().zip(&list[1..]) {
                        local_scope.set_variable(param.clone(), eval(arg, scope)?);
                    }

                    eval(&body, &mut local_scope)
                }
                Expr::Function(func) => {
                    let args: Result<Vec<_>, _> =
                        list[1..].iter().map(|arg| eval(arg, scope)).collect();
                    apply_function(Expr::Function(func), args?, scope)
                }
                Expr::BuiltinFunction(builtin_func) => {
                    match builtin_func.kind {
                        BuiltinKind::Eager => {
                            let args: Result<Vec<_>, _> =
                                list[1..].iter().map(|arg| eval(arg, scope)).collect();
                            (builtin_func.func)(&args?, scope)
                        }
                        BuiltinKind::SpecialForm => {
                            // For special forms, pass the raw arguments
                            (builtin_func.func)(&list[1..], scope)
                        }
                    }
                }
                _ => Err("First element in the list is not a function or special form".to_string()),
            }
        }
        Expr::Number(_) => Ok(expr.clone()), // Numbers evaluate to themselves
        Expr::Symbol(name) => {
            // Look up symbols in the scope
            match scope.get_variable(name) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Undefined symbol '{}'", name)),
            }
        }
        Expr::Function(function) => Ok(Expr::Function(function.clone())),
        Expr::BuiltinFunction(_) => Ok(expr.clone()),
        Expr::Lambda(parameters, body) => Ok(Expr::Function(Rc::new(Function::new(
            parameters.clone(),
            body.clone(),
            Rc::new(scope.clone()),
        )))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::initialize_global_scope;
    use crate::interpreter::Expr;

    #[test]
    fn parse_quote() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        assert_eq!(
            eval(
                &Expr::list(vec![Expr::symbol("quote"), Expr::symbol("x")]),
                &mut global_scope
            ),
            Ok(Expr::symbol("x"))
        );
    }

    #[test]
    fn scope() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        global_scope.set_variable("x".to_string(), Expr::number(42.0));
        assert_eq!(
            eval(&Expr::symbol("x"), &mut global_scope),
            Ok(Expr::number(42.0))
        );
    }

    #[test]
    fn scope_parent() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        global_scope.set_variable("x".to_string(), Expr::number(42.0));
        let mut scope = Scope::with_parent(Rc::new(global_scope));
        assert_eq!(eval(&Expr::symbol("x"), &mut scope), Ok(Expr::number(42.0)));
    }

    #[test]
    fn lambda() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        global_scope.set_variable("x".to_string(), Expr::number(42.0));
        assert_eq!(
            eval(
                &Expr::lambda(vec!["x".to_string()], Expr::symbol("x")),
                &mut global_scope
            ),
            Ok(Expr::Function(Rc::new(Function::new(
                vec!["x".to_string()],
                Box::new(Expr::symbol("x")),
                Rc::new(global_scope.clone())
            ))))
        );
    }

    #[test]
    fn lambda_call() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        let lambda = Expr::lambda(vec!["x".to_string()], Expr::symbol("x"));

        // Convert the lambda expression into a function object
        let function = eval(&lambda, &mut global_scope).unwrap();

        // Apply the function (e.g., (func 42))
        let application = Expr::list(vec![function, Expr::number(42.0)]);
        let result = eval(&application, &mut global_scope);

        assert_eq!(result, Ok(Expr::number(42.0)));
    }

    #[test]
    fn if_call() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        let if_expr = Expr::list(vec![
            Expr::symbol("if"),
            Expr::number(1.0),
            Expr::number(42.0),
            Expr::number(0.0),
        ]);

        let result = eval(&if_expr, &mut global_scope);

        assert_eq!(result, Ok(Expr::number(42.0)));
    }

    #[test]
    fn define() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        let define_expr = Expr::list(vec![
            Expr::symbol("def"),
            Expr::symbol("x"),
            Expr::number(42.0),
        ]);

        let result = eval(&define_expr, &mut global_scope);

        assert_eq!(result, Ok(Expr::symbol("x")));

        assert_eq!(global_scope.get_variable("x"), Some(&Expr::number(42.0)));
    }

    #[test]
    fn quote() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        let quote_expr = Expr::list(vec![Expr::symbol("quote"), Expr::Number(42.0)]);

        let result = eval(&quote_expr, &mut global_scope);

        assert_eq!(result, Ok(Expr::number(42.0)));
    }

    #[test]
    fn quote_list() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        let quote_expr = Expr::list(vec![
            Expr::symbol("quote"),
            Expr::list(vec![
                Expr::symbol("def"),
                Expr::symbol("x"),
                Expr::number(42.0),
            ]),
        ]);

        let result = eval(&quote_expr, &mut global_scope);

        assert_eq!(
            result,
            Ok(Expr::List(vec![
                Expr::symbol("def"),
                Expr::symbol("x"),
                Expr::number(42.0),
            ]))
        );
    }

    #[test]
    fn first() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        let first_expr = Expr::List(vec![
            Expr::symbol("first"),
            Expr::List(vec![
                Expr::symbol("quote"),
                Expr::List(vec![Expr::number(1.0), Expr::number(2.0)]),
            ]),
        ]);

        let result = eval(&first_expr, &mut global_scope);

        assert_eq!(result, Ok(Expr::number(1.0)));
    }

    #[test]
    fn rest() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        let first_expr = Expr::list(vec![
            Expr::symbol("rest"),
            Expr::list(vec![
                Expr::symbol("quote"),
                Expr::list(vec![Expr::number(1.0), Expr::number(2.0)]),
            ]),
        ]);

        let result = eval(&first_expr, &mut global_scope);

        assert_eq!(result, Ok(Expr::list(vec![Expr::number(2.0)])));
    }

    #[test]
    fn list() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        let list_expr = Expr::list(vec![
            Expr::symbol("list"),
            Expr::number(1.0),
            Expr::number(2.0),
        ]);

        let result = eval(&list_expr, &mut global_scope);

        assert_eq!(
            result,
            Ok(Expr::list(vec![Expr::number(1.0), Expr::number(2.0)]))
        );
    }

    #[test]
    fn apply() {
        let mut global_scope = Scope::new();
        initialize_global_scope(&mut global_scope);
        let apply_expr = Expr::List(vec![
            Expr::symbol("apply"),
            Expr::symbol("+"),
            Expr::list(vec![Expr::number(1.0), Expr::number(2.0)]),
        ]);

        let result = eval(&apply_expr, &mut global_scope);

        assert_eq!(result, Ok(Expr::number(3.0)));
    }
}
