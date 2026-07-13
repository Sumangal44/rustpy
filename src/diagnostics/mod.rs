use crate::lexer::tokens::Span;

pub fn format_error(source: &str, span: &Span, message: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if span.line > 0 && span.line <= lines.len() {
        let line_content = lines[span.line - 1];
        let marker = format!("{:>width$}", "^", width = span.column + 1);

        format!(
            "Error: {}\n --> line {}:{}\n  |\n{} | {}\n  | {}",
            message, span.line, span.column, span.line, line_content, marker
        )
    } else {
        format!("Error: {} at line {}:{}", message, span.line, span.column)
    }
}

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
        let msg = match &self.kind {
            LexerErrorKind::UnexpectedCharacter(c) => format!("Unexpected character: '{}'", c),
            LexerErrorKind::UnterminatedString => "Unterminated string literal".to_string(),
            LexerErrorKind::InvalidNumber => "Invalid number literal".to_string(),
            LexerErrorKind::IndentationError => "Indentation error".to_string(),
        };
        write!(f, "{}", msg)
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
        let msg = match &self.kind {
            ParseErrorKind::UnexpectedToken(t) => format!("Unexpected token: {}", t),
            ParseErrorKind::UnexpectedEOF => "Unexpected EOF".to_string(),
            ParseErrorKind::InvalidSyntax(m) => format!("Invalid syntax: {}", m),
            ParseErrorKind::LexerError(e) => format!("Lexer error: {:?}", e),
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for ParseError {}
