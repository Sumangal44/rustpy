pub mod tokens;

use crate::diagnostics::{LexerError, LexerErrorKind};
use tokens::{Span, Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::str::Chars<'a>,
    current: Option<char>,
    pos: usize,
    line: usize,
    column: usize,

    // For tracking indentation
    indent_stack: Vec<usize>,
    pending_tokens: Vec<Token>,
    at_line_start: bool,
    paren_level: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Self {
            source,
            chars: source.chars(),
            current: None,
            pos: 0,
            line: 1,
            column: 0,
            indent_stack: vec![0],
            pending_tokens: Vec::new(),
            at_line_start: true,
            paren_level: 0,
        };
        lexer.advance();
        lexer
    }

    fn advance(&mut self) {
        if let Some(c) = self.current {
            self.pos += c.len_utf8();
            if c == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
        self.current = self.chars.next();
    }

    fn peek(&self) -> Option<char> {
        self.chars.clone().next()
    }

    fn span(&self, start_pos: usize, start_col: usize) -> Span {
        Span::new(start_pos, self.pos, self.line, start_col)
    }

    fn make_token(&self, kind: TokenKind, start_pos: usize, start_col: usize) -> Token {
        Token::new(kind, self.span(start_pos, start_col))
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        if !self.pending_tokens.is_empty() {
            return Ok(self.pending_tokens.remove(0));
        }

        self.skip_whitespace_and_comments();

        let start_pos = self.pos;
        let start_col = self.column;

        // Handle indentation at the start of a logical line
        if self.at_line_start && self.paren_level == 0 {
            self.at_line_start = false;
            let current_indent = start_col;
            let last_indent = *self.indent_stack.last().unwrap();

            if current_indent > last_indent {
                self.indent_stack.push(current_indent);
                return Ok(self.make_token(TokenKind::Indent, start_pos, start_col));
            } else if current_indent < last_indent {
                while let Some(&top) = self.indent_stack.last() {
                    if top <= current_indent {
                        break;
                    }
                    self.indent_stack.pop();
                    self.pending_tokens.push(self.make_token(
                        TokenKind::Dedent,
                        start_pos,
                        start_col,
                    ));
                }

                if *self.indent_stack.last().unwrap() != current_indent {
                    return Err(LexerError::new(
                        LexerErrorKind::IndentationError,
                        self.span(start_pos, start_col),
                    ));
                }

                if !self.pending_tokens.is_empty() {
                    return Ok(self.pending_tokens.remove(0));
                }
            }
        }

        if self.current.is_none() {
            if self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                return Ok(self.make_token(TokenKind::Dedent, start_pos, start_col));
            }
            return Ok(self.make_token(TokenKind::EOF, start_pos, start_col));
        }

        let c = self.current.unwrap();

        // Newlines
        if c == '\n' {
            self.advance();
            self.at_line_start = true;
            if self.paren_level == 0 {
                return Ok(self.make_token(TokenKind::Newline, start_pos, start_col));
            } else {
                return self.next_token();
            }
        }

        // Identifiers and Keywords
        if c.is_alphabetic() || c == '_' {
            return self.lex_identifier_or_keyword(start_pos, start_col);
        }

        // Numbers
        if c.is_ascii_digit() {
            return self.lex_number(start_pos, start_col);
        }

        // Strings
        if c == '"' || c == '\'' {
            return self.lex_string(start_pos, start_col, c);
        }

        // Operators and Punctuation
        self.lex_operator(start_pos, start_col)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.current {
                Some(' ') | Some('\t') => {
                    self.advance();
                }
                Some('#') => {
                    while let Some(c) = self.current {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                Some('\\') => {
                    if self.peek() == Some('\n') {
                        self.advance(); // consume \
                        self.advance(); // consume \n
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn lex_identifier_or_keyword(
        &mut self,
        start_pos: usize,
        start_col: usize,
    ) -> Result<Token, LexerError> {
        let mut value = String::new();
        while let Some(c) = self.current {
            if c.is_alphanumeric() || c == '_' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let kind = match value.as_str() {
            "def" => TokenKind::Def,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elif" => TokenKind::Elif,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "class" => TokenKind::Class,
            "pass" => TokenKind::Pass,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "import" => TokenKind::Import,
            "from" => TokenKind::From,
            "as" => TokenKind::As,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "is" => TokenKind::Is,
            "None" => TokenKind::None,
            "True" => TokenKind::True,
            "False" => TokenKind::False,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "yield" => TokenKind::Yield,
            "try" => TokenKind::Try,
            "except" => TokenKind::Except,
            "finally" => TokenKind::Finally,
            "raise" => TokenKind::Raise,
            "with" => TokenKind::With,
            "lambda" => TokenKind::Lambda,
            "global" => TokenKind::Global,
            "nonlocal" => TokenKind::Nonlocal,
            "del" => TokenKind::Del,
            "match" => TokenKind::Match,
            "case" => TokenKind::Case,
            _ => TokenKind::Identifier(value),
        };

        Ok(self.make_token(kind, start_pos, start_col))
    }

    fn lex_number(&mut self, start_pos: usize, start_col: usize) -> Result<Token, LexerError> {
        let mut value = String::new();
        let mut is_float = false;

        while let Some(c) = self.current {
            if c.is_ascii_digit() {
                value.push(c);
                self.advance();
            } else if c == '.' {
                if is_float {
                    break;
                }
                is_float = true;
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let kind = if is_float {
            let num = value.parse::<f64>().map_err(|_| {
                LexerError::new(
                    LexerErrorKind::InvalidNumber,
                    self.span(start_pos, start_col),
                )
            })?;
            TokenKind::FloatLiteral(num)
        } else {
            let num = value.parse::<i64>().map_err(|_| {
                LexerError::new(
                    LexerErrorKind::InvalidNumber,
                    self.span(start_pos, start_col),
                )
            })?;
            TokenKind::IntLiteral(num)
        };

        Ok(self.make_token(kind, start_pos, start_col))
    }

    fn lex_string(
        &mut self,
        start_pos: usize,
        start_col: usize,
        quote: char,
    ) -> Result<Token, LexerError> {
        self.advance(); // consume quote
        let mut value = String::new();
        let mut escape = false;

        while let Some(c) = self.current {
            if escape {
                let escaped_char = match c {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    _ => c,
                };
                value.push(escaped_char);
                escape = false;
                self.advance();
            } else if c == '\\' {
                escape = true;
                self.advance();
            } else if c == quote {
                self.advance(); // consume closing quote
                return Ok(self.make_token(TokenKind::StringLiteral(value), start_pos, start_col));
            } else {
                value.push(c);
                self.advance();
            }
        }

        Err(LexerError::new(
            LexerErrorKind::UnterminatedString,
            self.span(start_pos, start_col),
        ))
    }

    fn lex_operator(&mut self, start_pos: usize, start_col: usize) -> Result<Token, LexerError> {
        let c = self.current.unwrap();
        let next_c = self.peek();

        let (kind, len) = match (c, next_c) {
            ('+', Some('=')) => (TokenKind::PlusEqual, 2),
            ('+', _) => (TokenKind::Plus, 1),
            ('-', Some('=')) => (TokenKind::MinusEqual, 2),
            ('-', Some('>')) => (TokenKind::Arrow, 2),
            ('-', _) => (TokenKind::Minus, 1),
            ('*', Some('*')) => {
                if let Some('=') = {
                    let mut iter = self.chars.clone();
                    iter.next();
                    iter.next()
                } {
                    (TokenKind::DoubleStarEqual, 3)
                } else {
                    (TokenKind::DoubleStar, 2)
                }
            }
            ('*', Some('=')) => (TokenKind::StarEqual, 2),
            ('*', _) => (TokenKind::Star, 1),
            ('/', Some('/')) => {
                if let Some('=') = {
                    let mut iter = self.chars.clone();
                    iter.next();
                    iter.next()
                } {
                    (TokenKind::DoubleSlashEqual, 3)
                } else {
                    (TokenKind::DoubleSlash, 2)
                }
            }
            ('/', Some('=')) => (TokenKind::SlashEqual, 2),
            ('/', _) => (TokenKind::Slash, 1),
            ('=', Some('=')) => (TokenKind::EqualEqual, 2),
            ('=', _) => (TokenKind::Equal, 1),
            ('!', Some('=')) => (TokenKind::NotEqual, 2),
            ('<', Some('=')) => (TokenKind::LessEqual, 2),
            ('<', Some('<')) => {
                if let Some('=') = {
                    let mut iter = self.chars.clone();
                    iter.next();
                    iter.next()
                } {
                    (TokenKind::LeftShiftEqual, 3)
                } else {
                    (TokenKind::LeftShift, 2)
                }
            }
            ('<', _) => (TokenKind::Less, 1),
            ('>', Some('=')) => (TokenKind::GreaterEqual, 2),
            ('>', Some('>')) => {
                if let Some('=') = {
                    let mut iter = self.chars.clone();
                    iter.next();
                    iter.next()
                } {
                    (TokenKind::RightShiftEqual, 3)
                } else {
                    (TokenKind::RightShift, 2)
                }
            }
            ('>', _) => (TokenKind::Greater, 1),
            ('(', _) => {
                self.paren_level += 1;
                (TokenKind::LParen, 1)
            }
            (')', _) => {
                self.paren_level = self.paren_level.saturating_sub(1);
                (TokenKind::RParen, 1)
            }
            ('[', _) => {
                self.paren_level += 1;
                (TokenKind::LBracket, 1)
            }
            (']', _) => {
                self.paren_level = self.paren_level.saturating_sub(1);
                (TokenKind::RBracket, 1)
            }
            ('{', _) => {
                self.paren_level += 1;
                (TokenKind::LBrace, 1)
            }
            ('}', _) => {
                self.paren_level = self.paren_level.saturating_sub(1);
                (TokenKind::RBrace, 1)
            }
            (',', _) => (TokenKind::Comma, 1),
            (':', _) => (TokenKind::Colon, 1),
            ('.', _) => (TokenKind::Dot, 1),
            _ => {
                return Err(LexerError::new(
                    LexerErrorKind::UnexpectedCharacter(c),
                    self.span(start_pos, start_col),
                ));
            }
        };

        for _ in 0..len {
            self.advance();
        }

        Ok(self.make_token(kind, start_pos, start_col))
    }
}
