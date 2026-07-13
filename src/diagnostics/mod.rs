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

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    UnexpectedToken(String),
    UnexpectedEOF,
    InvalidSyntax(String),
    LexerError(LexerErrorKind),
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            ParseErrorKind::UnexpectedToken(t) => write!(f, "Unexpected token: {}", t),
            ParseErrorKind::UnexpectedEOF => write!(f, "Unexpected EOF"),
            ParseErrorKind::InvalidSyntax(m) => write!(f, "Invalid syntax: {}", m),
            ParseErrorKind::LexerError(e) => write!(f, "Lexer error: {:?}", e),
        }
    }
}

impl std::error::Error for ParseError {}
