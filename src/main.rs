mod diagnostics;
mod lexer;

use lexer::Lexer;
use lexer::tokens::TokenKind;

fn main() {
    println!("RustPy Interpreter - Phase 1");
    let source = "def add(a, b):\n    return a + b\n";
    let mut lexer = Lexer::new(source);

    loop {
        match lexer.next_token() {
            Ok(token) => {
                println!("{:?}", token);
                if token.kind == TokenKind::EOF {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Lexer error: {} at line {}", e, e.span.line);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokens::TokenKind;

    #[test]
    fn test_basic_tokens() {
        let source = "def add(a, b): return a + b";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Def);
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Identifier("add".to_string())
        );
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LParen);
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Identifier("a".to_string())
        );
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Comma);
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Identifier("b".to_string())
        );
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Return);
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Identifier("a".to_string())
        );
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Plus);
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Identifier("b".to_string())
        );
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::EOF);
    }

    #[test]
    fn test_indentation() {
        let source = "if True:\n    pass\n";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::If);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::True);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Newline);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Indent);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Pass);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Newline);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Dedent);
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::EOF);
    }
}
