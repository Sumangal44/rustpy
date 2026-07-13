use crate::ast::{BinOpKind, Expr, Module, Stmt, UnaryOpKind};
use crate::diagnostics::{LexerError, ParseError, ParseErrorKind};
use crate::lexer::Lexer;
use crate::lexer::tokens::{Span, Token, TokenKind};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Result<Self, ParseError> {
        let current_token = match lexer.next_token() {
            Ok(t) => t,
            Err(e) => return Err(Self::lexer_error_to_parse_error(e)),
        };
        let peek_token = match lexer.next_token() {
            Ok(t) => t,
            Err(e) => return Err(Self::lexer_error_to_parse_error(e)),
        };

        Ok(Self {
            lexer,
            current_token,
            peek_token,
        })
    }

    fn lexer_error_to_parse_error(e: LexerError) -> ParseError {
        ParseError::new(ParseErrorKind::LexerError(e.kind), e.span)
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        self.current_token = self.peek_token.clone();
        self.peek_token = match self.lexer.next_token() {
            Ok(t) => t,
            Err(e) => return Err(Self::lexer_error_to_parse_error(e)),
        };
        Ok(())
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.current_token.kind == *kind
    }

    fn check_peek(&self, kind: &TokenKind) -> bool {
        self.peek_token.kind == *kind
    }

    fn consume(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind == kind {
            self.advance()?;
            Ok(())
        } else {
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken(format!("{:?}", self.current_token.kind)),
                self.current_token.span.clone(),
            ))
        }
    }

    fn consume_newlines(&mut self) -> Result<(), ParseError> {
        while self.check(&TokenKind::Newline) {
            self.advance()?;
        }
        Ok(())
    }

    pub fn parse_module(&mut self) -> Result<Module, ParseError> {
        let mut body = Vec::new();

        self.consume_newlines()?;
        while !self.check(&TokenKind::EOF) {
            body.push(self.parse_statement()?);
            self.consume_newlines()?;
        }

        Ok(Module { body })
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match &self.current_token.kind {
            TokenKind::Def => self.parse_function_def(),
            TokenKind::Class => self.parse_class_def(),
            TokenKind::Try => self.parse_try(),
            TokenKind::Raise => self.parse_raise(),
            TokenKind::Return => self.parse_return(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Pass => {
                self.advance()?;
                self.consume(TokenKind::Newline)?;
                Ok(Stmt::Pass)
            }
            _ => self.parse_assign_or_expr(),
        }
    }

    fn parse_function_def(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Def)?;

        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken("Expected function name".to_string()),
                    self.current_token.span.clone(),
                ));
            }
        };
        self.advance()?;

        self.consume(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                match &self.current_token.kind {
                    TokenKind::Identifier(n) => {
                        params.push(n.clone());
                        self.advance()?;
                    }
                    _ => {
                        return Err(ParseError::new(
                            ParseErrorKind::UnexpectedToken("Expected parameter name".to_string()),
                            self.current_token.span.clone(),
                        ));
                    }
                }

                if self.check(&TokenKind::Comma) {
                    self.advance()?;
                } else {
                    break;
                }
            }
        }
        self.consume(TokenKind::RParen)?;
        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;

        let body = self.parse_block()?;

        Ok(Stmt::FunctionDef { name, params, body })
    }

    fn parse_class_def(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Class)?;

        let name = match &self.current_token.kind {
            TokenKind::Identifier(n) => n.clone(),
            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken("Expected class name".to_string()),
                    self.current_token.span.clone(),
                ));
            }
        };
        self.advance()?;

        // Optional inheritance (skip for Phase 11)
        if self.check(&TokenKind::LParen) {
            self.advance()?;
            // Ignoring base classes for now, just consume them
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                self.advance()?;
            }
            self.consume(TokenKind::RParen)?;
        }

        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;

        let body = self.parse_block()?;

        Ok(Stmt::ClassDef { name, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.consume(TokenKind::Indent)?;
        let mut body = Vec::new();

        while !self.check(&TokenKind::Dedent) && !self.check(&TokenKind::EOF) {
            self.consume_newlines()?;
            if self.check(&TokenKind::Dedent) {
                break;
            }
            body.push(self.parse_statement()?);
        }

        self.consume(TokenKind::Dedent)?;
        Ok(body)
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Return)?;

        let value = if self.check(&TokenKind::Newline) || self.check(&TokenKind::EOF) {
            None
        } else {
            Some(self.parse_expression(0)?)
        };

        if self.check(&TokenKind::Newline) {
            self.advance()?;
        }

        Ok(Stmt::Return { value })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::If)?;
        let test = self.parse_expression(0)?;
        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;

        let body = self.parse_block()?;
        let mut orelse = Vec::new();

        if self.check(&TokenKind::Else) {
            self.advance()?;
            self.consume(TokenKind::Colon)?;
            self.consume(TokenKind::Newline)?;
            orelse = self.parse_block()?;
        } else if self.check(&TokenKind::Elif) {
            // Transform elif into an else with a nested if
            // For now, let's just parse it as if
            // This is a simplification for Phase 2
            let elif_stmt = self.parse_if()?;
            orelse.push(elif_stmt);
        }

        Ok(Stmt::If { test, body, orelse })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::While)?;
        let test = self.parse_expression(0)?;
        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;

        let body = self.parse_block()?;

        Ok(Stmt::While { test, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::For)?;
        let target = self.parse_expression(0)?;
        self.consume(TokenKind::In)?;
        let iter = self.parse_expression(0)?;
        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;

        let body = self.parse_block()?;

        Ok(Stmt::For { target, iter, body })
    }

    fn parse_try(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Try)?;
        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;

        let body = self.parse_block()?;

        let mut handlers = Vec::new();
        while self.check(&TokenKind::Except) {
            self.advance()?;

            // For now, support bare `except:` or `except Exception:` but don't bind it.
            // Actually, we'll just skip the exception type if it's there.
            if !self.check(&TokenKind::Colon) {
                // Read whatever expression is there
                self.parse_expression(0)?;
                // Ignore `as X` for now
                if self.check(&TokenKind::As) {
                    self.advance()?;
                    self.parse_expression(0)?;
                }
            }

            self.consume(TokenKind::Colon)?;
            self.consume(TokenKind::Newline)?;

            let handler_body = self.parse_block()?;
            handlers.push(("Exception".to_string(), handler_body));
        }

        Ok(Stmt::Try { body, handlers })
    }

    fn parse_raise(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Raise)?;
        let exc = self.parse_expression(0)?;
        if self.check(&TokenKind::Newline) {
            self.advance()?;
        }
        Ok(Stmt::Raise { exc: Box::new(exc) })
    }

    fn parse_assign_or_expr(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expression(0)?;

        if self.check(&TokenKind::Equal) {
            self.advance()?;
            let value = self.parse_expression(0)?;
            self.consume(TokenKind::Newline)?;
            Ok(Stmt::Assign {
                targets: vec![expr],
                value,
            })
        } else {
            if self.check(&TokenKind::Newline) {
                self.advance()?;
            }
            Ok(Stmt::ExprStmt { value: expr })
        }
    }

    fn parse_expression(&mut self, precedence: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_prefix()?;

        while precedence < self.peek_precedence() {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        match &self.current_token.kind {
            TokenKind::Identifier(name) => {
                let expr = Expr::Identifier(name.clone());
                self.advance()?;
                Ok(expr)
            }
            TokenKind::IntLiteral(val) => {
                let expr = Expr::IntLiteral(*val);
                self.advance()?;
                Ok(expr)
            }
            TokenKind::FloatLiteral(val) => {
                let expr = Expr::FloatLiteral(*val);
                self.advance()?;
                Ok(expr)
            }
            TokenKind::StringLiteral(val) => {
                let expr = Expr::StringLiteral(val.clone());
                self.advance()?;
                Ok(expr)
            }
            TokenKind::True => {
                self.advance()?;
                Ok(Expr::BooleanLiteral(true))
            }
            TokenKind::False => {
                self.advance()?;
                Ok(Expr::BooleanLiteral(false))
            }
            TokenKind::None => {
                self.advance()?;
                Ok(Expr::NoneLiteral)
            }
            TokenKind::LParen => {
                self.advance()?;
                let expr = self.parse_expression(0)?;
                self.consume(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBracket => {
                self.advance()?;
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        elements.push(self.parse_expression(0)?);
                        if self.check(&TokenKind::Comma) {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RBracket)?;
                Ok(Expr::List(elements))
            }
            TokenKind::LBrace => {
                self.advance()?;
                let mut pairs = Vec::new();
                if !self.check(&TokenKind::RBrace) {
                    loop {
                        let key = self.parse_expression(0)?;
                        self.consume(TokenKind::Colon)?;
                        let value = self.parse_expression(0)?;
                        pairs.push((key, value));
                        if self.check(&TokenKind::Comma) {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RBrace)?;
                Ok(Expr::Dict(pairs))
            }
            TokenKind::Minus | TokenKind::Plus | TokenKind::Not => {
                let op = match self.current_token.kind {
                    TokenKind::Minus => UnaryOpKind::Minus,
                    TokenKind::Plus => UnaryOpKind::Plus,
                    TokenKind::Not => UnaryOpKind::Not,
                    _ => unreachable!(),
                };
                self.advance()?;
                let operand = Box::new(self.parse_expression(6)?); // Unary precedence
                Ok(Expr::UnaryOp { op, operand })
            }
            _ => Err(ParseError::new(
                ParseErrorKind::InvalidSyntax(format!(
                    "Expected expression, got {:?}",
                    self.current_token.kind
                )),
                self.current_token.span.clone(),
            )),
        }
    }

    fn parse_infix(&mut self, left: Expr) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::LParen) {
            return self.parse_call(left);
        } else if self.check(&TokenKind::LBracket) {
            return self.parse_subscript(left);
        } else if self.check(&TokenKind::Dot) {
            return self.parse_attribute(left);
        }

        let op = match self.current_token.kind {
            TokenKind::Plus => BinOpKind::Add,
            TokenKind::Minus => BinOpKind::Sub,
            TokenKind::Star => BinOpKind::Mult,
            TokenKind::Slash => BinOpKind::Div,
            TokenKind::DoubleSlash => BinOpKind::FloorDiv,
            TokenKind::Percent => BinOpKind::Mod,
            TokenKind::DoubleStar => BinOpKind::Pow,
            TokenKind::EqualEqual => BinOpKind::Eq,
            TokenKind::NotEqual => BinOpKind::NotEq,
            TokenKind::Less => BinOpKind::Lt,
            TokenKind::LessEqual => BinOpKind::LtEq,
            TokenKind::Greater => BinOpKind::Gt,
            TokenKind::GreaterEqual => BinOpKind::GtEq,
            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidSyntax(format!(
                        "Expected infix operator, got {:?}",
                        self.current_token.kind
                    )),
                    self.current_token.span.clone(),
                ));
            }
        };

        let precedence = self.current_precedence();
        self.advance()?; // Consume operator

        let right = self.parse_expression(precedence)?;

        Ok(Expr::BinOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    fn parse_call(&mut self, func: Expr) -> Result<Expr, ParseError> {
        self.consume(TokenKind::LParen)?;

        let mut args = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                args.push(self.parse_expression(0)?);
                if self.check(&TokenKind::Comma) {
                    self.advance()?;
                } else {
                    break;
                }
            }
        }

        self.consume(TokenKind::RParen)?;

        Ok(Expr::Call {
            func: Box::new(func),
            args,
        })
    }

    fn parse_subscript(&mut self, value: Expr) -> Result<Expr, ParseError> {
        self.consume(TokenKind::LBracket)?;
        let slice = self.parse_expression(0)?;
        self.consume(TokenKind::RBracket)?;
        Ok(Expr::Subscript {
            value: Box::new(value),
            slice: Box::new(slice),
        })
    }

    fn parse_attribute(&mut self, value: Expr) -> Result<Expr, ParseError> {
        self.consume(TokenKind::Dot)?;

        let attr = match &self.current_token.kind {
            TokenKind::Identifier(name) => name.clone(),
            _ => {
                return Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken("Expected attribute name".to_string()),
                    self.current_token.span.clone(),
                ));
            }
        };
        self.advance()?;

        Ok(Expr::Attribute {
            value: Box::new(value),
            attr,
        })
    }

    fn peek_precedence(&self) -> u8 {
        self.token_precedence(&self.current_token.kind)
    }

    fn current_precedence(&self) -> u8 {
        self.token_precedence(&self.current_token.kind)
    }

    fn token_precedence(&self, kind: &TokenKind) -> u8 {
        match kind {
            TokenKind::Dot => 8,
            TokenKind::LParen | TokenKind::LBracket => 7,
            TokenKind::DoubleStar => 6,
            TokenKind::Star | TokenKind::Slash | TokenKind::DoubleSlash | TokenKind::Percent => 5,
            TokenKind::Plus | TokenKind::Minus => 4,
            TokenKind::EqualEqual
            | TokenKind::NotEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => 3,
            TokenKind::And => 2,
            TokenKind::Or => 1,
            _ => 0,
        }
    }
}
