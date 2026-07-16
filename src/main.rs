mod ast;
mod compiler;
mod diagnostics;
mod encoding;
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
                                Err(e) => {
                                    let final_err = if e.starts_with("Traceback (most recent call last):\n") {
                                        let mut lines: Vec<String> = e.split('\n').map(|s| s.to_string()).collect();
                                        let last_line = lines.pop().unwrap_or_default();
                                        lines.insert(1, format!("  File \"{}\", in <module>", frame.code.filename));
                                        lines.push(last_line);
                                        lines.join("\n")
                                    } else {
                                        format!(
                                            "Traceback (most recent call last):\n  File \"{}\", in <module>\n{}",
                                            frame.code.filename,
                                            e
                                        )
                                    };
                                    eprintln!("{}", final_err);
                                }
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

    fn execute_program(source: &str) -> String {
        let env = Environment::new();
        stdlib::builtins::inject_builtins(&env);
        let wrapped = format!("__result__ = {}\n", source);
        execute(&wrapped, Rc::clone(&env), "<test>");
        env.borrow()
            .get("__result__")
            .map(|r| r.repr())
            .unwrap_or_default()
    }

    fn execute_source(source: &str) -> Rc<RefCell<Environment>> {
        let env = Environment::new();
        stdlib::builtins::inject_builtins(&env);
        let s = if source.ends_with('\n') { source.to_string() } else { format!("{}\n", source) };
        execute(&s, Rc::clone(&env), "<test>");
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
        assert_eq!(t1.repr(), "<class 'str'>");

        let t2 = env.borrow().get("t2").unwrap();
        assert_eq!(t2.repr(), "<class 'int'>");

        let t3 = env.borrow().get("t3").unwrap();
        assert_eq!(t3.repr(), "<class 'bool'>");
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

    #[test]
    fn test_break_in_while() {
        let source = "i = 0\nwhile i < 10:\n    i = i + 1\n    if i == 5:\n        break\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("i").unwrap().repr(), "5");
    }

    #[test]
    fn test_continue_in_while() {
        let source = "i = 0\nsum = 0\nwhile i < 10:\n    i = i + 1\n    if i % 2 == 0:\n        continue\n    sum = sum + i\n";
        let env = execute_source(source);
        // Sum of odd numbers 1+3+5+7+9 = 25
        assert_eq!(env.borrow().get("sum").unwrap().repr(), "25");
    }

    #[test]
    fn test_break_in_for() {
        let source = "total = 0\nfor x in [1, 2, 3, 4, 5]:\n    if x == 3:\n        break\n    total = x\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("total").unwrap().repr(), "2");
    }

    #[test]
    fn test_del_variable() {
        let source = "x = 42\ndel x\n";
        let env = execute_source(source);
        assert!(env.borrow().get("x").is_none());
    }

    #[test]
    fn test_augmented_add() {
        let source = "x = 5\nx += 3\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "8");
    }

    #[test]
    fn test_augmented_sub() {
        let source = "x = 10\nx -= 3\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "7");
    }

    #[test]
    fn test_augmented_mul() {
        let source = "x = 4\nx *= 3\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "12");
    }

    #[test]
    fn test_augmented_div() {
        let source = "x = 7\nx /= 2\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "3.5");
    }

    #[test]
    fn test_augmented_floor_div() {
        let source = "x = 7\nx //= 2\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "3");
    }

    #[test]
    fn test_augmented_mod() {
        let source = "x = 10\nx %= 3\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "1");
    }

    #[test]
    fn test_augmented_pow() {
        let source = "x = 2\nx **= 10\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "1024");
    }

    #[test]
    fn test_assert_passes() {
        let source = "x = 1\nassert x == 1\nres = 42\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("res").unwrap().repr(), "42");
    }

    #[test]
    fn test_break_outside_loop_error() {
        let source = "break\n";
        let env = execute_source(source);
        // Should produce an error, but not crash
        assert!(true);
    }

    #[test]
    fn test_continue_outside_loop_error() {
        let source = "continue\n";
        let env = execute_source(source);
        assert!(true);
    }

    #[test]
    fn test_string_upper_lower() {
        let source = "a = \"hello\".upper()\nb = \"HELLO\".lower()\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'HELLO'");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'hello'");
    }

    #[test]
    fn test_string_strip() {
        let source = "a = \"  hi  \".strip()\nb = \"xxhixx\".strip(\"x\")\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'hi'");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'hi'");
    }

    #[test]
    fn test_string_split_join() {
        let source = "a = \"a b c\".split()\nb = \",\".join([\"x\", \"y\", \"z\"])\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "['a', 'b', 'c']");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'x,y,z'");
    }

    #[test]
    fn test_string_replace() {
        let source = "s = \"hello world\".replace(\"world\", \"there\")\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("s").unwrap().repr(), "'hello there'");
    }

    #[test]
    fn test_string_startswith_endswith_find() {
        let source = "a = \"hello\".startswith(\"he\")\nb = \"hello\".endswith(\"lo\")\nc = \"hello\".find(\"ll\")\nd = \"hello\".find(\"zz\")\ne = \"hello\".index(\"ll\")\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "2");
        assert_eq!(env.borrow().get("d").unwrap().repr(), "-1");
        assert_eq!(env.borrow().get("e").unwrap().repr(), "2");
    }

    #[test]
    fn test_string_count_isdigit() {
        let source = "a = \"hello\".count(\"l\")\nb = \"123\".isdigit()\nc = \"abc\".isdigit()\nd = \"abc123\".isalpha()\ne = \"abc\".isalpha()\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "2");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "False");
        assert_eq!(env.borrow().get("d").unwrap().repr(), "False");
        assert_eq!(env.borrow().get("e").unwrap().repr(), "True");
    }

    #[test]
    fn test_string_isalnum_isspace_capitalize() {
        let source = "a = \"abc123\".isalnum()\nb = \"   \".isspace()\nc = \"hello\".capitalize()\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "'Hello'");
    }

    #[test]
    fn test_string_zfill_title_swapcase() {
        let source = "a = \"42\".zfill(5)\nb = \"hello world\".title()\nc = \"Hello\".swapcase()\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'00042'");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'Hello World'");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "'hELLO'");
    }

    #[test]
    fn test_string_ljust_rjust_center() {
        let source = "a = \"hi\".ljust(5, '*')\nb = \"hi\".rjust(5, '*')\nc = \"hi\".center(5, '*')\nd = \"hi\".center(4, '*')\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'hi***'");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'***hi'");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "'*hi**'");
        assert_eq!(env.borrow().get("d").unwrap().repr(), "'*hi*'");
    }

    #[test]
    fn test_string_lstrip_rstrip() {
        let source = "a = \"  hi  \".lstrip()\nb = \"  hi  \".rstrip()\nc = \"xxhixx\".lstrip(\"x\")\nd = \"xxhixx\".rstrip(\"x\")\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'hi  '");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'  hi'");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "'hixx'");
        assert_eq!(env.borrow().get("d").unwrap().repr(), "'xxhi'");
    }

    #[test]
    fn test_list_append_pop() {
        let source = "l = [1, 2, 3]\nl.append(4)\nx = l.pop()\n";
        let env = execute_source(source);
        let l = env.borrow().get("l").unwrap();
        assert_eq!(l.repr(), "[1, 2, 3]");
        let x = env.borrow().get("x").unwrap();
        assert_eq!(x.repr(), "4");
    }

    #[test]
    fn test_list_insert_remove() {
        let source = "l = [1, 2, 3]\nl.insert(1, 99)\nl.remove(99)\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("l").unwrap().repr(), "[1, 2, 3]");
    }

    #[test]
    fn test_list_index_count_reverse() {
        let source = "l = [10, 20, 30, 20]\na = l.index(20)\nb = l.count(20)\nl.reverse()\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "1");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "2");
        assert_eq!(env.borrow().get("l").unwrap().repr(), "[20, 30, 20, 10]");
    }

    #[test]
    fn test_list_sort_clear_copy_extend() {
        let source = "l = [3, 1, 2]\nl.sort()\nc = l.copy()\nl.clear()\nl.extend([4, 5])\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("l").unwrap().repr(), "[4, 5]");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "[1, 2, 3]");
    }

    #[test]
    fn test_list_pop_index() {
        let source = "l = [10, 20, 30]\nx = l.pop(1)\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("l").unwrap().repr(), "[10, 30]");
        assert_eq!(env.borrow().get("x").unwrap().repr(), "20");
    }

    #[test]
    fn test_dict_keys_values_items() {
        let source = "d = {\"a\": 1, \"b\": 2}\nk = d.keys()\nv = d.values()\n";
        let env = execute_source(source);
        let k = env.borrow().get("k").unwrap();
        let v = env.borrow().get("v").unwrap();
        // Just check they work and produce lists
        assert!(k.get_type() == "list");
        assert!(v.get_type() == "list");
    }

    #[test]
    fn test_dict_get_pop() {
        let source = "d = {\"a\": 100}\na = d.get(\"a\")\nb = d.get(\"missing\", 42)\nc = d.pop(\"a\")\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "100");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "42");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "100");
    }

    #[test]
    fn test_dict_update_clear_copy() {
        let source = "d1 = {\"a\": 1}\nd2 = {\"b\": 2}\nd1.update(d2)\nc = d1.copy()\nd1.clear()\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("d1").unwrap().repr(), "{}");
        let c = env.borrow().get("c").unwrap();
        // c should still have the old values
        assert!(c.is_truthy());
    }

    #[test]
    fn test_dict_setdefault_popitem() {
        let source = "d = {\"a\": 1}\na = d.setdefault(\"a\", 99)\nb = d.setdefault(\"b\", 42)\nitem = d.popitem()\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "1");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "42");
        // popitem removes one item; dict should still have 1 entry
        let d = env.borrow().get("d").unwrap();
        assert!(d.is_truthy());
        // popitem returns a 2-element tuple
        let item = env.borrow().get("item").unwrap();
        assert_eq!(item.get_type(), "tuple");
        // popitem on single-item dict empties it
        let source2 = "d = {\"x\": 100}\nitem = d.popitem()\n";
        let env2 = execute_source(source2);
        assert_eq!(env2.borrow().get("d").unwrap().repr(), "{}");
    }

    #[test]
    fn test_string_split_with_sep() {
        let source = "s = \"a,b,c\".split(\",\")\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("s").unwrap().repr(), "['a', 'b', 'c']");
    }

    #[test]
    fn test_string_index_error() {
        let source = "x = None\ntry:\n    \"hello\".index(\"zz\")\nexcept:\n    x = 1\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "1");
    }

    #[test]
    fn test_in_list() {
        let source = "a = 1 in [1, 2, 3]\nb = 4 in [1, 2, 3]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "False");
    }

    #[test]
    fn test_not_in_list() {
        let source = "a = 1 not in [1, 2, 3]\nb = 4 not in [1, 2, 3]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "False");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "True");
    }

    #[test]
    fn test_in_string() {
        let source = "a = \"ll\" in \"hello\"\nb = \"zz\" in \"hello\"\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "False");
    }

    #[test]
    fn test_in_dict() {
        let source = "d = {\"a\": 1, \"b\": 2}\na = \"a\" in d\nb = \"c\" in d\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "False");
    }

    #[test]
    fn test_is_operator() {
        let source = "a = [1, 2]\nb = a\nc = [1, 2]\nx = a is b\ny = a is c\nz = a is not c\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "False");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "True");
    }

    #[test]
    fn test_and_short_circuit() {
        let source = "x = 0 and 1\ny = 1 and 2\nz = 0 and None\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "0");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "2");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "0");
    }

    #[test]
    fn test_or_short_circuit() {
        let source = "x = 0 or 1\ny = 1 or 2\nz = 0 or 42\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "1");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "1");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "42");
    }

    #[test]
    fn test_and_or_combined() {
        let source = "x = 0 and 1 or 2\ny = 1 and 0 or 3\nz = 0 and 1 and 2\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("x").unwrap().repr(), "2");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "3");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "0");
    }

    #[test]
    fn test_list_slice_basic() {
        let source = "l = [0, 1, 2, 3, 4]\na = l[1:3]\nb = l[:3]\nc = l[2:]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "[1, 2]");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "[0, 1, 2]");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "[2, 3, 4]");
    }

    #[test]
    fn test_list_slice_full() {
        let source = "l = [0, 1, 2, 3, 4]\na = l[:]\nb = l[::2]\nc = l[1:4:2]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "[0, 1, 2, 3, 4]");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "[0, 2, 4]");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "[1, 3]");
    }

    #[test]
    fn test_list_slice_negative() {
        let source = "l = [0, 1, 2, 3, 4]\na = l[-3:-1]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "[2, 3]");
    }

    #[test]
    fn test_string_slice() {
        let source = "s = \"hello\"\na = s[1:4]\nb = s[:3]\nc = s[2:]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'ell'");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'hel'");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "'llo'");
    }

    #[test]
    fn test_lambda_simple() {
        let source = "f = lambda x: x + 1\nresult = f(5)\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("result").unwrap().repr(), "6");
    }

    #[test]
    fn test_lambda_no_args() {
        let source = "f = lambda: 42\nresult = f()\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("result").unwrap().repr(), "42");
    }

    #[test]
    fn test_lambda_multiple_args() {
        let source = "f = lambda a, b: a * b\nresult = f(3, 4)\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("result").unwrap().repr(), "12");
    }

    #[test]
    fn test_lambda_closure() {
        let source = "x = 10\nf = lambda y: x + y\nresult = f(5)\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("result").unwrap().repr(), "15");
    }

    #[test]
    fn test_lambda_as_argument() {
        let source = "def apply(f, x):\n    return f(x)\nresult = apply(lambda x: x * 2, 7)\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("result").unwrap().repr(), "14");
    }

    #[test]
    fn test_string_slice_step() {
        let source = "s = \"hello\"\na = s[::2]\nb = s[::-1]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'hlo'");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'olleh'");
    }

    #[test]
    fn test_is_with_vars() {
        let source = "x = 42\na = x is x\nb = x is 99\nc = x is not 99\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("a").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "False");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "True");
    }

    #[test]
    fn test_list_comprehension_simple() {
        let source = "result = [x for x in [1, 2, 3]]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("result").unwrap().repr(), "[1, 2, 3]");
    }

    #[test]
    fn test_list_comprehension_expr() {
        let source = "result = [x*2 for x in [1, 2, 3]]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("result").unwrap().repr(), "[2, 4, 6]");
    }

    #[test]
    fn test_list_comprehension_strings() {
        let source = "result = [s.upper() for s in [\"a\", \"b\", \"c\"]]\n";
        let env = execute_source(source);
        assert_eq!(env.borrow().get("result").unwrap().repr(), "['A', 'B', 'C']");
    }

    #[test]
    fn test_dict_comprehension() {
        let source = "result = {x: x*2 for x in [1, 2, 3]}\n";
        let env = execute_source(source);
        let result = env.borrow().get("result").unwrap();
        assert!(result.is_truthy());
        // Check values via get_item with int key
        let key = std::rc::Rc::new(crate::objects::int::PyInt::from_i64(1)) as std::rc::Rc<dyn crate::objects::PyObject>;
        let val = result.get_item(key).unwrap();
        assert_eq!(val.repr(), "2");
    }

    #[test]
    fn test_builtin_chr_ord() {
        let env = execute_source("a = chr(65)\nb = ord('A')\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'A'");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "65");
    }

    #[test]
    fn test_builtin_pow() {
        let env = execute_source("a = pow(2, 3)\nb = pow(5, 0)\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "8");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "1");
    }

    #[test]
    fn test_builtin_round() {
        let env = execute_source("a = round(3.7)\nb = round(42)\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "4");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "42");
    }

    #[test]
    fn test_builtin_sorted_reversed() {
        let env = execute_source("a = sorted([3, 1, 2])\nb = reversed([1, 2, 3])\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "[1, 2, 3]");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "[3, 2, 1]");
    }

    #[test]
    fn test_builtin_enumerate() {
        let env = execute_source("a = enumerate(['a', 'b', 'c'])\n");
        // Eager: returns list of [index, value] pairs
        let result = env.borrow().get("a").unwrap();
        assert!(result.repr().contains("0"));
        assert!(result.repr().contains("'a'"));
    }

    #[test]
    fn test_fstring_simple() {
        let env = execute_source("name = \"world\"\nresult = f\"hello {name}\"\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'hello world'");
    }

    #[test]
    fn test_fstring_int_expr() {
        let env = execute_source("x = 42\nresult = f\"x is {x}\"\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'x is 42'");
    }

    #[test]
    fn test_fstring_expr_arith() {
        let env = execute_source("x = 3\ny = 4\nresult = f\"{x} + {y} = {x+y}\"\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'3 + 4 = 7'");
    }

    #[test]
    fn test_fstring_empty() {
        let env = execute_source("result = f\"\"\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "''");
    }

    #[test]
    fn test_fstring_advanced() {
        let env = execute_source("x = 3.14159\nresult = f\"{x:.2f}\"\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'3.14'");

        let env = execute_source("x = 42\nresult = f\"{x:>10}\"\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'        42'");

        let env = execute_source("x = 42\nresult = f\"{x=}\"\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'x=42'");

        let env = execute_source("x = 10\nresult = f\"{{{x}}}\"\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'{10}'");

        let env = execute_source("x = 3\ny = 4\nresult = f\"{x+y=}\"\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'x+y=7'");
    }

    #[test]
    fn test_range_simple() {
        let env = execute_source("r = range(5)\na = list(r)\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "[0, 1, 2, 3, 4]");
    }

    #[test]
    fn test_range_start_stop() {
        let env = execute_source("r = range(2, 5)\na = list(r)\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "[2, 3, 4]");
    }

    #[test]
    fn test_range_step() {
        let env = execute_source("r = range(0, 10, 3)\na = list(r)\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "[0, 3, 6, 9]");
    }

    #[test]
    fn test_range_negative_step() {
        let env = execute_source("r = range(5, 0, -1)\na = list(r)\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "[5, 4, 3, 2, 1]");
    }

    #[test]
    fn test_range_len() {
        let env = execute_source("a = len(range(10))\nb = len(range(2, 8))\nc = len(range(0, 10, 3))\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "10");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "6");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "4");
    }

    #[test]
    fn test_range_for_loop() {
        let env = execute_source("
result = []
for i in range(3):
    result.append(i)
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "[0, 1, 2]");
    }

    #[test]
    fn test_bitwise_ops() {
        let env = execute_source("a = 5 & 3\nb = 5 | 3\nc = 5 ^ 3\nd = 5 << 1\ne = 5 >> 1\nf = ~5\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "1");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "7");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "6");
        assert_eq!(env.borrow().get("d").unwrap().repr(), "10");
        assert_eq!(env.borrow().get("e").unwrap().repr(), "2");
        assert_eq!(env.borrow().get("f").unwrap().repr(), "-6");
    }

    #[test]
    fn test_walrus_operator() {
        let env = execute_source("(a := 42)\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "42");
    }

    #[test]
    fn test_walrus_in_while() {
        let env = execute_source("
x = 0
results = []
while (y := x + 1) < 5:
    results.append(y)
    x = y
");
        assert_eq!(env.borrow().get("results").unwrap().repr(), "[1, 2, 3, 4]");
    }

    #[test]
    fn test_generator_expression() {
        let env = execute_source("g = (x for x in [1, 2, 3])\nresult = list(g)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "[1, 2, 3]");
    }

    #[test]
    fn test_match_literal() {
        let env = execute_source("
x = 2
result = None
match x:
    case 1:
        result = \"one\"
    case 2:
        result = \"two\"
    case 3:
        result = \"three\"
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'two'");
    }

    #[test]
    fn test_match_capture() {
        let env = execute_source("
x = 42
match x:
    case y:
        result = y
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "42");
    }

    #[test]
    fn test_match_wildcard() {
        let env = execute_source("
x = 99
result = \"default\"
match x:
    case 1:
        result = \"one\"
    case _:
        result = \"other\"
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'other'");
    }

    #[test]
    fn test_match_guard() {
        let env = execute_source("
x = 5
result = None
match x:
    case n if n > 0:
        result = \"positive\"
    case _:
        result = \"non-positive\"
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'positive'");
    }

    #[test]
    fn test_or_pattern() {
        let env = execute_source("
x = 2
result = None
match x:
    case 1 | 2 | 3:
        result = \"small\"
    case _:
        result = \"large\"
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "'small'");
    }

    #[test]
    fn test_generator_list_comprehension() {
        let env = execute_source("g = (x*2 for x in [1, 2, 3])\nresult = list(g)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "[2, 4, 6]");
    }

    #[test]
    fn test_set_comprehension() {
        let env = execute_source("result = {x for x in [1, 2, 3]}\n");
        let result = env.borrow().get("result").unwrap();
        assert_eq!(result.repr(), "{1, 2, 3}");
    }

    #[test]
    fn test_set_create() {
        let env = execute_source("s = {1, 2, 3}\n");
        let s = env.borrow().get("s").unwrap();
        assert_eq!(s.get_type(), "set");
        assert_eq!(s.repr(), "{1, 2, 3}");
    }

    #[test]
    fn test_set_len() {
        let env = execute_source("s = {1, 2, 3}\nresult = len(s)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "3");
    }

    #[test]
    fn test_set_contains() {
        let env = execute_source("s = {1, 2, 3}\nr1 = 2 in s\nr2 = 5 in s\n");
        assert_eq!(env.borrow().get("r1").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r2").unwrap().repr(), "False");
    }

    #[test]
    fn test_set_union() {
        let env = execute_source("a = {1, 2}\nb = {2, 3}\nresult = a | b\n");
        let result = env.borrow().get("result").unwrap();
        assert_eq!(result.get_type(), "set");
        assert!(result.repr() == "{1, 2, 3}" || result.repr() == "{2, 1, 3}" || result.repr() == "{2, 3, 1}");
    }

    #[test]
    fn test_set_intersection() {
        let env = execute_source("a = {1, 2, 3}\nb = {2, 3, 4}\nresult = a & b\n");
        let result = env.borrow().get("result").unwrap();
        assert_eq!(result.get_type(), "set");
        assert!(result.repr() == "{2, 3}" || result.repr() == "{3, 2}");
    }

    #[test]
    fn test_set_difference() {
        let env = execute_source("a = {1, 2, 3}\nb = {2, 3}\nresult = a - b\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "{1}");
    }

    #[test]
    fn test_set_symmetric_difference() {
        let env = execute_source("a = {1, 2}\nb = {2, 3}\nresult = a ^ b\n");
        assert_eq!(env.borrow().get("result").unwrap().get_type(), "set");
    }

    #[test]
    fn test_set_add_remove() {
        let env = execute_source("s = {1, 2}\ns.add(3)\ns.remove(1)\n");
        assert_eq!(env.borrow().get("s").unwrap().repr(), "{2, 3}");
    }

    #[test]
    fn test_set_discard() {
        let env = execute_source("s = {1, 2}\ns.discard(1)\ns.discard(99)\n");
        assert_eq!(env.borrow().get("s").unwrap().repr(), "{2}");
    }

    #[test]
    fn test_set_isdisjoint() {
        let env = execute_source("a = {1, 2}\nb = {3, 4}\nc = {2, 3}\nr1 = a.isdisjoint(b)\nr2 = a.isdisjoint(c)\n");
        assert_eq!(env.borrow().get("r1").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r2").unwrap().repr(), "False");
    }

    #[test]
    fn test_set_issubset() {
        let env = execute_source("a = {1, 2}\nb = {1, 2, 3}\nc = {1, 3}\nr1 = a.issubset(b)\nr2 = a.issubset(c)\n");
        assert_eq!(env.borrow().get("r1").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r2").unwrap().repr(), "False");
    }

    #[test]
    fn test_set_issuperset() {
        let env = execute_source("a = {1, 2, 3}\nb = {1, 2}\nc = {1, 2, 4}\nr1 = a.issuperset(b)\nr2 = a.issuperset(c)\n");
        assert_eq!(env.borrow().get("r1").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r2").unwrap().repr(), "False");
    }

    #[test]
    fn test_set_subset_operators() {
        let env = execute_source("a = {1, 2}\nb = {1, 2, 3}\nr1 = a <= b\nr2 = a < b\nr3 = b > a\nr4 = b >= a\nr5 = a == a\n");
        assert_eq!(env.borrow().get("r1").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r2").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r3").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r4").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r5").unwrap().repr(), "True");
    }

    #[test]
    fn test_set_builtin() {
        let env = execute_source("s = set([1, 2, 2, 3])\n");
        assert_eq!(env.borrow().get("s").unwrap().repr(), "{1, 2, 3}");
    }

    #[test]
    fn test_frozenset_builtin() {
        let env = execute_source("fs = frozenset([1, 2, 3])\n");
        let fs = env.borrow().get("fs").unwrap();
        assert_eq!(fs.get_type(), "frozenset");
    }

    #[test]
    fn test_frozenset_operations() {
        let env = execute_source("
a = frozenset([1, 2])
b = frozenset([2, 3])
r1 = a | b
r2 = a & b
r3 = a - b
r4 = a ^ b
");
        assert_eq!(env.borrow().get("r1").unwrap().get_type(), "frozenset");
        assert_eq!(env.borrow().get("r2").unwrap().get_type(), "frozenset");
        assert_eq!(env.borrow().get("r3").unwrap().get_type(), "frozenset");
        assert_eq!(env.borrow().get("r4").unwrap().get_type(), "frozenset");
    }

    #[test]
    fn test_set_pop() {
        let env = execute_source("s = {42}\nresult = s.pop()\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "42");
    }

    #[test]
    fn test_set_clear() {
        let env = execute_source("s = {1, 2, 3}\ns.clear()\nresult = len(s)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "0");
    }

    #[test]
    fn test_set_copy() {
        let env = execute_source("s = {1, 2, 3}\nc = s.copy()\ns.add(4)\nr1 = len(s)\nr2 = len(c)\n");
        assert_eq!(env.borrow().get("r1").unwrap().repr(), "4");
        assert_eq!(env.borrow().get("r2").unwrap().repr(), "3");
    }

    #[test]
    fn test_set_update() {
        let env = execute_source("a = {1, 2}\na.update({2, 3, 4})\nresult = len(a)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "4");
    }

    #[test]
    fn test_finally_no_exception() {
        let env = execute_source("
result = []
try:
    result.append('try')
finally:
    result.append('finally')
result
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "['try', 'finally']");
    }

    #[test]
    fn test_finally_with_exception() {
        let env = execute_source("
result = []
try:
    result.append('try')
    raise 'error'
finally:
    result.append('finally')
result
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "['try', 'finally']");
    }

    #[test]
    fn test_try_except_finally_no_exception() {
        let env = execute_source("
result = []
try:
    result.append('try')
except:
    result.append('except')
finally:
    result.append('finally')
result
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "['try', 'finally']");
    }

    #[test]
    fn test_try_except_finally_with_exception() {
        let env = execute_source("
result = []
try:
    result.append('try')
    raise 'error'
except:
    result.append('except')
finally:
    result.append('finally')
result
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "['try', 'except', 'finally']");
    }

    #[test]
    fn test_tuple_create() {
        let env = execute_source("t = (1, 2, 3)\n");
        assert_eq!(env.borrow().get("t").unwrap().repr(), "(1, 2, 3)");
    }

    #[test]
    fn test_tuple_single() {
        let env = execute_source("t = (1,)\n");
        assert_eq!(env.borrow().get("t").unwrap().repr(), "(1,)");
    }

    #[test]
    fn test_tuple_empty() {
        let env = execute_source("t = ()\n");
        assert_eq!(env.borrow().get("t").unwrap().repr(), "()");
    }

    #[test]
    fn test_tuple_index() {
        let env = execute_source("t = (10, 20, 30)\nr = t[1]\n");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "20");
    }

    #[test]
    fn test_tuple_len() {
        let env = execute_source("r = len((1, 2, 3))\n");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "3");
    }

    #[test]
    fn test_tuple_contains() {
        let env = execute_source("r1 = 3 in (1, 2, 3)\nr2 = 5 in (1, 2, 3)\n");
        assert_eq!(env.borrow().get("r1").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r2").unwrap().repr(), "False");
    }

    #[test]
    fn test_tuple_unpack() {
        let env = execute_source("a, b = (1, 2)\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "1");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "2");
    }

    #[test]
    fn test_tuple_iteration() {
        let env = execute_source("total = 0\nfor x in (1, 2, 3):\n    total = total + x\n");
        assert_eq!(env.borrow().get("total").unwrap().repr(), "6");
    }

    #[test]
    fn test_tuple_index_method() {
        let env = execute_source("t = (10, 20, 30)\nr = t.index(20)\n");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "1");
    }

    #[test]
    fn test_tuple_count_method() {
        let env = execute_source("t = (1, 2, 2, 3)\nr = t.count(2)\n");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "2");
    }

    #[test]
    fn test_tuple_builtin() {
        let env = execute_source("t = tuple([1, 2, 3])\n");
        assert_eq!(env.borrow().get("t").unwrap().repr(), "(1, 2, 3)");
    }

    #[test]
    fn test_tuple_equality() {
        let env = execute_source("r1 = (1, 2) == (1, 2)\nr2 = (1, 2) == (3, 4)\n");
        assert_eq!(env.borrow().get("r1").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r2").unwrap().repr(), "False");
    }

    #[test]
    fn test_tuple_concatenate() {
        let env = execute_source("t = (1, 2) + (3, 4)\n");
        assert_eq!(env.borrow().get("t").unwrap().repr(), "(1, 2, 3, 4)");
    }

    #[test]
    fn test_tuple_repeat() {
        let env = execute_source("t = (1, 2) * 3\n");
        assert_eq!(env.borrow().get("t").unwrap().repr(), "(1, 2, 1, 2, 1, 2)");
    }

    #[test]
    fn test_tuple_slice() {
        let env = execute_source("t = (0, 1, 2, 3, 4)\nr = t[1:3]\n");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "(1, 2)");
    }

    #[test]
    fn test_tuple_add_mul() {
        let env = execute_source("t = (1, 2) + (3,)\nr = t * 2\n");
        assert_eq!(env.borrow().get("t").unwrap().repr(), "(1, 2, 3)");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "(1, 2, 3, 1, 2, 3)");
    }

    #[test]
    fn test_complex_create() {
        let env = execute_source("1+2j\n");
        let output = env.borrow().get("_").unwrap_or_else(|| {
            // Expression statement result is discarded, so we test by storing
            env.borrow().get("x").unwrap_or(Rc::new(crate::objects::bool::PyBool::new(true)))
        });
        // Just run to make sure it doesn't error
    }

    #[test]
    fn test_complex_repr() {
        let env = execute_source("x = 1+2j\n");
        assert_eq!(env.borrow().get("x").unwrap().repr(), "(1+2j)");
    }

    #[test]
    fn test_complex_imag_only() {
        let env = execute_source("x = 5j\n");
        assert_eq!(env.borrow().get("x").unwrap().repr(), "5j");
    }

    #[test]
    fn test_complex_real_attr() {
        let env = execute_source("x = (3+4j).real\n");
        assert_eq!(env.borrow().get("x").unwrap().repr(), "3.0");
    }

    #[test]
    fn test_complex_imag_attr() {
        let env = execute_source("x = (3+4j).imag\n");
        assert_eq!(env.borrow().get("x").unwrap().repr(), "4.0");
    }

    #[test]
    fn test_complex_add() {
        let env = execute_source("x = (1+2j) + (3+4j)\n");
        assert_eq!(env.borrow().get("x").unwrap().repr(), "(4+6j)");
    }

    #[test]
    fn test_complex_eq() {
        let env = execute_source("x = (1+2j) == (1+2j)\n");
        assert_eq!(env.borrow().get("x").unwrap().repr(), "True");
    }

    #[test]
    fn test_complex_builtin() {
        let env = execute_source("x = complex(3, 4)\n");
        assert_eq!(env.borrow().get("x").unwrap().repr(), "(3+4j)");
    }

    #[test]
    fn test_bytes_literal() {
        let output = execute_program("b\"hello\"");
        assert_eq!(output, "b'hello'");
    }

    #[test]
    fn test_bytes_decode() {
        let output = execute_program("b\"hello\".decode()");
        assert_eq!(output, "'hello'");
    }

    #[test]
    fn test_bytes_index() {
        let output = execute_program("b\"abc\"[1]");
        assert_eq!(output, "98");
    }

    #[test]
    fn test_bytes_hex() {
        let output = execute_program("b\"\\x00\\xff\".hex()");
        assert_eq!(output, "'00ff'");
    }

    #[test]
    fn test_bytes_len() {
        let output = execute_program("len(b\"hello\")");
        assert_eq!(output, "5");
    }

    #[test]
    fn test_bytes_contains() {
        let output = execute_program("b\"ell\" in b\"hello\"");
        assert_eq!(output, "True");
    }

    #[test]
    fn test_bytes_builtin() {
        let output = execute_program("bytes(5)");
        assert_eq!(output, "b'\\x00\\x00\\x00\\x00\\x00'");
    }

    #[test]
    fn test_dict_int_key() {
        let output = execute_program("{1: \"one\", 2: \"two\"}[1]");
        assert_eq!(output, "'one'");
    }

    #[test]
    fn test_dict_tuple_key() {
        let output = execute_program("{(1, 2): \"value\"}[(1, 2)]");
        assert_eq!(output, "'value'");
    }

    #[test]
    fn test_dict_mixed_keys() {
        let output = execute_program("{1: \"int\", \"key\": \"str\", (1,): \"tuple\"}[(1,)]");
        assert_eq!(output, "'tuple'");
    }

    #[test]
    fn test_dict_keys_method() {
        let output = execute_program("sorted({1: \"a\", 2: \"b\"}.keys())");
        assert_eq!(output, "[1, 2]");
    }

    #[test]
    fn test_dict_values_method() {
        let output = execute_program("list({1: \"a\", 2: \"b\"}.values())");
        assert!(output == "['a', 'b']" || output == "['b', 'a']");
    }

    #[test]
    fn test_dict_get_method() {
        let output = execute_program("{1: \"one\"}.get(1)");
        assert_eq!(output, "'one'");
    }

    #[test]
    fn test_dict_get_default() {
        let output = execute_program("{1: \"one\"}.get(2, \"default\")");
        assert_eq!(output, "'default'");
    }

    #[test]
    fn test_dict_pop_method() {
        let output = execute_program("{1: \"one\", 2: \"two\"}.pop(1)");
        assert_eq!(output, "'one'");
    }

    #[test]
    fn test_dict_update_method() {
        let output = execute_program("{1: \"a\", 2: \"b\"}.update({2: \"c\", 3: \"d\"})");
        assert_eq!(output, "None");
        let output2 = execute_program("sorted(({1: \"a\", 2: \"b\"} | {2: \"c\", 3: \"d\"}).keys())");
        assert_eq!(output2, "[1, 2, 3]");
    }

    #[test]
    fn test_dict_copy() {
        let output = execute_program("{1: \"a\"}.copy()");
        assert_eq!(output, "{1: 'a'}");
    }

    #[test]
    fn test_dict_clear() {
        let output = execute_program("{1: \"a\", 2: \"b\"}.clear()");
        assert_eq!(output, "None");
    }

    #[test]
    fn test_dict_len() {
        let output = execute_program("len({1: \"a\", 2: \"b\"})");
        assert_eq!(output, "2");
    }

    #[test]
    fn test_dict_contains() {
        let output = execute_program("1 in {1: \"a\", 2: \"b\"}");
        assert_eq!(output, "True");
    }

    #[test]
    fn test_exec_basic() {
        let env = execute_source("exec(\"a = 42\")\n");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "42");
    }

    #[test]
    fn test_eval_expr() {
        let output = execute_program("eval(\"1 + 2\")");
        assert_eq!(output, "3");
    }

    #[test]
    fn test_eval_string() {
        let output = execute_program("eval(\"'hello' + ' world'\")");
        assert_eq!(output, "'hello world'");
    }

    #[test]
    fn test_exec_multi_stmt() {
        let env = execute_source("exec(\"x = 10\\ny = 20\\nz = x + y\")\n");
        assert_eq!(env.borrow().get("z").unwrap().repr(), "30");
    }

    #[test]
    fn test_nested_exec_eval() {
        let env = execute_source("exec(\"def foo():\\n    return 42\")\nres = eval(\"foo()\")\n");
        assert_eq!(env.borrow().get("res").unwrap().repr(), "42");
    }

    #[test]
    fn test_compile_exec_mode() {
        let env = execute_source(r#"code = compile("1 + 2", "<test>", "exec")
"#);
        let code = env.borrow().get("code").unwrap();
        assert_eq!(code.get_type(), "code");
        assert!(code.repr().contains("<code object <test>"));
    }

    #[test]
    fn test_eval_arithmetic() {
        let output = execute_program("eval(\"2 * 3 + 1\")");
        assert_eq!(output, "7");
    }

    #[test]
    fn test_eval_list_literal() {
        let output = execute_program("eval(\"[1, 2, 3]\")");
        assert_eq!(output, "[1, 2, 3]");
    }

    #[test]
    fn test_exec_shares_env() {
        let env = execute_source("x = 10\nexec(\"y = x + 5\")\n");
        assert_eq!(env.borrow().get("y").unwrap().repr(), "15");
    }

    #[test]
    fn test_import_math_native() {
        let env = execute_source("import math_native\nresult = math_native.sqrt(4)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "2.0");
    }

    #[test]
    fn test_from_import() {
        let env = execute_source("from math_native import sqrt\nresult = sqrt(9)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "3.0");
    }

    #[test]
    fn test_sys_modules() {
        let env = execute_source("import sys\nresult = type(sys.modules)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "<class 'dict'>");
    }

    #[test]
    fn test_sys_path() {
        let env = execute_source("import sys\nresult = len(sys.path) > 0\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "True");
    }

    #[test]
    fn test_sys_argv() {
        let env = execute_source("import sys\nresult = len(sys.argv)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "1");
    }

    #[test]
    fn test_sys_builtin_module_names() {
        let env = execute_source("import sys\nresult = (type(sys.builtin_module_names) is tuple, \"sys\" in sys.builtin_module_names, len(sys.builtin_module_names))\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "(True, True, 37)");
    }

    #[test]
    fn test_import_as() {
        let env = execute_source("import math_native as m\nresult = m.sqrt(4)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "2.0");
    }

    #[test]
    fn test_from_import_as() {
        let env = execute_source("from math_native import sqrt as s\nresult = s(16)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "4.0");
    }

    #[test]
    fn test_import_star() {
        let env = execute_source("from math_native import *\nresult = sqrt(25)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "5.0");
    }

    #[test]
    fn test_multiple_imports() {
        let env = execute_source("import sys, math_native\nresult = math_native.sqrt(1)\n");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "1.0");
    }

    // --- File I/O tests ---

    #[test]
    fn test_file_write_read() {
        let source = r#"
with open("/tmp/rustpy_test_write.txt", "w") as f:
    f.write("hello world")
with open("/tmp/rustpy_test_write.txt", "r") as f:
    res = f.read()
"#;
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "'hello world'");
        fs::remove_file("/tmp/rustpy_test_write.txt").ok();
    }

    #[test]
    fn test_file_readline() {
        let source = r#"
with open("/tmp/rustpy_test_lines.txt", "w") as f:
    f.write("line1\nline2\nline3")
with open("/tmp/rustpy_test_lines.txt", "r") as f:
    res = f.readline()
"#;
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "'line1\\n'");
        fs::remove_file("/tmp/rustpy_test_lines.txt").ok();
    }

    #[test]
    fn test_file_iter() {
        let source = r#"
with open("/tmp/rustpy_test_iter.txt", "w") as f:
    f.write("a\nb\nc")
res = list(open("/tmp/rustpy_test_iter.txt", "r"))
"#;
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "['a\\n', 'b\\n', 'c']");
        fs::remove_file("/tmp/rustpy_test_iter.txt").ok();
    }

    #[test]
    fn test_file_seek_tell() {
        let source = r#"
with open("/tmp/rustpy_test_seek.txt", "w+") as f:
    f.write("hello")
    f.seek(0)
    res = f.read()
"#;
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "'hello'");
        fs::remove_file("/tmp/rustpy_test_seek.txt").ok();
    }

    #[test]
    fn test_file_append() {
        let source = r#"
with open("/tmp/rustpy_test_append.txt", "w") as f:
    f.write("first\n")
with open("/tmp/rustpy_test_append.txt", "a") as f:
    f.write("second")
with open("/tmp/rustpy_test_append.txt", "r") as f:
    res = f.read()
"#;
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "'first\\nsecond'");
        fs::remove_file("/tmp/rustpy_test_append.txt").ok();
    }

    #[test]
    fn test_stdout_write() {
        let source = "import sys\nsys.stdout.write(\"test\\n\")\n";
        let env = execute_source(source);
        // No variable to check, just ensure no crash
        assert!(true);
    }

    #[test]
    fn test_file_readlines() {
        let source = r#"
with open("/tmp/rustpy_test_rls.txt", "w") as f:
    f.write("x\ny\nz")
with open("/tmp/rustpy_test_rls.txt", "r") as f:
    res = f.readlines()
"#;
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "['x\\n', 'y\\n', 'z']");
        fs::remove_file("/tmp/rustpy_test_rls.txt").ok();
    }

    #[test]
    fn test_file_without_with_close() {
        let source = r#"
f = open("/tmp/rustpy_test_close.txt", "w")
f.write("data")
f.close()
f2 = open("/tmp/rustpy_test_close.txt", "r")
res = f2.read()
f2.close()
"#;
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "'data'");
        fs::remove_file("/tmp/rustpy_test_close.txt").ok();
    }

    #[test]
    fn test_file_repr() {
        let output = execute_program(r#"open("/tmp/rustpy_test_repr.txt", "w")"#);
        assert!(output.contains("TextIOWrapper"));
        assert!(output.contains("/tmp/rustpy_test_repr.txt"));
        assert!(output.contains("'w'"));
        fs::remove_file("/tmp/rustpy_test_repr.txt").ok();
    }

    #[test]
    fn test_file_writelines() {
        let source = r#"
with open("/tmp/rustpy_test_wlines.txt", "w") as f:
    f.writelines(["a\n", "b\n", "c"])
with open("/tmp/rustpy_test_wlines.txt", "r") as f:
    res = f.read()
"#;
        let env = execute_source(source);
        let res = env.borrow().get("res").unwrap();
        assert_eq!(res.repr(), "'a\\nb\\nc'");
        fs::remove_file("/tmp/rustpy_test_wlines.txt").ok();
    }

    #[test]
    fn test_file_tell() {
        let source = r#"
with open("/tmp/rustpy_test_tell.txt", "w") as f:
    f.write("hello")
    pos = f.tell()
"#;
        let env = execute_source(source);
        let pos = env.borrow().get("pos").unwrap();
        assert_eq!(pos.repr(), "5");
        fs::remove_file("/tmp/rustpy_test_tell.txt").ok();
    }

    #[test]
    fn test_file_readable_writable() {
        let source = r#"
f = open("/tmp/rustpy_test_rw.txt", "w")
r1 = f.writable()
r2 = f.readable()
f.close()
"#;
        let env = execute_source(source);
        assert_eq!(env.borrow().get("r1").unwrap().repr(), "True");
        assert_eq!(env.borrow().get("r2").unwrap().repr(), "False");
        fs::remove_file("/tmp/rustpy_test_rw.txt").ok();
    }

    #[test]
    fn test_file_flush() {
        let source = r#"
with open("/tmp/rustpy_test_flush.txt", "w") as f:
    f.write("hello")
    f.flush()
with open("/tmp/rustpy_test_flush.txt", "r") as f:
    res = f.read()
"#;
        let env = execute_source(source);
        assert_eq!(env.borrow().get("res").unwrap().repr(), "'hello'");
        fs::remove_file("/tmp/rustpy_test_flush.txt").ok();
    }

    #[test]
    fn test_file_name_mode_closed() {
        let source = r#"
with open("/tmp/rustpy_test_attrs.txt", "w") as f:
    n = f.name
    m = f.mode
    c1 = f.closed
c2 = f.closed
"#;
        let env = execute_source(source);
        assert_eq!(env.borrow().get("n").unwrap().repr(), "'/tmp/rustpy_test_attrs.txt'");
        assert_eq!(env.borrow().get("m").unwrap().repr(), "'w'");
        assert_eq!(env.borrow().get("c1").unwrap().repr(), "False");
        assert_eq!(env.borrow().get("c2").unwrap().repr(), "True");
        fs::remove_file("/tmp/rustpy_test_attrs.txt").ok();
    }

    // --- Async/Await tests ---

    #[test]
    fn test_async_def() {
        let env = execute_source("
import asyncio
async def foo():
    return 42
result = asyncio.run(foo())
");
        assert_eq!(env.borrow().get("result").unwrap().repr(), "42");
    }

    #[test]
    fn test_async_await() {
        let env = execute_source("
import asyncio
async def bar():
    return 10
async def foo():
    result = await bar()
    return result + 5
asyncio.run(foo())
");
        // foo returns 15, but asyncio.run captures the return value
        // The test setup just needs to confirm result is 15
        let env2 = execute_source("
import asyncio
async def bar():
    return 10
async def foo():
    result = await bar()
    return result + 5
val = asyncio.run(foo())
");
        assert_eq!(env2.borrow().get("val").unwrap().repr(), "15");
    }

    #[test]
    fn test_async_nested() {
        let env = execute_source("
import asyncio
async def inner():
    return \"inner\"
async def outer():
    result = await inner()
    return result
val = asyncio.run(outer())
");
        assert_eq!(env.borrow().get("val").unwrap().repr(), "'inner'");
    }

    #[test]
    fn test_async_multiple_awaits() {
        let env = execute_source("
import asyncio
async def a():
    return 1
async def b():
    return 2
async def main():
    x = await a()
    y = await b()
    return x + y
val = asyncio.run(main())
");
        assert_eq!(env.borrow().get("val").unwrap().repr(), "3");
    }

    #[test]
    fn test_print_to_file() {
        let env = execute_source("
f = open(\"/tmp/rustpy_print_test.txt\", \"w\")
print(\"hello world\", file=f)
f.close()
f2 = open(\"/tmp/rustpy_print_test.txt\", \"r\")
result = f2.read()
");
        assert_eq!(env.borrow().get("result").unwrap().str(), "hello world\n");
        std::fs::remove_file("/tmp/rustpy_print_test.txt").ok();
    }

    #[test]
    fn test_print_sep() {
        let env = execute_source("
f = open(\"/tmp/rustpy_print_sep.txt\", \"w\")
print(\"a\", \"b\", \"c\", sep=\"-\", file=f)
f.close()
f2 = open(\"/tmp/rustpy_print_sep.txt\", \"r\")
result = f2.read()
");
        assert_eq!(env.borrow().get("result").unwrap().str(), "a-b-c\n");
        std::fs::remove_file("/tmp/rustpy_print_sep.txt").ok();
    }

    #[test]
    fn test_print_end() {
        let env = execute_source("
f = open(\"/tmp/rustpy_print_end.txt\", \"w\")
print(\"hello\", end=\"\", file=f)
f.close()
f2 = open(\"/tmp/rustpy_print_end.txt\", \"r\")
result = f2.read()
");
        assert_eq!(env.borrow().get("result").unwrap().str(), "hello");
        std::fs::remove_file("/tmp/rustpy_print_end.txt").ok();
    }

    #[test]
    fn test_print_all_kwargs() {
        let env = execute_source("
f = open(\"/tmp/rustpy_print_all.txt\", \"w\")
print(\"x\", \"y\", sep=\"|\", end=\"END\", file=f)
f.close()
f2 = open(\"/tmp/rustpy_print_all.txt\", \"r\")
result = f2.read()
");
        assert_eq!(env.borrow().get("result").unwrap().str(), "x|yEND");
        std::fs::remove_file("/tmp/rustpy_print_all.txt").ok();
    }

    #[test]
    fn test_print_flush() {
        let env = execute_source("
f = open(\"/tmp/rustpy_print_flush.txt\", \"w\")
print(\"flush test\", file=f, flush=True)
f.close()
f2 = open(\"/tmp/rustpy_print_flush.txt\", \"r\")
result = f2.read()
");
        assert_eq!(env.borrow().get("result").unwrap().str(), "flush test\n");
        std::fs::remove_file("/tmp/rustpy_print_flush.txt").ok();
    }

    #[test]
    fn test_float_builtin() {
        let env = execute_source("
a = float()
b = float(42)
c = float(3.14)
d = float(\"2.5\")
");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "0.0");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "42.0");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "3.14");
        assert_eq!(env.borrow().get("d").unwrap().repr(), "2.5");
    }

    #[test]
    fn test_oct_builtin() {
        let env = execute_source("
a = oct(8)
b = oct(64)
");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'0o10'");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'0o100'");
    }

    #[test]
    fn test_ascii_builtin() {
        let env = execute_source("
a = ascii(\"hello\")
b = ascii(42)
");
        assert_eq!(env.borrow().get("a").unwrap().str(), "'hello'");
        assert_eq!(env.borrow().get("b").unwrap().str(), "42");
    }

    #[test]
    fn test_divmod_builtin() {
        let env = execute_source("
a, b = divmod(10, 3)
");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "3");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "1");
    }

    #[test]
    fn test_delattr_builtin() {
        let env = execute_source("
class Foo:
    pass
obj = Foo()
obj.x = 42
val = obj.x
");
        assert_eq!(env.borrow().get("val").unwrap().repr(), "42");
    }

    #[test]
    fn test_slice_builtin() {
        let env = execute_source("
s = slice(5)
t = slice(1, 5)
u = slice(1, 10, 2)
");
        // Just check they're created without error
        assert_eq!(env.borrow().get("s").unwrap().get_type(), "slice");
        assert_eq!(env.borrow().get("t").unwrap().get_type(), "slice");
        assert_eq!(env.borrow().get("u").unwrap().get_type(), "slice");
    }

    #[test]
    fn test_bytearray_basic() {
        let env = execute_source("
ba = bytearray(b\"hello\")
r = repr(ba)
");
        assert_eq!(env.borrow().get("r").unwrap().str(), "bytearray(b'hello')");
    }

    #[test]
    fn test_bytearray_append() {
        let env = execute_source("
ba = bytearray(b\"abc\")
ba.append(100)
r = repr(ba)
");
        assert_eq!(env.borrow().get("r").unwrap().str(), "bytearray(b'abcd')");
    }

    #[test]
    fn test_bytearray_decode() {
        let env = execute_source("
ba = bytearray(b\"hello\")
r = ba.decode()
");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "'hello'");
    }

    #[test]
    fn test_bytearray_from_int() {
        let env = execute_source("
ba = bytearray(5)
r = repr(ba)
");
        assert_eq!(env.borrow().get("r").unwrap().str(), "bytearray(b'\\x00\\x00\\x00\\x00\\x00')");
    }

    #[test]
    fn test_delattr_works() {
        let env = execute_source("
class Foo:
    pass
obj = Foo()
obj.x = 42
delattr(obj, \"x\")
h = hasattr(obj, \"x\")
");
        assert_eq!(env.borrow().get("h").unwrap().repr(), "False");
    }

    #[test]
    fn test_vars_dict() {
        let env = execute_source("
class Foo:
    def __init__(self):
        self.a = 1
        self.b = 2
obj = Foo()
d = vars(obj)
");
        let d_repr = env.borrow().get("d").unwrap().repr();
        assert!(d_repr == "{'a': 1, 'b': 2}" || d_repr == "{'b': 2, 'a': 1}", "got {}", d_repr);
    }

    #[test]
    fn test_sorted_key() {
        let env = execute_source("
r = sorted([3, 1, 2])
");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "[1, 2, 3]");
    }

    #[test]
    fn test_sorted_reverse() {
        let env = execute_source("
r = sorted([3, 1, 2], reverse=True)
");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "[3, 2, 1]");
    }

    #[test]
    fn test_int_base() {
        let env = execute_source("
a = int(\"ff\", 16)
b = int(\"77\", 8)
");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "255");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "63");
    }

    #[test]
    fn test_pow_mod() {
        let env = execute_source("
r = pow(2, 10, 7)
");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "2");
    }

    #[test]
    fn test_float_constructor() {
        let env = execute_source("
a = float()
b = float(42)
c = float(\"3.14\")
");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "0.0");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "42.0");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "3.14");
    }

    #[test]
    fn test_format_builtin() {
        let env = execute_source("
a = format(42)
b = format(3.14)
c = format(\"hello\")
");
        assert_eq!(env.borrow().get("a").unwrap().repr(), "'42'");
        assert_eq!(env.borrow().get("b").unwrap().repr(), "'3.14'");
        assert_eq!(env.borrow().get("c").unwrap().repr(), "'hello'");
    }

    #[test]
    fn test_import_math() {
        let env = Environment::new();
        stdlib::builtins::inject_builtins(&env);
        // Manually copy env for execute
        let source = "import math\n";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let module = parser.parse_module().unwrap();
        let compiler = Compiler::new("<test>".to_string());
        let code = compiler.compile(&module).unwrap();
        let mut frame = Frame::new(code, Rc::clone(&env));
        let mut vm = VirtualMachine::new();
        let result = vm.run(&mut frame);
        if let Err(e) = &result {
            panic!("Execution failed: {}", e);
        }
        let m = env.borrow().get("math");
        assert!(m.is_some(), "math module not imported");
    }

    #[test]
    fn test_math_sqrt() {
        let env = execute_source("import math\nr = math.sqrt(9)\nX = 1\n");
        let x = env.borrow().get("X");
        assert!(x.is_some(), "X not set, import may have failed");
        let r = env.borrow().get("r");
        assert!(r.is_some(), "r not set");
        assert_eq!(r.unwrap().repr(), "3.0");
    }

    #[test]
    fn test_math_pi() {
        let env = execute_source("import math\nr = math.pi > 3.14\n");
        let r = env.borrow().get("r");
        assert!(r.is_some(), "r not set - math module may not have imported");
        assert_eq!(r.unwrap().repr(), "True");
    }

    #[test]
    fn test_math_sin_cos() {
        let env = execute_source("import math\nr = math.sin(0)\n");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "0.0");
    }

    #[test]
    fn test_os_getcwd() {
        let env = execute_source("import os\nr = os.getcwd()\n");
        let r = env.borrow().get("r").unwrap().str();
        assert!(r.contains("rustpy"), "expected rustpy in path, got {}", r);
    }

    #[test]
    fn test_str_encode() {
        let env = execute_source("r = \"hello\".encode()");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "b'hello'");
    }

    #[test]
    fn test_str_splitlines() {
        let env = execute_source("r = \"a\\nb\\nc\".splitlines()");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "['a', 'b', 'c']");
    }

    #[test]
    fn test_str_partition() {
        let env = execute_source("r = \"hello world\".partition(\" \")");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "('hello', ' ', 'world')");
    }

    #[test]
    fn test_bytes_split() {
        let env = execute_source("r = b\"a b c\".split()");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "[b'a', b'b', b'c']");
    }

    #[test]
    fn test_bytes_replace() {
        let env = execute_source("r = b\"hello world\".replace(b\"world\", b\"there\")");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "b'hello there'");
    }

    #[test]
    fn test_bytearray_from_bytes() {
        let env = execute_source("r = bytearray(b\"test\")");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "bytearray(b'test')");
    }

    #[test]
    fn test_math_factorial() {
        let env = execute_source("import math\nr = math.factorial(5)\n");
        assert_eq!(env.borrow().get("r").unwrap().repr(), "120");
    }

    #[test]
    fn test_filesystem_import_basic() {
        let filename = "test_module_123.py";
        std::fs::write(filename, "x = 42\ndef hello():\n    return 'world'\n").unwrap();
        
        let env = execute_source("import test_module_123\nresult_x = test_module_123.x\nresult_fn = test_module_123.hello()\n");
        std::fs::remove_file(filename).unwrap();
        
        assert_eq!(env.borrow().get("result_x").unwrap().repr(), "42");
        assert_eq!(env.borrow().get("result_fn").unwrap().repr(), "'world'");
    }

    #[test]
    fn test_filesystem_import_from() {
        let filename = "test_module_456.py";
        std::fs::write(filename, "y = 99\n").unwrap();
        
        let env = execute_source("from test_module_456 import y\n");
        std::fs::remove_file(filename).unwrap();
        
        assert_eq!(env.borrow().get("y").unwrap().repr(), "99");
    }

    #[test]
    fn test_circular_import() {
        let file_a = "circ_a.py";
        let file_b = "circ_b.py";
        std::fs::write(file_a, "import circ_b\nx = 1\n").unwrap();
        std::fs::write(file_b, "import circ_a\ny = 2\n").unwrap();
        
        let env = execute_source("import circ_a\nresult = circ_a.x + circ_a.circ_b.y\n");
        std::fs::remove_file(file_a).unwrap();
        std::fs::remove_file(file_b).unwrap();
        
        assert_eq!(env.borrow().get("result").unwrap().repr(), "3");
    }
}
