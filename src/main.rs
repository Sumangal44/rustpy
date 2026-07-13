mod ast;
mod compiler;
mod diagnostics;
mod lexer;
mod objects;
mod parser;
mod runtime;
mod stdlib;
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
    println!("RustPy Interpreter - Phase 7");

    let source =
        "print(\"Hello from RustPy built-ins!\")\nx = len(\"12345\")\nprint(\"Length is:\", x)\n";
    println!("Executing source:\n{}", source);

    let lexer = Lexer::new(source);
    match Parser::new(lexer) {
        Ok(mut parser) => {
            match parser.parse_module() {
                Ok(module) => {
                    let compiler = Compiler::new("<module>".to_string());
                    match compiler.compile(&module) {
                        Ok(code) => {
                            let env = Environment::new();
                            stdlib::builtins::inject_builtins(&env); // Inject Built-ins!

                            let mut frame = Frame::new(code, Rc::clone(&env));
                            let mut vm = VirtualMachine::new();

                            match vm.run(&mut frame) {
                                Ok(_) => {
                                    println!("Execution successful!");
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

    fn execute_source(source: &str) -> Rc<RefCell<Environment>> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let module = parser.parse_module().unwrap();
        let compiler = Compiler::new("<test_module>".to_string());
        let code = compiler.compile(&module).unwrap();
        let env = Environment::new();
        stdlib::builtins::inject_builtins(&env);
        let mut frame = Frame::new(code, Rc::clone(&env));
        let mut vm = VirtualMachine::new();
        vm.run(&mut frame).unwrap();
        env
    }

    #[test]
    fn test_builtin_len() {
        let source = "x = len(\"rustpy\")\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "6");
    }

    #[test]
    fn test_builtin_type() {
        let source = "t1 = type(\"hello\")\nt2 = type(42)\nt3 = type(True)\n";
        let env = execute_source(source);

        let t1 = env.borrow().get("t1").unwrap();
        assert_eq!(t1.repr(), "'<class 'str'>'");

        let t2 = env.borrow().get("t2").unwrap();
        assert_eq!(t2.repr(), "'<class 'int'>'");

        let t3 = env.borrow().get("t3").unwrap();
        assert_eq!(t3.repr(), "'<class 'bool'>'");
    }

    #[test]
    fn test_builtin_str() {
        let source = "s = str(123)\n";
        let env = execute_source(source);
        let s = env.borrow().get("s").unwrap();
        assert_eq!(s.repr(), "'123'"); // Evaluates to string representation
    }
}
