use crate::lexer::tokens::Span;

#[derive(Debug, Clone)]
pub enum LexerErrorKind {
    UnexpectedCharacter(char),
    UnterminatedString,
    InvalidNumber,
    IndentationError,
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub span: Span,
}

impl LexerError {
    pub fn new(kind: LexerErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            LexerErrorKind::UnexpectedCharacter(c) => write!(f, "Unexpected character: '{}'", c),
            LexerErrorKind::UnterminatedString => write!(f, "Unterminated string literal"),
            LexerErrorKind::InvalidNumber => write!(f, "Invalid number literal"),
            LexerErrorKind::IndentationError => write!(f, "Indentation error"),
        }
    }
}

impl std::error::Error for LexerError {}
