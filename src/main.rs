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

    #[test]
    fn test_function_decorator() {
        let source = "def make_pretty(func):\n    def inner():\n        return \"***\" + func() + \"***\"\n    return inner\n\n@make_pretty\ndef ordinary():\n    return \"Hello\"\n\nres = ordinary()\n";
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "'***Hello***'");
    }

    #[test]
    fn test_class_decorator() {
        let source = "def class_dec(cls):\n    cls.added = 42\n    return cls\n\n@class_dec\nclass Foo:\n    pass\n\nres = Foo.added\n";
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "42");
    }

    #[test]
    fn test_multiple_decorators() {
        let source = "def a(func):\n    def inner():\n        return \"A\" + func()\n    return inner\n\ndef b(func):\n    def inner():\n        return \"B\" + func()\n    return inner\n\n@a\n@b\ndef ordinary():\n    return \"Hello\"\n\nres = ordinary()\n";
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "'ABHello'");
    }

    #[test]
    fn test_with_statement() {
        let source = "class ContextManager:\n    def __enter__(self):\n        return 42\n    def __exit__(self, exc_type, exc_value, traceback):\n        pass\n\nwith ContextManager() as x:\n    res = x\n";
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "42");
    }

    #[test]
    fn test_with_statement_suppress_exception() {
        let source = "class ContextManager:\n    def __enter__(self):\n        return 42\n    def __exit__(self, exc_type, exc_value, traceback):\n        return True\n\nres = 0\nwith ContextManager():\n    res = 1\n    raise Exception(\"test error\")\n    res = 2\n";
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "1");
    }

    #[test]
    fn test_float_literal() {
        let source = "x = 3.14\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.get_type(), "float");
    }

    #[test]
    fn test_true_division() {
        let source = "x = 7 / 2\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "3.5");
    }

    #[test]
    fn test_floor_division() {
        let source = "x = 7 // 2\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "3");
    }

    #[test]
    fn test_modulo() {
        let source = "x = 10 % 3\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "1");
    }

    #[test]
    fn test_power() {
        let source = "x = 2 ** 10\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "1024");
    }

    #[test]
    fn test_unary_minus() {
        let source = "x = -5\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "-5");
    }

    #[test]
    fn test_unary_plus() {
        let source = "x = +5\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "5");
    }

    #[test]
    fn test_unary_not() {
        let source = "x = not True\ny = not False\nz = not 0\nw = not 42\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "False");
        let y = env.borrow().get("y").unwrap();
        assert_eq!(y.repr(), "True");
        let z = env.borrow().get("z").unwrap();
        assert_eq!(z.repr(), "True");
        let w = env.borrow().get("w").unwrap();
        assert_eq!(w.repr(), "False");
    }

    #[test]
    fn test_compare_eq() {
        let source = "x = 5 == 5\ny = 5 == 6\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "True");
        let y = env.borrow().get("y").unwrap();
        assert_eq!(y.repr(), "False");
    }

    #[test]
    fn test_compare_not_eq() {
        let source = "x = 5 != 6\ny = 5 != 5\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "True");
        let y = env.borrow().get("y").unwrap();
        assert_eq!(y.repr(), "False");
    }

    #[test]
    fn test_compare_lt_gt() {
        let source = "x = 3 < 5\ny = 5 < 3\nz = 5 > 3\nw = 3 > 5\n";
        let env = execute_source(source);
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "True");
        let y = env.borrow().get("y").unwrap();
        assert_eq!(y.repr(), "False");
        let z = env.borrow().get("z").unwrap();
        assert_eq!(z.repr(), "True");
        let w = env.borrow().get("w").unwrap();
        assert_eq!(w.repr(), "False");
    }

    #[test]
    fn test_compare_le_ge() {
        let source = "x = 3 <= 5\ny = 5 <= 3\nz = 5 <= 5\nw = 5 >= 3\nu = 3 >= 5\nv = 5 >= 5\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "False");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("w").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("u").unwrap().repr(), "False");
        assert_eq!(env.borrow().get("v").unwrap().repr(), "True");
    }

    #[test]
    fn test_float_arithmetic() {
        let source = "x = 2.5 + 3.5\ny = 10.0 - 4.5\nz = 3.0 * 1.5\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "6.0");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "5.5");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "4.5");
    }

    #[test]
    fn test_mixed_int_float() {
        let source = "x = 1 + 2.5\ny = 5.5 - 2\nz = 3 * 1.5\nw = 7 / 2\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "3.5");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "3.5");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "4.5");
        assert_eq!(env.borrow().get("w").unwrap().repr(), "3.5");
    }

    #[test]
    fn test_string_compare() {
        let source = "x = \"abc\" == \"abc\"\ny = \"abc\" != \"xyz\"\nz = \"abc\" < \"xyz\"\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "True");
    }

    #[test]
    fn test_bool_compare() {
        let source = "x = True == True\ny = True != False\nz = True == 1\nw = True == 1.0\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("w").unwrap().repr(), "True");
    }
}
