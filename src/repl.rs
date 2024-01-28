use crate::{
    builtins::initialize_global_scope,
    interpreter::{eval, Scope},
    parser::{expr_to_string, parse},
};
use std::io::{self, Write}; // Import Write for the flush method

pub fn repl() {
    let mut global_scope = Scope::new();
    initialize_global_scope(&mut global_scope);

    let mut input = String::new();
    loop {
        input.clear();
        print!("> ");
        io::stdout().flush().unwrap(); // Flush stdout to ensure the prompt is displayed

        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "exit" {
            break;
        }
        match parse(input) {
            Ok(expr) => match eval(&expr, &mut global_scope) {
                Ok(result) => println!("{}", expr_to_string(&result)),
                Err(e) => println!("Error: {}", e),
            },
            Err(e) => println!("Error: {}", e),
        }
    }
}
