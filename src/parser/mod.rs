use crate::ast::{BinOpKind, CompKind, Expr, MatchCase, Module, Pattern, Stmt, UnaryOpKind, WithItem};
use crate::diagnostics::{LexerError, ParseError, ParseErrorKind};
use crate::lexer::Lexer;
use crate::lexer::tokens::{Token, TokenKind};

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

    fn consume_stmt_end(&mut self) -> Result<(), ParseError> {
        if self.check(&TokenKind::Newline) {
            self.advance()?;
            Ok(())
        } else if self.check(&TokenKind::EOF) {
            Ok(())
        } else {
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken(format!("Expected newline, got {:?}", self.current_token.kind)),
                self.current_token.span.clone(),
            ))
        }
    }

    pub fn parse_module(&mut self) -> Result<Module, ParseError> {
        let mut body = Vec::new();

        self.consume_newlines()?;
        while !self.check(&TokenKind::EOF) {
            body.push(self.parse_statement()?);
            while self.check(&TokenKind::Semicolon) {
                self.advance()?;
                if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::EOF) {
                    body.push(self.parse_statement()?);
                }
            }
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

        // Handle 'async' as a statement prefix
        if self.check(&TokenKind::Async) {
            return self.parse_async_statement(decorators);
        }

        match &self.current_token.kind {
            TokenKind::Def => self.parse_function_def(decorators, false),
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
                self.consume_stmt_end()?;
                Ok(Stmt::Pass)
            }
            TokenKind::Break => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.advance()?;
                self.consume_stmt_end()?;
                Ok(Stmt::Break)
            }
            TokenKind::Continue => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.advance()?;
                self.consume_stmt_end()?;
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
            TokenKind::Match => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_match()
            }
            TokenKind::Import => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_import()
            }
            TokenKind::From => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_import_from()
            }
            TokenKind::Yield => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.advance()?;
                if self.check(&TokenKind::From) {
                    self.advance()?;
                    let value = Box::new(self.parse_expression(0)?);
                    self.consume_stmt_end()?;
                    return Ok(Stmt::YieldStmt(Expr::YieldFrom(value)));
                }
                let mut value = None;
                if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::EOF) {
                    value = Some(Box::new(self.parse_expression(0)?));
                }
                self.consume_stmt_end()?;
                Ok(Stmt::YieldStmt(Expr::Yield(value)))
            }
            _ => {
                if !decorators.is_empty() { return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Decorators not allowed here".to_string()), self.current_token.span.clone())); }
                self.parse_assign_or_expr()
            }
        }
    }

    fn parse_async_statement(&mut self, decorators: Vec<Expr>) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Async)?;
        match &self.current_token.kind {
            TokenKind::Def => self.parse_function_def(decorators, true),
            TokenKind::For => {
                let mut stmt = self.parse_for()?;
                if let Stmt::For { ref mut is_async, .. } = stmt {
                    *is_async = true;
                }
                Ok(stmt)
            }
            TokenKind::With => {
                let mut stmt = self.parse_with()?;
                if let Stmt::With { ref mut is_async, .. } = stmt {
                    *is_async = true;
                }
                Ok(stmt)
            }
            _ => Err(ParseError::new(
                ParseErrorKind::InvalidSyntax("expected 'def', 'for', or 'with' after 'async'".to_string()),
                self.current_token.span.clone(),
            )),
        }
    }

    fn parse_function_def(&mut self, decorators: Vec<Expr>, is_async: bool) -> Result<Stmt, ParseError> {
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
        let mut posonly_params = Vec::new();
        let mut params = Vec::new();
        let mut defaults = Vec::new();
        let mut vararg = None;
        let mut kwarg = None;
        let mut kwonly_params = Vec::new();
        let mut after_bare_star = false;

        if !self.check(&TokenKind::RParen) {
            loop {
                match &self.current_token.kind {
                    TokenKind::Slash => {
                        self.advance()?;
                        let mut posonly = Vec::new();
                        std::mem::swap(&mut params, &mut posonly);
                        posonly_params = posonly;
                    }
                    TokenKind::Star => {
                        self.advance()?;
                        if let TokenKind::Identifier(n) = &self.current_token.kind {
                            vararg = Some(n.clone());
                            self.advance()?;
                        }
                        after_bare_star = true;
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
                        if after_bare_star {
                            kwonly_params.push(n.clone());
                        } else {
                            params.push(n.clone());
                        }
                        self.advance()?;
                        if self.check(&TokenKind::Colon) {
                            self.advance()?;
                            self.parse_expression(0)?;
                        }
                        if self.check(&TokenKind::Equal) {
                            self.advance()?;
                            if after_bare_star {
                                // kwonly defaults tracked here but only `defaults` field exists
                            }
                            defaults.push(Some(self.parse_expression(0)?));
                        } else {
                            defaults.push(None);
                        }
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

        let mut returns = None;
        if self.check(&TokenKind::Arrow) {
            self.advance()?;
            returns = Some(Box::new(self.parse_expression(0)?));
        }

        self.consume(TokenKind::Colon)?;

        let body = self.parse_suite()?;

        Ok(Stmt::FunctionDef {
            name,
            posonly_params,
            params,
            kwonly_params,
            defaults,
            vararg,
            kwarg,
            body,
            decorators,
            is_async,
            returns,
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

        let body = self.parse_suite()?;

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
            while self.check(&TokenKind::Semicolon) {
                self.advance()?;
                if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::EOF) {
                    body.push(self.parse_statement()?);
                }
            }
        }

        self.consume(TokenKind::Dedent)?;
        Ok(body)
    }

    fn parse_suite(&mut self) -> Result<Vec<Stmt>, ParseError> {
        if self.check(&TokenKind::Newline) {
            self.advance()?;
            self.consume(TokenKind::Indent)?;
            let mut body = Vec::new();
            while !self.check(&TokenKind::Dedent) && !self.check(&TokenKind::EOF) {
                self.consume_newlines()?;
                if self.check(&TokenKind::Dedent) {
                    break;
                }
                body.push(self.parse_statement()?);
                while self.check(&TokenKind::Semicolon) {
                    self.advance()?;
                    if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::EOF) {
                        body.push(self.parse_statement()?);
                    }
                }
            }
            self.consume(TokenKind::Dedent)?;
            Ok(body)
        } else {
            let mut body = Vec::new();
            loop {
                body.push(self.parse_statement()?);
                if self.check(&TokenKind::Semicolon) {
                    self.advance()?;
                    if self.check(&TokenKind::Newline) || self.check(&TokenKind::EOF) {
                        if self.check(&TokenKind::Newline) {
                            self.advance()?;
                        }
                        break;
                    }
                } else {
                    break;
                }
            }
            Ok(body)
        }
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Return)?;

        let value = if self.check(&TokenKind::Newline) || self.check(&TokenKind::EOF) {
            None
        } else {
            Some(self.parse_expression_list()?)
        };

        if self.check(&TokenKind::Newline) {
            self.advance()?;
        }

        Ok(Stmt::Return { value })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        if self.check(&TokenKind::If) {
            self.advance()?;
        } else if self.check(&TokenKind::Elif) {
            self.advance()?;
        }
        let test = self.parse_expression(0)?;
        self.consume(TokenKind::Colon)?;

        let body = self.parse_suite()?;
        let mut orelse = Vec::new();

        if self.check(&TokenKind::Else) {
            self.advance()?;
            self.consume(TokenKind::Colon)?;
            orelse = self.parse_suite()?;
        } else if self.check(&TokenKind::Elif) {
            let elif_stmt = self.parse_if()?;
            orelse.push(elif_stmt);
        }

        Ok(Stmt::If { test, body, orelse })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::While)?;
        let test = self.parse_expression(0)?;
        self.consume(TokenKind::Colon)?;

        let body = self.parse_suite()?;
        let mut orelse = Vec::new();
        if self.check(&TokenKind::Else) {
            self.advance()?;
            self.consume(TokenKind::Colon)?;
            orelse = self.parse_suite()?;
        }

        Ok(Stmt::While { test, body, orelse })
    }

    fn parse_for_target(&mut self) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::LParen) {
            self.advance()?;
            if self.check(&TokenKind::RParen) {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidSyntax("Empty tuple not allowed in for loop target".to_string()),
                    self.current_token.span.clone(),
                ));
            }
            let mut elements = Vec::new();
            loop {
                let expr = self.parse_for_target()?;
                elements.push(expr);
                if self.check(&TokenKind::Comma) {
                    self.advance()?;
                    if self.check(&TokenKind::RParen) {
                        break;
                    }
                } else if self.check(&TokenKind::RParen) {
                    break;
                } else {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedToken("Expected ',' or ')' in tuple target".to_string()),
                        self.current_token.span.clone(),
                    ));
                }
            }
            self.consume(TokenKind::RParen)?;
            if elements.len() == 1 {
                return Ok(elements.into_iter().next().unwrap());
            }
            return Ok(Expr::Tuple(elements));
        }
        match &self.current_token.kind {
            TokenKind::Identifier(name) => {
                let first = Expr::Identifier(name.clone());
                self.advance()?;
                if self.check(&TokenKind::Comma) {
                    let mut elements = vec![first];
                    while self.check(&TokenKind::Comma) {
                        self.advance()?;
                        elements.push(self.parse_for_target()?);
                    }
                    return Ok(Expr::Tuple(elements));
                }
                Ok(first)
            }
            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken("Expected identifier in for loop target".to_string()),
                self.current_token.span.clone(),
            )),
        }
    }

    fn parse_comp_ifs(&mut self) -> Vec<Expr> {
        let mut ifs = Vec::new();
        while self.check(&TokenKind::If) {
            self.advance().unwrap(); // consume 'if'
            ifs.push(self.parse_expression(0).unwrap());
        }
        ifs
    }

    fn parse_list_comp(&mut self, elt: Expr) -> Result<Expr, ParseError> {
        self.consume(TokenKind::For)?;
        let target = self.parse_for_target()?;
        self.consume(TokenKind::In)?;
        let iter = self.parse_expression_opts(0, false)?;
        let ifs = self.parse_comp_ifs();
        self.consume(TokenKind::RBracket)?;
        Ok(Expr::Comprehension {
            kind: crate::ast::CompKind::List,
            elt: Box::new(elt),
            key: None,
            target: Box::new(target),
            iter: Box::new(iter),
            ifs,
        })
    }

    fn parse_dict_comp(&mut self, key: Expr, elt: Expr) -> Result<Expr, ParseError> {
        self.consume(TokenKind::For)?;
        let target = self.parse_for_target()?;
        self.consume(TokenKind::In)?;
        let iter = self.parse_expression_opts(0, false)?;
        let ifs = self.parse_comp_ifs();
        self.consume(TokenKind::RBrace)?;
        Ok(Expr::Comprehension {
            kind: crate::ast::CompKind::Dict,
            elt: Box::new(elt),
            key: Some(Box::new(key)),
            target: Box::new(target),
            iter: Box::new(iter),
            ifs,
        })
    }

    fn parse_set_comp(&mut self, elt: Expr) -> Result<Expr, ParseError> {
        self.consume(TokenKind::For)?;
        let target = self.parse_for_target()?;
        self.consume(TokenKind::In)?;
        let iter = self.parse_expression_opts(0, false)?;
        let ifs = self.parse_comp_ifs();
        self.consume(TokenKind::RBrace)?;
        Ok(Expr::Comprehension {
            kind: crate::ast::CompKind::Set,
            elt: Box::new(elt),
            key: None,
            target: Box::new(target),
            iter: Box::new(iter),
            ifs,
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::For)?;
        let target = self.parse_for_target()?;
        self.consume(TokenKind::In)?;
        let iter = self.parse_expression(0)?;
        self.consume(TokenKind::Colon)?;

        let body = self.parse_suite()?;
        let mut orelse = Vec::new();
        if self.check(&TokenKind::Else) {
            self.advance()?;
            self.consume(TokenKind::Colon)?;
            orelse = self.parse_suite()?;
        }

        Ok(Stmt::For { target, iter, body, orelse, is_async: false })
    }

    fn parse_with(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::With)?;
        let mut items = Vec::new();
        loop {
            let context_expr = self.parse_expression(0)?;
            let mut optional_vars = None;
            if self.check(&TokenKind::As) {
                self.advance()?;
                optional_vars = Some(self.parse_expression(0)?);
            }
            items.push(WithItem { context_expr, optional_vars });
            if self.check(&TokenKind::Comma) {
                self.advance()?;
            } else {
                break;
            }
        }
        self.consume(TokenKind::Colon)?;

        let body = self.parse_suite()?;

        Ok(Stmt::With {
            items,
            body,
            is_async: false,
        })
    }

    fn parse_try(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Try)?;
        self.consume(TokenKind::Colon)?;

        let body = self.parse_suite()?;

        let mut handlers = Vec::new();
        while self.check(&TokenKind::Except) {
            self.advance()?;

            let mut type_names: Vec<String> = Vec::new();
            let mut as_name = None;
            if !self.check(&TokenKind::Colon) {
                let exc_expr = self.parse_expression(0)?;
                match &exc_expr {
                    Expr::Identifier(name) => {
                        type_names.push(name.clone());
                    }
                    Expr::Tuple(elts) => {
                        for elt in elts {
                            if let Expr::Identifier(name) = elt {
                                type_names.push(name.clone());
                            }
                        }
                    }
                    _ => {}
                }
                if self.check(&TokenKind::As) {
                    self.advance()?;
                    if let TokenKind::Identifier(name) = &self.current_token.kind {
                        as_name = Some(name.clone());
                        self.advance()?;
                    }
                }
            }

            self.consume(TokenKind::Colon)?;

            let handler_body = self.parse_suite()?;
            if type_names.is_empty() {
                handlers.push((None, as_name, handler_body));
            } else {
                for name in &type_names {
                    handlers.push((Some(name.clone()), as_name.clone(), handler_body.clone()));
                }
            }
        }

        let mut else_body = None;
        if self.check(&TokenKind::Else) {
            self.advance()?;
            self.consume(TokenKind::Colon)?;
            else_body = Some(self.parse_suite()?);
        }

        let mut finally_body = None;
        if self.check(&TokenKind::Finally) {
            self.advance()?;
            self.consume(TokenKind::Colon)?;
            finally_body = Some(self.parse_suite()?);
        }

        Ok(Stmt::Try { body, handlers, else_body, finally_body })
    }

    fn parse_raise(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Raise)?;
        let exc = if self.check(&TokenKind::Newline) || self.check(&TokenKind::EOF) {
            None
        } else {
            Some(Box::new(self.parse_expression(0)?))
        };
        let mut cause = None;
        if self.check(&TokenKind::From) {
            self.advance()?;
            cause = Some(Box::new(self.parse_expression(0)?));
        }
        self.consume_stmt_end()?;
        Ok(Stmt::Raise { exc, cause })
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
        self.consume_stmt_end()?;
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
        self.consume_stmt_end()?;
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
        self.consume_stmt_end()?;
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
            TokenKind::AmpersandEqual => Some(BinOpKind::BitAnd),
            TokenKind::PipeEqual => Some(BinOpKind::BitOr),
            TokenKind::CaretEqual => Some(BinOpKind::BitXor),
            TokenKind::LeftShiftEqual => Some(BinOpKind::LShift),
            TokenKind::RightShiftEqual => Some(BinOpKind::RShift),
            _ => None,
        }
    }

    fn parse_assign_or_expr(&mut self) -> Result<Stmt, ParseError> {
        let first = self.parse_expression(0)?;
        let mut targets = vec![first];

        while self.check(&TokenKind::Comma) {
            self.advance()?;
            if self.check(&TokenKind::Equal) {
                // a, = ...  single-element unpack target list ends at =
                break;
            }
            targets.push(self.parse_expression(0)?);
        }

        if let Some(op) = self.parse_aug_op() {
            self.advance()?;
            let value = self.parse_expression(0)?;
            if self.check(&TokenKind::Newline) {
                self.advance()?;
            }
            if targets.len() != 1 {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidSyntax("augmented assignment with multiple targets".to_string()),
                    self.current_token.span.clone(),
                ));
            }
            Ok(Stmt::AugAssign {
                target: Box::new(targets.into_iter().next().unwrap()),
                op,
                value,
            })
        } else if self.check(&TokenKind::Equal) {
            self.advance()?;
            let value = self.parse_expression_list()?;
            if self.check(&TokenKind::Newline) {
                self.advance()?;
            }
            Ok(Stmt::Assign { targets, value })
        } else {
            if self.check(&TokenKind::Newline) {
                self.advance()?;
            }
            if targets.len() > 1 {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidSyntax("cannot assign to tuple literal (use parentheses for tuple expressions)".to_string()),
                    self.current_token.span.clone(),
                ));
            }
            Ok(Stmt::ExprStmt { value: targets.into_iter().next().unwrap() })
        }
    }

    pub fn parse_expression_list(&mut self) -> Result<Expr, ParseError> {
        let first = self.parse_expression(0)?;
        if self.check(&TokenKind::Comma) {
            let mut elements = vec![first];
            while self.check(&TokenKind::Comma) {
                self.advance()?;
                if self.check(&TokenKind::Newline) || self.check(&TokenKind::EOF) || self.check(&TokenKind::Equal) || self.check(&TokenKind::RParen) || self.check(&TokenKind::RBracket) || self.check(&TokenKind::RBrace) || self.check(&TokenKind::Colon) {
                    break;
                }
                elements.push(self.parse_expression(0)?);
            }
            Ok(Expr::Tuple(elements))
        } else {
            Ok(first)
        }
    }

    pub fn parse_expression(&mut self, precedence: u8) -> Result<Expr, ParseError> {
        self.parse_expression_opts(precedence, true)
    }

    fn parse_expression_opts(&mut self, precedence: u8, allow_ternary: bool) -> Result<Expr, ParseError> {
        let mut left = self.parse_prefix()?;

        while precedence < self.peek_precedence() {
            left = self.parse_infix(left)?;
        }

        // Walrus operator := has lowest precedence, only parsed at top level
        if precedence == 0 && self.check(&TokenKind::ColonEqual) {
            self.advance()?;
            let value = self.parse_expression(0)?;
            if let Expr::Identifier(_) = &left {
                left = Expr::NamedExpr {
                    target: Box::new(left),
                    value: Box::new(value),
                };
            } else {
                return Err(ParseError::new(
                    ParseErrorKind::InvalidSyntax("Assignment target must be an identifier".to_string()),
                    self.current_token.span.clone(),
                ));
            }
        }

        // Ternary/conditional expression: body if test else orelse
        if allow_ternary && precedence == 0 && self.check(&TokenKind::If) {
            let body = Box::new(left);
            self.advance()?;
            let test = self.parse_expression(0)?;
            self.consume(TokenKind::Else)?;
            let orelse = self.parse_expression(0)?;
            left = Expr::IfExp {
                test: Box::new(test),
                body,
                orelse: Box::new(orelse),
            };
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
                let expr = Expr::IntLiteral(val.clone());
                self.advance()?;
                Ok(expr)
            }
            TokenKind::FloatLiteral(val) => {
                let expr = Expr::FloatLiteral(*val);
                self.advance()?;
                Ok(expr)
            }
            TokenKind::ImagLiteral(val) => {
                let expr = Expr::ImagLiteral(*val);
                self.advance()?;
                Ok(expr)
            }
            TokenKind::StringLiteral(val) => {
                let expr = Expr::StringLiteral(val.clone());
                self.advance()?;
                Ok(expr)
            }
            TokenKind::BytesLiteral(val) => {
                let expr = Expr::BytesLiteral(val.clone());
                self.advance()?;
                Ok(expr)
            }
            TokenKind::FStringLiteral(content) => {
                let content_clone = content.clone();
                self.advance()?;
                self.parse_fstring_content(&content_clone)
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
            TokenKind::Ellipsis => {
                self.advance()?;
                Ok(Expr::Ellipsis)
            }
            TokenKind::Star => {
                self.advance()?;
                let value = Box::new(self.parse_expression(0)?);
                Ok(Expr::Starred { value })
            }
            TokenKind::LParen => {
                self.advance()?;
                if self.check(&TokenKind::RParen) {
                    self.advance()?;
                    return Ok(Expr::Tuple(Vec::new()));
                }
                let first = self.parse_expression(0)?;
                // Check for generator expression: (expr for target in iter)
                if self.check(&TokenKind::For) {
                    return self.parse_generator_expr(first);
                }
                if self.check(&TokenKind::Comma) {
                    let mut elements = vec![first];
                    while self.check(&TokenKind::Comma) {
                        self.advance()?;
                        if self.check(&TokenKind::RParen) {
                            break;
                        }
                        elements.push(self.parse_expression(0)?);
                    }
                    self.consume(TokenKind::RParen)?;
                    return Ok(Expr::Tuple(elements));
                }
                self.consume(TokenKind::RParen)?;
                Ok(first)
            }
            TokenKind::Await => {
                self.advance()?;
                let expr = self.parse_expression(0)?;
                Ok(Expr::Await(Box::new(expr)))
            }
            TokenKind::Yield => {
                self.advance()?;
                if self.check(&TokenKind::From) {
                    self.advance()?;
                    let value = Box::new(self.parse_expression(0)?);
                    return Ok(Expr::YieldFrom(value));
                }
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
                if self.check(&TokenKind::RBracket) {
                    self.advance()?;
                    return Ok(Expr::List(Vec::new()));
                }
                let first = self.parse_expression(0)?;
                if self.check(&TokenKind::For) {
                    return self.parse_list_comp(first);
                }
                let mut elements = vec![first];
                while self.check(&TokenKind::Comma) {
                    self.advance()?;
                    if self.check(&TokenKind::RBracket) {
                        break;
                    }
                    elements.push(self.parse_expression(0)?);
                }
                self.consume(TokenKind::RBracket)?;
                Ok(Expr::List(elements))
            }
            TokenKind::LBrace => {
                self.advance()?;
                if self.check(&TokenKind::RBrace) {
                    self.advance()?;
                    return Ok(Expr::Dict(Vec::new()));
                }
                let first = self.parse_expression(0)?;
                if self.check(&TokenKind::For) {
                    return self.parse_set_comp(first);
                }
                if self.check(&TokenKind::Colon) {
                    self.advance()?;
                    let second = self.parse_expression(0)?;
                    if self.check(&TokenKind::For) {
                        return self.parse_dict_comp(first, second);
                    }
                    let mut pairs = vec![(first, second)];
                    while self.check(&TokenKind::Comma) {
                        self.advance()?;
                        if self.check(&TokenKind::RBrace) {
                            break;
                        }
                        let key = self.parse_expression(0)?;
                        self.consume(TokenKind::Colon)?;
                        let value = self.parse_expression(0)?;
                        pairs.push((key, value));
                    }
                    self.consume(TokenKind::RBrace)?;
                    Ok(Expr::Dict(pairs))
                } else {
                    let mut elements = vec![first];
                    while self.check(&TokenKind::Comma) {
                        self.advance()?;
                        if self.check(&TokenKind::RBrace) {
                            break;
                        }
                        elements.push(self.parse_expression(0)?);
                    }
                    self.consume(TokenKind::RBrace)?;
                    Ok(Expr::Set(elements))
                }
            }
            TokenKind::Lambda => {
                self.advance()?;
                let mut posonly_params = Vec::new();
                let mut params = Vec::new();
                let mut vararg = None;
                let mut kwarg = None;
                let mut kwonly_params = Vec::new();
                let mut after_bare_star = false;
                if !self.check(&TokenKind::Colon) {
                    loop {
                        match &self.current_token.kind {
                            TokenKind::Slash => {
                                self.advance()?;
                                let mut posonly = Vec::new();
                                std::mem::swap(&mut params, &mut posonly);
                                posonly_params = posonly;
                            }
                            TokenKind::Star => {
                                self.advance()?;
                                if let TokenKind::Identifier(name) = &self.current_token.kind {
                                    vararg = Some(name.clone());
                                    self.advance()?;
                                }
                                after_bare_star = true;
                            }
                            TokenKind::DoubleStar => {
                                self.advance()?;
                                if let TokenKind::Identifier(name) = &self.current_token.kind {
                                    kwarg = Some(name.clone());
                                    self.advance()?;
                                } else {
                                    return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Expected parameter name after **".to_string()), self.current_token.span.clone()));
                                }
                            }
                            TokenKind::Identifier(name) => {
                                if after_bare_star {
                                    kwonly_params.push(name.clone());
                                } else {
                                    params.push(name.clone());
                                }
                                self.advance()?;
                            }
                            _ => {
                                return Err(ParseError::new(ParseErrorKind::UnexpectedToken("Expected parameter name".to_string()), self.current_token.span.clone()));
                            }
                        }
                        if self.check(&TokenKind::Comma) {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::Colon)?;
                let body = self.parse_expression(0)?;
                Ok(Expr::Lambda {
                    params,
                    posonly_params,
                    kwonly_params,
                    vararg,
                    kwarg,
                    body: Box::new(body),
                })
            }
            TokenKind::Minus | TokenKind::Plus => {
                let op = match self.current_token.kind {
                    TokenKind::Minus => UnaryOpKind::Minus,
                    TokenKind::Plus => UnaryOpKind::Plus,
                    _ => unreachable!(),
                };
                self.advance()?;
                let operand = Box::new(self.parse_expression(6)?); // Unary precedence (tighter than **)
                Ok(Expr::UnaryOp { op, operand })
            }
            TokenKind::Not => {
                let op = UnaryOpKind::Not;
                self.advance()?;
                let operand = Box::new(self.parse_expression(3)?); // Looser than comparisons
                Ok(Expr::UnaryOp { op, operand })
            }
            TokenKind::Tilde => {
                self.advance()?;
                let operand = Box::new(self.parse_expression(6)?);
                Ok(Expr::UnaryOp { op: UnaryOpKind::Invert, operand })
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

    fn parse_fstring_content(&self, content: &str) -> Result<Expr, ParseError> {
        use crate::ast::FStringSegment;

        let mut segments = Vec::new();
        let mut literal = String::new();
        let mut chars = content.char_indices().peekable();
        let mut escape = false;

        while let Some((_, c)) = chars.next() {
            if escape {
                match c {
                    'n' => literal.push('\n'),
                    't' => literal.push('\t'),
                    'r' => literal.push('\r'),
                    '\\' => literal.push('\\'),
                    '{' => literal.push('{'),
                    '}' => literal.push('}'),
                    '\'' => literal.push('\''),
                    '"' => literal.push('"'),
                    _ => { literal.push('\\'); literal.push(c); }
                }
                escape = false;
                continue;
            }
            if c == '\\' {
                escape = true;
                continue;
            }
            if c == '{' {
                if chars.peek().map(|(_, c)| *c) == Some('{') {
                    chars.next();
                    literal.push('{');
                    continue;
                }
                if !literal.is_empty() {
                    segments.push(FStringSegment::Text(literal.clone()));
                    literal.clear();
                }
                let mut expr_text = String::new();
                let mut depth = 1;
                let mut in_spec = false;
                let mut spec_text = String::new();
                while let Some((_, ec)) = chars.next() {
                    if in_spec {
                        if ec == '{' { depth += 1; spec_text.push('{'); }
                        else if ec == '}' { depth -= 1; if depth == 0 { break; } if depth > 0 { spec_text.push('}'); } }
                        else { spec_text.push(ec); }
                    } else if ec == ':' && depth == 1 {
                        in_spec = true;
                    } else {
                        if ec == '{' { depth += 1; expr_text.push('{'); }
                        else if ec == '}' { depth -= 1; if depth == 0 { break; } expr_text.push('}'); }
                        else { expr_text.push(ec); }
                    }
                }
                if depth != 0 {
                    return Err(ParseError::new(
                        ParseErrorKind::InvalidSyntax("Unclosed '{' in f-string expression".to_string()),
                        self.current_token.span.clone(),
                    ));
                }
                let spec = if spec_text.is_empty() { None } else { Some(spec_text) };
                // Handle debug syntax {expr=}
                let sub_expr_text = if expr_text.ends_with('=') {
                    let debug_name = expr_text.trim_end_matches('=');
                    let name_val = if debug_name.is_empty() { String::new() } else { debug_name.to_string() };
                    segments.push(FStringSegment::Text(format!("{}=", name_val)));
                    name_val.to_string()
                } else {
                    expr_text
                };
                let sub_expr = self.parse_fstring_expr(&sub_expr_text)?;
                segments.push(FStringSegment::Expr { expr: Box::new(sub_expr), format_spec: spec });
                continue;
            }
            if c == '}' {
                if chars.peek().map(|(_, c)| *c) == Some('}') {
                    chars.next();
                    literal.push('}');
                    continue;
                }
                return Err(ParseError::new(
                    ParseErrorKind::InvalidSyntax("Single '}' in f-string; must escape as '}}'".to_string()),
                    self.current_token.span.clone(),
                ));
            }
            literal.push(c);
        }

        if escape {
            return Err(ParseError::new(
                ParseErrorKind::InvalidSyntax("Unterminated escape sequence in f-string".to_string()),
                self.current_token.span.clone(),
            ));
        }

        if !literal.is_empty() {
            segments.push(FStringSegment::Text(literal));
        }

        Ok(Expr::FString(segments))
    }

    fn parse_fstring_expr(&self, text: &str) -> Result<Expr, ParseError> {
        let lexer = crate::lexer::Lexer::new(text);
        let mut sub_parser = Parser::new(lexer)?;
        let expr = sub_parser.parse_expression(0)?;
        if !sub_parser.check(&TokenKind::EOF) {
            return Err(ParseError::new(
                ParseErrorKind::InvalidSyntax("Extra tokens in f-string expression".to_string()),
                self.current_token.span.clone(),
            ));
        }
        Ok(expr)
    }

    fn parse_generator_expr(&mut self, elt: Expr) -> Result<Expr, ParseError> {
        self.consume(TokenKind::For)?;
        let target = self.parse_for_target()?;
        self.consume(TokenKind::In)?;
        let iter = self.parse_expression_opts(0, false)?;
        let ifs = self.parse_comp_ifs();
        self.consume(TokenKind::RParen)?;
        Ok(Expr::Comprehension {
            kind: CompKind::Generator,
            elt: Box::new(elt),
            key: None,
            target: Box::new(target),
            iter: Box::new(iter),
            ifs,
        })
    }

    fn parse_match(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Match)?;
        let subject = self.parse_expression(0)?;
        self.consume(TokenKind::Colon)?;
        self.consume(TokenKind::Newline)?;
        self.consume(TokenKind::Indent)?;

        let mut cases = Vec::new();
        while self.check(&TokenKind::Case) {
            cases.push(self.parse_case()?);
        }

        self.consume(TokenKind::Dedent)?;
        Ok(Stmt::Match {
            subject: Box::new(subject),
            cases,
        })
    }

    fn parse_case(&mut self) -> Result<MatchCase, ParseError> {
        self.consume(TokenKind::Case)?;
        let pattern = self.parse_pattern()?;
        let guard = if self.check(&TokenKind::If) {
            self.advance()?;
            Some(Box::new(self.parse_expression(0)?))
        } else {
            None
        };
        self.consume(TokenKind::Colon)?;
        let body = self.parse_suite()?;
        Ok(MatchCase { pattern, guard, body })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let mut patterns = vec![self.parse_single_pattern()?];
        while self.check(&TokenKind::Pipe) {
            self.advance()?;
            patterns.push(self.parse_single_pattern()?);
        }
        if patterns.len() == 1 {
            Ok(patterns.into_iter().next().unwrap())
        } else {
            Ok(Pattern::Or(patterns))
        }
    }

    fn parse_single_pattern(&mut self) -> Result<Pattern, ParseError> {
        match &self.current_token.kind {
            TokenKind::IntLiteral(val) => {
                let expr = Expr::IntLiteral(val.clone());
                self.advance()?;
                Ok(Pattern::Literal(Box::new(expr)))
            }
            TokenKind::FloatLiteral(val) => {
                let expr = Expr::FloatLiteral(*val);
                self.advance()?;
                Ok(Pattern::Literal(Box::new(expr)))
            }
            TokenKind::StringLiteral(val) => {
                let expr = Expr::StringLiteral(val.clone());
                self.advance()?;
                Ok(Pattern::Literal(Box::new(expr)))
            }
            TokenKind::BytesLiteral(val) => {
                let expr = Expr::BytesLiteral(val.clone());
                self.advance()?;
                Ok(Pattern::Literal(Box::new(expr)))
            }
            TokenKind::True => {
                self.advance()?;
                Ok(Pattern::Literal(Box::new(Expr::BooleanLiteral(true))))
            }
            TokenKind::False => {
                self.advance()?;
                Ok(Pattern::Literal(Box::new(Expr::BooleanLiteral(false))))
            }
            TokenKind::None => {
                self.advance()?;
                Ok(Pattern::Literal(Box::new(Expr::NoneLiteral)))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance()?;
                if self.check(&TokenKind::LParen) {
                    self.advance()?;
                    let mut pos_args = Vec::new();
                    let mut kw_args: Vec<(String, Pattern)> = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            // Check if this is a keyword pattern: identifier = pattern
                            let is_kw = match &self.current_token.kind {
                                TokenKind::Identifier(_) => self.peek_token.kind == TokenKind::Equal,
                                _ => false,
                            };
                            if is_kw {
                                // keyword pattern: name=subpattern
                                let kw_name = match &self.current_token.kind {
                                    TokenKind::Identifier(n) => n.clone(),
                                    _ => unreachable!(),
                                };
                                self.advance()?; // consume identifier
                                self.consume(TokenKind::Equal)?; // consume '='
                                let subpat = self.parse_pattern()?;
                                kw_args.push((kw_name, subpat));
                            } else {
                                pos_args.push(self.parse_pattern()?);
                            }
                            if self.check(&TokenKind::Comma) {
                                self.advance()?;
                            } else {
                                break;
                            }
                        }
                    }
                    self.consume(TokenKind::RParen)?;
                    Ok(Pattern::Class(name, pos_args, kw_args))
                } else {
                    Ok(Pattern::Capture(name))
                }
            }
            TokenKind::LBrace => {
                self.advance()?;
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBrace) {
                    loop {
                        let key = self.parse_expression(0)?;
                        self.consume(TokenKind::Colon)?;
                        let val = self.parse_pattern()?;
                        elements.push((key, val));
                        if self.check(&TokenKind::Comma) {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RBrace)?;
                Ok(Pattern::Mapping(elements))
            }
            TokenKind::LBracket => {
                self.advance()?;
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    loop {
                        elements.push(self.parse_pattern()?);
                        if self.check(&TokenKind::Comma) {
                            self.advance()?;
                        } else {
                            break;
                        }
                    }
                }
                self.consume(TokenKind::RBracket)?;
                Ok(Pattern::Sequence(elements))
            }
            _ => Err(ParseError::new(
                ParseErrorKind::InvalidSyntax(format!("Expected pattern, got {:?}", self.current_token.kind)),
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

        let op = match &self.current_token.kind {
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
            TokenKind::In => BinOpKind::In,
            TokenKind::Is => {
                if self.check_peek(&TokenKind::Not) {
                    self.advance()?;
                    BinOpKind::IsNot
                } else {
                    BinOpKind::Is
                }
            }
            TokenKind::Not => {
                if self.check_peek(&TokenKind::In) {
                    self.advance()?;
                    BinOpKind::NotIn
                } else {
                    return Err(ParseError::new(
                        ParseErrorKind::InvalidSyntax("expected 'in' after 'not'".to_string()),
                        self.current_token.span.clone(),
                    ));
                }
            }
            TokenKind::And => BinOpKind::And,
            TokenKind::Or => BinOpKind::Or,
            TokenKind::At => BinOpKind::MatMul,
            TokenKind::LeftShift => BinOpKind::LShift,
            TokenKind::RightShift => BinOpKind::RShift,
            TokenKind::Ampersand => BinOpKind::BitAnd,
            TokenKind::Pipe => BinOpKind::BitOr,
            TokenKind::Caret => BinOpKind::BitXor,
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
                    } else if self.check(&TokenKind::For) {
                        // Generator expression as function argument
                        self.consume(TokenKind::For)?;
                        let target = self.parse_for_target()?;
                        self.consume(TokenKind::In)?;
                        let iter = self.parse_expression(0)?;
                        let ifs = self.parse_comp_ifs();
                        let gen_expr = Expr::Comprehension {
                            kind: CompKind::Generator,
                            elt: Box::new(expr),
                            key: None,
                            target: Box::new(target),
                            iter: Box::new(iter),
                            ifs,
                        };
                        args.push(gen_expr);
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
        if self.check(&TokenKind::Colon) {
            self.advance()?;
            let (stop, step) = self.parse_slice_tail()?;
            self.consume(TokenKind::RBracket)?;
            return Ok(Expr::Slice {
                value: Box::new(value),
                start: None,
                stop,
                step,
            });
        }
        let first = self.parse_expression(0)?;
        if self.check(&TokenKind::Colon) {
            self.advance()?;
            let (stop, step) = self.parse_slice_tail()?;
            self.consume(TokenKind::RBracket)?;
            Ok(Expr::Slice {
                value: Box::new(value),
                start: Some(Box::new(first)),
                stop,
                step,
            })
        } else {
            self.consume(TokenKind::RBracket)?;
            Ok(Expr::Subscript {
                value: Box::new(value),
                slice: Box::new(first),
            })
        }
    }

    fn parse_slice_tail(&mut self) -> Result<(Option<Box<Expr>>, Option<Box<Expr>>), ParseError> {
        // After seeing first ':', parse stop (optional) and step (optional after second ':')
        let stop = if self.check(&TokenKind::Colon) || self.check(&TokenKind::RBracket) {
            None
        } else {
            Some(Box::new(self.parse_expression(0)?))
        };
        let step = if self.check(&TokenKind::Colon) {
            self.advance()?;
            if self.check(&TokenKind::RBracket) {
                None
            } else {
                Some(Box::new(self.parse_expression(0)?))
            }
        } else {
            None
        };
        Ok((stop, step))
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

    fn parse_import(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::Import)?;
        let mut names = Vec::new();
        loop {
            let name = match &self.current_token.kind {
                TokenKind::Identifier(n) => {
                    let mut full_name = n.clone();
                    self.advance()?;
                    while self.check(&TokenKind::Dot) {
                        self.advance()?;
                        full_name.push('.');
                        if let TokenKind::Identifier(part) = &self.current_token.kind {
                            full_name.push_str(part);
                            self.advance()?;
                        } else {
                            return Err(ParseError::new(
                                ParseErrorKind::UnexpectedToken("Expected identifier after '.'".to_string()),
                                self.current_token.span.clone(),
                            ));
                        }
                    }
                    full_name
                }
                _ => {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedToken("Expected module name".to_string()),
                        self.current_token.span.clone(),
                    ));
                }
            };
            let mut asname = None;
            if self.check(&TokenKind::As) {
                self.advance()?;
                if let TokenKind::Identifier(a) = &self.current_token.kind {
                    asname = Some(a.clone());
                    self.advance()?;
                } else {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedToken("Expected identifier after 'as'".to_string()),
                        self.current_token.span.clone(),
                    ));
                }
            }
            names.push(crate::ast::Alias { name, asname });
            if self.check(&TokenKind::Comma) {
                self.advance()?;
            } else {
                break;
            }
        }
        self.consume_stmt_end()?;
        Ok(Stmt::Import { names })
    }

    fn parse_import_from(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenKind::From)?;
        let mut level = 0;
        while self.check(&TokenKind::Dot) {
            level += 1;
            self.advance()?;
        }
        let mut module = String::new();
        // Parse module name (possibly dotted)
        if let TokenKind::Identifier(n) = &self.current_token.kind {
            module = n.clone();
            self.advance()?;
            while self.check(&TokenKind::Dot) {
                self.advance()?;
                module.push('.');
                if let TokenKind::Identifier(part) = &self.current_token.kind {
                    module.push_str(part);
                    self.advance()?;
                } else {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedToken("Expected identifier after '.'".to_string()),
                        self.current_token.span.clone(),
                    ));
                }
            }
        }

        self.consume(TokenKind::Import)?;

        let mut names = Vec::new();
        if self.check(&TokenKind::Star) {
            self.advance()?;
            names.push(crate::ast::Alias {
                name: "*".to_string(),
                asname: None,
            });
        } else {
            loop {
                if let TokenKind::Identifier(n) = &self.current_token.kind {
                    let name = n.clone();
                    self.advance()?;
                    let mut asname = None;
                    if self.check(&TokenKind::As) {
                        self.advance()?;
                        if let TokenKind::Identifier(a) = &self.current_token.kind {
                            asname = Some(a.clone());
                            self.advance()?;
                        } else {
                            return Err(ParseError::new(
                                ParseErrorKind::UnexpectedToken("Expected identifier after 'as'".to_string()),
                                self.current_token.span.clone(),
                            ));
                        }
                    }
                    names.push(crate::ast::Alias { name, asname });
                    if self.check(&TokenKind::Comma) {
                        self.advance()?;
                    } else {
                        break;
                    }
                } else {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedToken("Expected identifier".to_string()),
                        self.current_token.span.clone(),
                    ));
                }
            }
        }
        self.consume(TokenKind::Newline)?;
        Ok(Stmt::ImportFrom {
            module,
            names,
            level,
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
            TokenKind::Star | TokenKind::Slash | TokenKind::DoubleSlash | TokenKind::Percent | TokenKind::At => 5,
            TokenKind::Plus | TokenKind::Minus | TokenKind::LeftShift | TokenKind::RightShift => 4,
            // Comparisons: ==, !=, <, <=, >, >=, in, not in, is, is not
            TokenKind::EqualEqual
            | TokenKind::NotEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::In
            | TokenKind::Is
            | TokenKind::Not => 4,
            TokenKind::Ampersand | TokenKind::Pipe | TokenKind::Caret => 3,
            TokenKind::And => 2,
            TokenKind::Or => 1,
            _ => 0,
        }
    }
}
