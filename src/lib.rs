use interpreter::{eval, Scope};
use parser::{expr_to_string, parse};
use wasm_bindgen::prelude::*;
pub mod builtins;
pub mod interpreter;
pub mod parser;
pub mod repl;

#[wasm_bindgen]
pub struct Evaluator {
    scope: Scope,
}

#[wasm_bindgen]
impl Evaluator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Evaluator {
        let scope = Scope::new();
        Evaluator { scope }
    }

    pub fn eval(&mut self, input: &str) -> String {
        match parse(input) {
            Ok(expr) => match eval(&expr, &mut self.scope) {
                Ok(result) => expr_to_string(&result),
                Err(e) => format!("Error: {}", e),
            },
            Err(e) => format!("Error: {}", e),
        }
    }
}
