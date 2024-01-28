use crate::interpreter::Expr;

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_string = false;
    let mut escaped = false;

    for c in input.chars() {
        if escaped {
            current_token.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            in_string = !in_string;
        } else if c.is_whitespace() && !in_string {
            if !current_token.is_empty() {
                tokens.push(current_token);
                current_token = String::new();
            }
        } else if c == '(' || c == ')' {
            if !current_token.is_empty() {
                tokens.push(current_token);
                current_token = String::new();
            }
            tokens.push(c.to_string());
        } else {
            current_token.push(c);
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    tokens
}

fn parse_expr(tokens: &mut Vec<String>) -> Result<Expr, String> {
    if tokens.is_empty() {
        return Err("Unexpected end of input".to_string());
    }

    let token = tokens.remove(0);
    match token.as_str() {
        "(" => {
            let mut list = Vec::new();
            while !tokens.is_empty() && tokens[0] != ")" {
                list.push(parse_expr(tokens)?);
            }
            if tokens.is_empty() {
                return Err("Unexpected end of input".to_string());
            }
            tokens.remove(0); // Remove closing paren
            Ok(Expr::List(list))
        }
        ")" => Err("Unexpected ')'".to_string()),
        _ => {
            if let Ok(number) = token.parse::<f64>() {
                Ok(Expr::Number(number))
            } else {
                Ok(Expr::Symbol(token))
            }
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, String> {
    let mut tokens = tokenize(input);
    let expr = parse_expr(&mut tokens)?;
    if !tokens.is_empty() {
        return Err("Unexpected tokens at end of input".to_string());
    }
    Ok(expr)
}

pub fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Symbol(s) => s.clone(),
        Expr::Number(n) => n.to_string(),
        Expr::List(list) => {
            let items: Vec<String> = list.iter().map(expr_to_string).collect();
            format!("({})", items.join(" "))
        }
        Expr::Lambda(params, body) => {
            let params_str = params.join(" ");
            let body_str = expr_to_string(body);
            format!("(fn ({}) {})", params_str, body_str)
        }
        Expr::Function(_) => "<function>".to_string(),
        Expr::BuiltinFunction(_) => "<builtin-function>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number() {
        assert_eq!(parse("42"), Ok(Expr::number(42.0)));
    }

    #[test]
    fn parse_symbol() {
        assert_eq!(parse("x"), Ok(Expr::symbol("x")));
    }

    #[test]
    fn parse_list() {
        assert_eq!(
            parse("(+ 1 2)"),
            Ok(Expr::list(vec![
                Expr::symbol("+"),
                Expr::number(1.0),
                Expr::number(2.0)
            ]))
        );
    }
}
