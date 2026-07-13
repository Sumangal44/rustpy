mod ast;
mod compiler;
mod diagnostics;
mod lexer;
mod objects;
mod parser;
mod runtime;
mod vm;

use compiler::Compiler;
use lexer::Lexer;
use parser::Parser;
use runtime::Environment;
use vm::VirtualMachine;
use vm::frame::Frame;

fn main() {
    println!("RustPy Interpreter - Phase 5");

    // Demonstrate End-to-End Execution
    let source = "a = 10 * 5\nb = a + 2\n";
    println!("Executing source:\n{}", source);

    let lexer = Lexer::new(source);
    match Parser::new(lexer) {
        Ok(mut parser) => {
            match parser.parse_module() {
                Ok(module) => {
                    let compiler = Compiler::new();
                    match compiler.compile(&module) {
                        Ok(code) => {
                            let mut env = Environment::new();
                            let mut frame = Frame::new(code, env);
                            let mut vm = VirtualMachine::new();

                            match vm.run(&mut frame) {
                                Ok(_) => {
                                    println!("Execution successful!");
                                    // Verify state
                                    if let Some(a) = frame.env.get("a") {
                                        println!("a = {}", a.repr());
                                    }
                                    if let Some(b) = frame.env.get("b") {
                                        println!("b = {}", b.repr());
                                    }
                                }
                                Err(e) => println!("Runtime error: {}", e),
                            }
                        }
                        Err(e) => println!("Compiler error: {}", e),
                    }
                }
                Err(e) => println!(
                    "{}",
                    diagnostics::format_error(source, &e.span, &e.to_string())
                ),
            }
        }
        Err(e) => println!(
            "{}",
            diagnostics::format_error(source, &e.span, &e.to_string())
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::PyObject;
    use crate::objects::int::PyInt;
    use std::rc::Rc;

    fn execute_source(source: &str) -> Environment {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let module = parser.parse_module().unwrap();
        let compiler = Compiler::new();
        let code = compiler.compile(&module).unwrap();
        let env = Environment::new();
        let mut frame = Frame::new(code, env);
        let mut vm = VirtualMachine::new();
        vm.run(&mut frame).unwrap();
        frame.env
    }

    #[test]
    fn test_end_to_end_math() {
        let env = execute_source("a = 10 * 5 + 2\n");
        let a = env.get("a").unwrap();
        assert_eq!(a.repr(), "52");
    }

    #[test]
    fn test_end_to_end_variables() {
        let env = execute_source("a = 10\nb = a + 5\n");
        let b = env.get("b").unwrap();
        assert_eq!(b.repr(), "15");
    }

    #[test]
    fn test_end_to_end_if_statement() {
        let env = execute_source("a = 0\nif 1:\n    a = 42\n");
        let a = env.get("a").unwrap();
        assert_eq!(a.repr(), "42");
    }
}
