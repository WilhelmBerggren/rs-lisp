use std::rc::Rc;

use crate::interpreter::{eval, BuiltinFunction, BuiltinKind, Expr, Scope};

fn builtin_add(args: &[Expr], _: &mut Scope) -> Result<Expr, String> {
    let mut result = 0.0;
    for expr in &args[0..] {
        if let Expr::Number(n) = expr {
            result += n;
        } else {
            return Err("Non-numeric argument to +".to_string());
        }
    }
    Ok(Expr::Number(result))
}

fn builtin_apply(args: &[Expr], scope: &mut Scope) -> Result<Expr, String> {
    if args.len() != 2 {
        return Err("apply expects exactly 2 arguments".to_string());
    }

    let func = eval(&args[0], scope)?;
    let arg_list = match &args[1] {
        Expr::List(list) => list,
        _ => return Err("Second argument to apply must be a list".to_string()),
    };

    // Apply the function to the evaluated arguments
    match func {
        Expr::Function(func) => {
            if args.len() != func.parameters.len() {
                return Err("Argument count does not match parameter count".to_string());
            }

            let mut local_scope = Scope::with_parent(Rc::new(scope.clone()));
            for (param, arg) in func.parameters.iter().zip(args) {
                let arg_val = eval(arg, scope)?;
                local_scope.set_variable(param.clone(), arg_val);
            }

            eval(&func.body, &mut local_scope)
        }

        Expr::BuiltinFunction(builtin_func) => {
            match builtin_func.kind {
                BuiltinKind::Eager => {
                    // For eagerly evaluated built-ins
                    let evaluated_args: Result<Vec<_>, _> =
                        arg_list.iter().map(|arg| eval(arg, scope)).collect();
                    (builtin_func.func)(&evaluated_args?, scope)
                }
                BuiltinKind::SpecialForm => {
                    // For special forms, pass the raw arguments
                    (builtin_func.func)(arg_list, scope)
                }
            }
        }
        _ => Err("First argument to apply is not a function".to_string()),
    }
}

fn builtin_list(args: &[Expr], scope: &mut Scope) -> Result<Expr, String> {
    let mut result = Vec::new();
    for expr in &args[0..] {
        match eval(&expr.clone(), scope) {
            Ok(evaluated) => result.push(evaluated),
            Err(e) => return Err(e),
        }
    }
    Ok(Expr::List(result))
}

fn builtin_fn(args: &[Expr], _: &mut Scope) -> Result<Expr, String> {
    if args.len() != 2 {
        return Err("fn expects exactly 2 arguments".to_string());
    }

    let parameters = if let Expr::List(parameters) = &args[0] {
        parameters
            .iter()
            .map(|expr| {
                if let Expr::Symbol(name) = expr {
                    Ok(name.clone())
                } else {
                    Err("Function parameters must be symbols".to_string())
                }
            })
            .collect::<Result<Vec<String>, String>>()?
    } else {
        return Err("Function parameters must be a list".to_string());
    };

    Ok(Expr::lambda(parameters, args[1].clone()))
}

fn builtin_quote(args: &[Expr], _: &mut Scope) -> Result<Expr, String> {
    if args.len() != 1 {
        return Err("quote expects exactly 1 argument".to_string());
    }

    Ok(args[0].clone())
}

fn builtin_def(args: &[Expr], scope: &mut Scope) -> Result<Expr, String> {
    if args.len() != 2 {
        return Err("def expects exactly 2 arguments".to_string());
    }

    let name = if let Expr::Symbol(name) = &args[0] {
        name
    } else {
        return Err("First argument to def must be a symbol".to_string());
    };

    let value = eval(&args[1], scope)?;

    scope.set_variable(name.clone(), value);

    Ok(Expr::Symbol(name.clone()))
}

fn builtin_if(args: &[Expr], scope: &mut Scope) -> Result<Expr, String> {
    if args.len() != 3 {
        return Err("if expects exactly 3 arguments".to_string());
    }

    let condition = eval(&args[0], scope)?;

    match condition {
        Expr::Number(n) => {
            if n == 0.0 {
                eval(&args[2], scope)
            } else {
                eval(&args[1], scope)
            }
        }
        _ => Err("Condition must be a number".to_string()),
    }
}

fn builtin_first(args: &[Expr], _: &mut Scope) -> Result<Expr, String> {
    if args.len() != 1 {
        return Err("first expects exactly 1 argument".to_string());
    }

    let list = match &args[0] {
        Expr::List(list) => list,
        _ => return Err("Argument to first must be a list".to_string()),
    };

    if list.is_empty() {
        return Err("Cannot get first element of empty list".to_string());
    }

    Ok(list[0].clone())
}

fn builtin_rest(args: &[Expr], _: &mut Scope) -> Result<Expr, String> {
    if args.len() != 1 {
        return Err("rest expects exactly 1 argument".to_string());
    }

    let list = match &args[0] {
        Expr::List(list) => list,
        _ => return Err("Argument to first must be a list".to_string()),
    };

    if list.is_empty() {
        return Err("Cannot get first element of empty list".to_string());
    }

    Ok(Expr::List(list[1..].to_vec()))
}

fn builtin_is_number(args: &[Expr], _: &mut Scope) -> Result<Expr, String> {
    if args.len() != 1 {
        return Err("number? expects exactly 1 argument".to_string());
    }

    match &args[0] {
        Expr::Number(_) => Ok(Expr::Number(1.0)),
        _ => Ok(Expr::Number(0.0)),
    }
}

fn builtin_is_symbol(args: &[Expr], _: &mut Scope) -> Result<Expr, String> {
    if args.len() != 1 {
        return Err("symbol? expects exactly 1 argument".to_string());
    }

    match &args[0] {
        Expr::Symbol(_) => Ok(Expr::Number(1.0)),
        _ => Ok(Expr::Number(0.0)),
    }
}

pub fn initialize_global_scope(scope: &mut Scope) {
    scope.set_variable(
        "+".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new("+", builtin_add, BuiltinKind::Eager)),
    );

    scope.set_variable(
        "apply".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "apply",
            builtin_apply,
            BuiltinKind::SpecialForm,
        )),
    );

    scope.set_variable(
        "list".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "list",
            builtin_list,
            BuiltinKind::Eager,
        )),
    );

    scope.set_variable(
        "fn".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "fn",
            builtin_fn,
            BuiltinKind::SpecialForm,
        )),
    );

    scope.set_variable(
        "quote".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "quote".to_string(),
            builtin_quote,
            BuiltinKind::SpecialForm,
        )),
    );

    scope.set_variable(
        "def".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "def",
            builtin_def,
            BuiltinKind::SpecialForm,
        )),
    );

    scope.set_variable(
        "if".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "if",
            builtin_if,
            BuiltinKind::SpecialForm,
        )),
    );

    scope.set_variable(
        "first".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "first",
            builtin_first,
            BuiltinKind::Eager,
        )),
    );

    scope.set_variable(
        "rest".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "rest",
            builtin_rest,
            BuiltinKind::Eager,
        )),
    );

    scope.set_variable(
        "number?".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "number?",
            builtin_is_number,
            BuiltinKind::SpecialForm,
        )),
    );

    scope.set_variable(
        "symbol?".to_string(),
        Expr::BuiltinFunction(BuiltinFunction::new(
            "symbol?",
            builtin_is_symbol,
            BuiltinKind::SpecialForm,
        )),
    );
}