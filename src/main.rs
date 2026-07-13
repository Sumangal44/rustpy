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
use std::env;
use std::fs;
use std::io::{self, Write};
use std::rc::Rc;
use vm::VirtualMachine;
use vm::frame::Frame;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rustpy [script]");
        std::process::exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_repl();
    }
}

fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            std::process::exit(74);
        }
    };

    let env = Environment::new();
    stdlib::builtins::inject_builtins(&env);

    execute(&source, env, path);
}

fn run_repl() {
    println!("RustPy Interpreter 0.1.0 (Phase 8)");
    println!("Type 'quit()' or 'exit()' to exit.");

    let env = Environment::new();
    stdlib::builtins::inject_builtins(&env);

    let mut input = String::new();

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();

        input.clear();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed == "quit()" || trimmed == "exit()" {
                    break;
                }
                if trimmed.is_empty() {
                    continue;
                }

                // For REPL, we want the newline appended to help the parser
                execute(&input, Rc::clone(&env), "<stdin>");
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn execute(source: &str, env: Rc<RefCell<Environment>>, filename: &str) {
    let lexer = Lexer::new(source);
    match Parser::new(lexer) {
        Ok(mut parser) => {
            match parser.parse_module() {
                Ok(module) => {
                    let compiler = Compiler::new(filename.to_string());
                    match compiler.compile(&module) {
                        Ok(code) => {
                            let mut frame = Frame::new(code, env);
                            let mut vm = VirtualMachine::new();

                            match vm.run(&mut frame) {
                                Ok(_) => {
                                    // Execution succeeded
                                }
                                Err(e) => eprintln!("RuntimeError: {}", e),
                            }
                        }
                        Err(e) => eprintln!("CompilerError: {}", e),
                    }
                }
                Err(e) => eprintln!(
                    "{}",
                    diagnostics::format_error(source, &e.span, &e.to_string())
                ),
            }
        }
        Err(e) => eprintln!(
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
        let env = Environment::new();
        stdlib::builtins::inject_builtins(&env);
        execute(source, Rc::clone(&env), "<test>");
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
        assert_eq!(s.repr(), "'123'");
    }

    #[test]
    fn test_list_creation_and_subscript() {
        let source = "l = [10, 20, 30]\nx = l[1]\n";
        let env = execute_source(source);
        let l = env.borrow().get("l").unwrap();
        assert_eq!(l.repr(), "[10, 20, 30]");
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "20");
    }

    #[test]
    fn test_dict_creation_and_subscript() {
        let source = "d = {\"a\": 100, \"b\": 200}\nx = d[\"a\"]\n";
        let env = execute_source(source);
        let d = env.borrow().get("d").unwrap();
        // Hash map ordering is non-deterministic, so just check it's truthy
        assert!(d.is_truthy());
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "100");
    }

    #[test]
    fn test_for_loop() {
        let source = "items = [1, 2, 3]\nsum = 0\nfor x in items:\n    sum = sum + x\n";
        let env = execute_source(source);
        let sum = env.borrow().get("sum").unwrap();
        assert_eq!(sum.repr(), "6");
    }

    #[test]
    fn test_class_instantiation() {
        let source = "class Box:\n    def __init__(self, v):\n        self.val = v\nb = Box(42)\nresult = b.val\n";
        let env = execute_source(source);
        let result = env.borrow().get("result").unwrap();
        assert_eq!(result.repr(), "42");
    }

    #[test]
    fn test_exceptions() {
        let source = "handled = 0\nexc = None\ntry:\n    raise ValueError(\"Error Message\")\n    handled = 99\nexcept:\n    handled = 1\n";
        let env = execute_source(source);
        let handled = env.borrow().get("handled").unwrap();
        assert_eq!(handled.repr(), "1");
    }

    #[test]
    fn test_generators() {
        let source = "def gen():\n    yield 1\n    yield 2\n    yield 3\n\ntotal = 0\nfor x in gen():\n    total = total + x\n";
        let env = execute_source(source);
        let total = env.borrow().get("total").unwrap();
        assert_eq!(total.repr(), "6");
    }

    #[test]
    fn test_advanced_functions() {
        let source = "def foo(a, b, *args, **kwargs):\n    return (a + b) + len(args) + len(kwargs)\nresult = foo(1, 2, 3, 4, c=5, d=6)\n";
        let env = execute_source(source);
        let result = env.borrow().get("result").unwrap();
        assert_eq!(result.repr(), "7");
    }

    #[test]
    fn test_inheritance() {
        let source = "class Base:\n    def greet(self):\n        return \"Hello from Base\"\n\nclass Derived(Base):\n    def greet2(self):\n        return \"Hello from Derived\"\n\nd = Derived()\nres1 = d.greet()\nres2 = d.greet2()\n";
        let env = execute_source(source);
        let res1 = env.borrow().get("res1").unwrap();
        assert_eq!(res1.repr(), "'Hello from Base'");
        let res2 = env.borrow().get("res2").unwrap();
        assert_eq!(res2.repr(), "'Hello from Derived'");
    }

    #[test]
    fn test_super_call() {
        let source = "class Base:\n    def greet(self):\n        return \"Base\"\n\nclass Derived(Base):\n    def greet(self):\n        return super(Derived, self).greet() + \" and Derived\"\n\nd = Derived()\nresult = d.greet()\n";
        let env = execute_source(source);
        let result = env.borrow().get("result").unwrap();
        assert_eq!(result.repr(), "'Base and Derived'");
    }
}
