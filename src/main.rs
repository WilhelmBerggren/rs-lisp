use crate::repl::repl;
pub mod builtins;
pub mod interpreter;
pub mod parser;
pub mod repl;

fn main() {
    repl();
}
