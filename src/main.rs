mod ast;
mod diagnostics;
mod lexer;
mod objects;
mod parser;
mod runtime;

use lexer::Lexer;
use objects::{PyObject, int::PyInt};
use parser::Parser;
use runtime::Environment;
use std::rc::Rc;

fn main() {
    println!("RustPy Interpreter - Phase 3");

    // Demonstrate beautiful errors
    let source = "def add(a, b\n    return a + b\n";
    let lexer = Lexer::new(source);
    match Parser::new(lexer) {
        Ok(mut parser) => {
            if let Err(e) = parser.parse_module() {
                println!(
                    "{}",
                    diagnostics::format_error(source, &e.span, &e.to_string())
                );
            }
        }
        Err(e) => {
            println!(
                "{}",
                diagnostics::format_error(source, &e.span, &e.to_string())
            );
        }
    }

    // Demonstrate Object Model
    let a: Rc<dyn PyObject> = Rc::new(PyInt::new(10));
    let b: Rc<dyn PyObject> = Rc::new(PyInt::new(20));

    if let Some(result) = a.add(b) {
        println!("10 + 20 = {}", result.repr());
    }

    let mut env = Environment::new();
    env.set("x".to_string(), Rc::new(PyInt::new(42)));
    if let Some(val) = env.get("x") {
        println!("x = {}", val.repr());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::PyObject;
    use crate::objects::int::PyInt;
    use crate::runtime::Environment;
    use std::rc::Rc;

    #[test]
    fn test_pyint_operations() {
        let a: Rc<dyn PyObject> = Rc::new(PyInt::new(10));
        let b: Rc<dyn PyObject> = Rc::new(PyInt::new(5));

        // Test add
        let sum = a.add(Rc::clone(&b)).unwrap();
        assert_eq!(sum.repr(), "15");

        // Test sub
        let diff = a.sub(Rc::clone(&b)).unwrap();
        assert_eq!(diff.repr(), "5");

        // Test mul
        let prod = a.mul(Rc::clone(&b)).unwrap();
        assert_eq!(prod.repr(), "50");
    }

    #[test]
    fn test_environment() {
        let mut env = Environment::new();
        env.set("x".to_string(), Rc::new(PyInt::new(42)));

        let val = env.get("x").unwrap();
        assert_eq!(val.repr(), "42");

        assert!(env.get("y").is_none());
    }
}
