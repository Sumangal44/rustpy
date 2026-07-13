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
        let mut decorators = Vec::new();
        while self.check(&TokenKind::At) {
            self.advance()?;
            decorators.push(self.parse_expression(0)?);
            self.consume(TokenKind::Newline)?;
        }

        match &self.current_token.kind {
            TokenKind::Def => self.parse_function_def(decorators),
            TokenKind::Class => self.parse_class_def(decorators),
            TokenKind::Try => {
                if !decorators.is_empty() {
                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone()));
                }
                self.parse_try()
            },
            TokenKind::Raise => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_raise()
            },
            TokenKind::Return => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_return()
            },
            TokenKind::If => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_if()
            },
            TokenKind::While => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_while()
            },
            TokenKind::For => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_for()
            },
            TokenKind::With => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_with()
            },
            TokenKind::Pass => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.advance()?;
                self.consume(TokenKind::Newline)?;
                Ok(Stmt::Pass)
            }
            TokenKind::Break => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.advance()?;
                self.consume(TokenKind::Newline)?;
                Ok(Stmt::Break)
            }
            TokenKind::Continue => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.advance()?;
                self.consume(TokenKind::Newline)?;
                Ok(Stmt::Continue)
            }
            TokenKind::Del => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_del()
            }
            TokenKind::Global => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_global()
            }
            TokenKind::Nonlocal => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_nonlocal()
            }
            TokenKind::Assert => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_assert()
            }
            TokenKind::Yield => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.advance()?;
                let mut value = None;
                if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::EOF) {
                    value = Some(Box::new(self.parse_expression(0)?));
                }
                self.consume(TokenKind::Newline)?;
                Ok(Stmt::YieldStmt(Expr::Yield(value)))
            }
            _ => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_assign_or_expr()
            }
        }
    }

    fn parse_function_def(&mut self, decorators: Vec<Expr>) -> Result<Stmt, ParseError> {
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
        let mut vararg = None;
        let mut kwarg = None;

        if !self.check(&TokenKind::RParen) {
            loop {
                match &self.current_token.kind {
                    TokenKind::Star => {
                        self.advance()?;
                        if let TokenKind::Identifier(n) = &self.current_token.kind {
                            vararg = Some(n.clone());
                            self.advance()?;
                        } else {
                            return Err(ParseError::new(
                                ParseErrorKind::UnexpectedToken("Expected identifier after *".to_string()),
                                self.current_token.span.clone(),
                            ));
                        }
                    }
                    TokenKind::DoubleStar => {
                        self.advance()?;
                        if let TokenKind::Identifier(n) = &self.current_token.kind {
                            kwarg = Some(n.clone());
                            self.advance()?;
                        } else {
                            return Err(ParseError::new(
                                ParseErrorKind::UnexpectedToken("Expected identifier after **".to_string()),
                                self.current_token.span.clone(),
                            ));
                        }
                    }
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

        Ok(Stmt::FunctionDef {
            name,
            params,
            vararg,
            kwarg,
            body,
            decorators,
        })
    }

    fn parse_class_def(&mut self, decorators: Vec<Expr>) -> Result<Stmt, ParseError> {
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

        let mut bases = Vec::new();
        if self.check(&TokenKind::LParen) {
            self.advance()?;
            if !self.check(&TokenKind::RParen) {
                loop {
                    bases.push(self.parse_expression(0)?);
                    if self.check(&TokenKind::Comma) {
                        self.advance()?;
                    } else {
                        break;
                    }
                }
            }
            self.consume(TokenKind::RParen)?;
        }
        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;

        let body = self.parse_block()?;

        Ok(Stmt::ClassDef { name, bases, body, decorators })
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

    fn parse_with(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::With)?;
        let context_expr = self.parse_expression(0)?;
        let mut optional_vars = None;

        if self.check(&TokenKind::As) {
            self.advance()?;
            optional_vars = Some(self.parse_expression(0)?);
        }

        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;

        let body = self.parse_block()?;

        Ok(Stmt::With {
            context_expr,
            optional_vars,
            body,
        })
    }

    fn parse_try(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Try)?;
        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;

        let body = self.parse_block()?;

        let mut handlers = Vec::new();
        while self.check(&TokenKind::Except) {
            self.advance()?;

            let mut exc_type_name = None;
            if !self.check(&TokenKind::Colon) {
                let exc_expr = self.parse_expression(0)?;
                // Extract the type name if it's an identifier
                if let Expr::Identifier(name) = &exc_expr {
                    exc_type_name = Some(name.clone());
                }
                if self.check(&TokenKind::As) {
                    self.advance()?;
                    // Bind the exception to a variable - consume the identifier
                    if let TokenKind::Identifier(_) = &self.current_token.kind {
                        self.advance()?;
                    }
                }
            }

            self.consume(TokenKind::Colon)?;
            self.consume(TokenKind::Newline)?;

            let handler_body = self.parse_block()?;
            handlers.push((exc_type_name, handler_body));
        }

        let mut finally_body = None;
        if self.check(&TokenKind::Finally) {
            self.advance()?;
            self.consume(TokenKind::Colon)?;
            self.consume(TokenKind::Newline)?;
            finally_body = Some(self.parse_block()?);
        }

        Ok(Stmt::Try { body, handlers, finally_body })
    }

    fn parse_raise(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Raise)?;
        let exc = self.parse_expression(0)?;
        if self.check(&TokenKind::Newline) {
            self.advance()?;
        }
        Ok(Stmt::Raise { exc: Box::new(exc) })
    }

    fn parse_del(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Del)?;
        let target = self.parse_expression(0)?;
        self.consume(TokenKind::Newline)?;
        Ok(Stmt::Del { target: Box::new(target) })
    }

    fn parse_global(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Global)?;
        let mut names = Vec::new();
        loop {
            if let TokenKind::Identifier(n) = &self.current_token.kind {
                names.push(n.clone());
                self.advance()?;
            } else {
                return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Expected identifier".to_string()), self.current_token.span.clone()));
            }
            if self.check(&TokenKind::Comma) {
                self.advance()?;
            } else {
                break;
            }
        }
        self.consume(TokenKind::Newline)?;
        Ok(Stmt::Global { names })
    }

    fn parse_nonlocal(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Nonlocal)?;
        let mut names = Vec::new();
        loop {
            if let TokenKind::Identifier(n) = &self.current_token.kind {
                names.push(n.clone());
                self.advance()?;
            } else {
                return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Expected identifier".to_string()), self.current_token.span.clone()));
            }
            if self.check(&TokenKind::Comma) {
                self.advance()?;
            } else {
                break;
            }
        }
        self.consume(TokenKind::Newline)?;
        Ok(Stmt::Nonlocal { names })
    }

    fn parse_assert(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Assert)?;
        let test = self.parse_expression(0)?;
        let mut msg = None;
        if self.check(&TokenKind::Comma) {
            self.advance()?;
            msg = Some(Box::new(self.parse_expression(0)?));
        }
        self.consume(TokenKind::Newline)?;
        Ok(Stmt::Assert { test, msg })
    }

    fn parse_aug_op(&self) -> Option<BinOpKind> {
        match &self.current_token.kind {
            TokenKind::PlusEqual => Some(BinOpKind::Add),
            TokenKind::MinusEqual => Some(BinOpKind::Sub),
            TokenKind::StarEqual => Some(BinOpKind::Mult),
            TokenKind::SlashEqual => Some(BinOpKind::Div),
            TokenKind::DoubleSlashEqual => Some(BinOpKind::FloorDiv),
            TokenKind::PercentEqual => Some(BinOpKind::Mod),
            TokenKind::DoubleStarEqual => Some(BinOpKind::Pow),
            _ => None,
        }
    }

    fn parse_assign_or_expr(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expression(0)?;

        if let Some(op) = self.parse_aug_op() {
            self.advance()?;
            let value = self.parse_expression(0)?;
            self.consume(TokenKind::Newline)?;
            Ok(Stmt::AugAssign {
                target: Box::new(expr),
                op,
                value,
            })
        } else if self.check(&TokenKind::Equal) {
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
            TokenKind::Yield => {
                self.advance()?;
                let mut value = None;
                // If it's not a closing paren or newline, try parsing the yielded expression
                if !self.check(&TokenKind::RParen)
                    && !self.check(&TokenKind::Newline)
                    && !self.check(&TokenKind::EOF)
                {
                    value = Some(Box::new(self.parse_expression(0)?));
                }
                Ok(Expr::Yield(value))
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
        let mut kwargs = Vec::new();
        let mut starargs = Vec::new();
        let mut kwargs_unpack = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                if self.check(&TokenKind::Star) {
                    self.advance()?;
                    starargs.push(self.parse_expression(0)?);
                } else if self.check(&TokenKind::DoubleStar) {
                    self.advance()?;
                    kwargs_unpack.push(self.parse_expression(0)?);
                } else {
                    // Could be a keyword argument if it's an identifier followed by '='
                    // Wait, we need to peek ahead to see if the next token is '='
                    // Since we don't have peek(2), we can parse an expression.
                    // If it's an Identifier and the next token is '=', it's a kwarg!
                    // Actually, let's look at `self.current_token` and `self.peek_token` if we had it.
                    // If we just check current token is Identifier and next is Assign...
                    // Wait, we can just check if current token is Identifier.
                    // If it is, and the NEXT token is `=`, we parse it as a kwarg.
                    // But we don't have a peek() function easily available that returns the next token.
                    // Let's just clone the lexer state if we want to look ahead? No, our lexer yields tokens.
                    // We DO have `self.current_token` but no `self.peek_token()`.
                    // Let's just add a temporary hack: if `self.current_token` is Identifier, 
                    // we could just parse it as an expression. If it's an assignment, `parse_expression` 
                    // doesn't handle `=` (assignment is a statement).
                    // So if it's an identifier and the next token is `=`, we can't parse it as an expression easily.
                    // Let's implement a small peek by checking if it's an Identifier. But how to know if next is '='?
                    // We'll leave `kwargs` unimplemented in parsing for this exact step to keep it simple, or implement it by parsing an expression and checking if the next token is `=`. Wait! If we parse an expression, we consume the identifier. Then `self.current_token` would be `=`.
                    let expr = self.parse_expression(0)?;
                    if self.check(&TokenKind::Equal) {
                        self.advance()?;
                        let value = self.parse_expression(0)?;
                        if let Expr::Identifier(name) = expr {
                            kwargs.push((name, value));
                        } else {
                            return Err(ParseError::new(
                                ParseErrorKind::UnexpectedToken("Keyword argument must be an identifier".to_string()),
                                self.current_token.span.clone(),
                            ));
                        }
                    } else {
                        args.push(expr);
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

        Ok(Expr::Call {
            func: Box::new(func),
            args,
            kwargs,
            starargs,
            kwargs_unpack,
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
