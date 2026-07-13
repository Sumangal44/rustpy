mod ast;
mod diagnostics;
mod lexer;
mod parser;

use lexer::Lexer;
use parser::Parser;

fn main() {
    println!("RustPy Interpreter - Phase 2");
    let source = "def add(a, b):\n    return a + b\n";

    let lexer = Lexer::new(source);
    match Parser::new(lexer) {
        Ok(mut parser) => match parser.parse_module() {
            Ok(module) => {
                println!("Parsed AST: {:#?}", module);
            }
            Err(e) => {
                eprintln!("Parser error: {} at line {}", e, e.span.line);
            }
        },
        Err(e) => {
            eprintln!("Initialization error: {} at line {}", e, e.span.line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinOpKind, Expr, Stmt};

    #[test]
    fn test_parse_assignment() {
        let source = "a = 1 + 2\n";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let module = parser.parse_module().unwrap();

        assert_eq!(module.body.len(), 1);
        if let Stmt::Assign { targets, value } = &module.body[0] {
            assert_eq!(targets.len(), 1);
            assert_eq!(targets[0], Expr::Identifier("a".to_string()));

            if let Expr::BinOp { left, op, right } = value {
                assert_eq!(**left, Expr::IntLiteral(1));
                assert_eq!(*op, BinOpKind::Add);
                assert_eq!(**right, Expr::IntLiteral(2));
            } else {
                panic!("Expected BinOp");
            }
        } else {
            panic!("Expected Assign statement");
        }
    }

    #[test]
    fn test_parse_function_def() {
        let source = "def foo(x):\n    return x * 2\n";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let module = parser.parse_module().unwrap();

        assert_eq!(module.body.len(), 1);
        if let Stmt::FunctionDef { name, params, body } = &module.body[0] {
            assert_eq!(name, "foo");
            assert_eq!(params, &vec!["x".to_string()]);
            assert_eq!(body.len(), 1);

            if let Stmt::Return {
                value: Some(Expr::BinOp { left, op, right }),
            } = &body[0]
            {
                assert_eq!(**left, Expr::Identifier("x".to_string()));
                assert_eq!(*op, BinOpKind::Mult);
                assert_eq!(**right, Expr::IntLiteral(2));
            } else {
                panic!("Expected Return BinOp inside FunctionDef");
            }
        } else {
            panic!("Expected FunctionDef");
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let source = "if True:\n    pass\n";
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer).unwrap();
        let module = parser.parse_module().unwrap();

        assert_eq!(module.body.len(), 1);
        if let Stmt::If { test, body, orelse } = &module.body[0] {
            assert_eq!(*test, Expr::BooleanLiteral(true));
            assert_eq!(body.len(), 1);
            assert_eq!(body[0], Stmt::Pass);
            assert!(orelse.is_empty());
        } else {
            panic!("Expected If statement");
        }
    }
}
