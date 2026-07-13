#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Def,
    Return,
    If,
    Else,
    Elif,
    While,
    For,
    In,
    Class,
    Pass,
    Break,
    Continue,
    Import,
    From,
    As,
    And,
    Or,
    Not,
    Is,
    None,
    True,
    False,
    Async,
    Await,
    Yield,
    Try,
    Except,
    Finally,
    Raise,
    With,
    Lambda,
    Global,
    Nonlocal,
    Del,
    Match,
    Case,

    // Identifiers and Literals
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BytesLiteral(Vec<u8>),

    // Operators and Punctuation
    Plus,
    Minus,
    Star,
    Slash,
    DoubleSlash,
    Percent,
    At,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    Equal,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    DoubleSlashEqual,
    PercentEqual,
    AtEqual,
    AmpersandEqual,
    PipeEqual,
    CaretEqual,
    DoubleStar,
    DoubleStarEqual,
    LeftShift,
    RightShift,
    LeftShiftEqual,
    RightShiftEqual,

    // Brackets
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // Delimiters
    Comma,
    Colon,
    Dot,
    Semicolon,
    Arrow,

    // Structural
    Indent,
    Dedent,
    Newline,

    // Special
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
