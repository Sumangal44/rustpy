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
use std::cell::RefCell;
use std::rc::Rc;
use vm::VirtualMachine;
use vm::frame::Frame;

fn main() {
    println!("RustPy Interpreter - Phase 6");

    let source = "def add(a, b):\n    return a + b\n\nc = add(10, 20)\n";
    println!("Executing source:\n{}", source);

    let lexer = Lexer::new(source);
    match Parser::new(lexer) {
        Ok(mut parser) => match parser.parse_module() {
            Ok(module) => {
                let compiler = Compiler::new("<module>".to_string());
                match compiler.compile(&module) {
                    Ok(code) => {
                        let env = Environment::new();
                        let mut frame = Frame::new(code, Rc::clone(&env));
                        let mut vm = VirtualMachine::new();

                        match vm.run(&mut frame) {
                            Ok(_) => {
                                println!("Execution successful!");
                                if let Some(c) = env.borrow().get("c") {
                                    println!("c = {}", c.repr());
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
        },
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

    fn execute_source(source: &str) -> Rc<RefCell<Environment>> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let module = parser.parse_module().unwrap();
        let compiler = Compiler::new("<test_module>".to_string());
        let code = compiler.compile(&module).unwrap();
        let env = Environment::new();
        let mut frame = Frame::new(code, Rc::clone(&env));
        let mut vm = VirtualMachine::new();
        vm.run(&mut frame).unwrap();
        env
    }

    #[test]
    fn test_function_call() {
        let source = "def add(x, y):\n    return x + y\n\nresult = add(100, 50)\n";
        let env = execute_source(source);
        let result = env.borrow().get("result").unwrap();
        assert_eq!(result.repr(), "150");
    }

    #[test]
    fn test_string_concat() {
        let source = "msg = \"hello \" + \"world\"\n";
        let env = execute_source(source);
        let msg = env.borrow().get("msg").unwrap();
        assert_eq!(msg.repr(), "'hello world'");
    }

    #[test]
    fn test_booleans() {
        let source = "t = True\nf = False\n";
        let env = execute_source(source);

        let t = env.borrow().get("t").unwrap();
        assert_eq!(t.repr(), "True");
        assert!(t.is_truthy());

        let f = env.borrow().get("f").unwrap();
        assert_eq!(f.repr(), "False");
        assert!(!f.is_truthy());
    }
}
