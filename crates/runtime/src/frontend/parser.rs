use crate::{
    frontend::token::{Token, TokenKind},
    runtime::ast::*,
};

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, String> {
    let mut parser = Parser::new(tokens);

    parser.parse()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    #[inline(always)]
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        let mut is_exported = false;

        if self.match_kind(&[TokenKind::Export]) {
            is_exported = true;
        }

        if self.match_kind(&[TokenKind::Let]) {
            return self.let_statement(false, is_exported);
        }

        if self.match_kind(&[TokenKind::Const]) {
            return self.let_statement(true, is_exported);
        }

        if self.match_kind(&[TokenKind::Fn]) {
            return self.fn_statement(is_exported);
        }

        if self.match_kind(&[TokenKind::Import]) {
            return self.import_statement();
        }

        if self.match_kind(&[TokenKind::If]) {
            return self.if_statement();
        }
        
        if self.match_kind(&[TokenKind::While]) {
            return self.while_statement();
        }
        
        if self.match_kind(&[TokenKind::Return]) {
            return self.return_statement();
        }
        
        if self.match_kind(&[TokenKind::LBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }

        let expr = self.expression()?;
        
        Ok(Stmt::Expr(expr))
    }

    fn import_statement(&mut self) -> Result<Stmt, String> {
        let token = self.advance().clone();

        match token.kind {
            TokenKind::String(s) => Ok(Stmt::Import(s)),

            TokenKind::Identifier(id) => {
                let mut full_path = id;

                while self.match_kind(&[TokenKind::Dot]) {
                    match self.advance().kind.clone() {
                        TokenKind::Identifier(next_id) => full_path.push_str(&format!(".{}", next_id)),
                        
                        _ => return Err("expected identifier after '.'".into()),
                    }
                }

                Ok(Stmt::Import(full_path))
            }

            _ => Err("expected module path or string after import".into()),
        }
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }

        self.consume(TokenKind::RBrace, "expected '}' after block")?;

        Ok(statements)
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        let condition = self.expression()?;

        self.consume(TokenKind::LBrace, "expected '{' after condition")?;

        let body = self.block()?;

        Ok(Stmt::While { condition, body })
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        if self.check(&TokenKind::RBrace) {
            return Ok(Stmt::Return(None));
        }

        let value = self.expression()?;

        Ok(Stmt::Return(Some(value)))
    }

    fn let_statement(&mut self, is_const: bool, is_exported: bool) -> Result<Stmt, String> {
        let name = match self.advance().kind.clone() {
            TokenKind::Identifier(s) => s,
            
            _ => return Err("expected identifier".into()),
        };

        self.consume(TokenKind::Equal, "expected '=' after variable name")?;

        let value = self.expression()?;

        Ok(Stmt::Let { name, value, is_const, is_exported })
    }

    fn fn_statement(&mut self, is_exported: bool) -> Result<Stmt, String> {
        let name = match self.advance().kind.clone() {
            TokenKind::Identifier(s) => s,
            
            _ => return Err("expected function name".into()),
        };

        self.consume(TokenKind::LParen, "expected '(' after function name")?;
        
        let mut params = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                match self.advance().kind.clone() {
                    TokenKind::Identifier(s) => params.push(s),
                    
                    _ => return Err("expected parameter name".into()),
                }
                
                if !self.match_kind(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RParen, "expected ')' after parameters")?;
        self.consume(TokenKind::LBrace, "expected '{' before function body")?;
        
        let body = self.block()?;

        Ok(Stmt::Function { name, params, body, is_exported })
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        let condition = self.expression()?;

        self.consume(TokenKind::LBrace, "expected '{' after condition")?;
        
        let then_branch = self.block()?;

        let else_branch = if self.match_kind(&[TokenKind::Else]) {
            self.consume(TokenKind::LBrace, "expected '{' after else")?;
            
            Some(self.block()?)
        } else {
            None
        };

        Ok(Stmt::If { condition, then_branch, else_branch })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_kind(&[TokenKind::EqualEqual, TokenKind::BangEqual]) {
            let operator = self.previous().kind.clone();
            let right = self.comparison()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_kind(&[TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) {
            let operator = self.previous().kind.clone();
            let right = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_kind(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous().kind.clone();
            let right = self.factor()?;
            
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_kind(&[TokenKind::Star, TokenKind::Slash]) {
            let operator = self.previous().kind.clone();
            let right = self.unary()?;
            
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_kind(&[TokenKind::LParen]) {
                let mut args = Vec::new();

                if !self.check(&TokenKind::RParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_kind(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RParen, "expected ')' after arguments")?;

                expr = Expr::Call { callee: Box::new(expr), args };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.advance().clone();

        match token.kind {
            TokenKind::Number(n) => Ok(Expr::Literal(ValueLiteral::Number(n))),
            TokenKind::String(s) => Ok(Expr::Literal(ValueLiteral::String(s))),
            TokenKind::True => Ok(Expr::Literal(ValueLiteral::Bool(true))),
            TokenKind::False => Ok(Expr::Literal(ValueLiteral::Bool(false))),
            TokenKind::Null => Ok(Expr::Literal(ValueLiteral::Null)),
            TokenKind::Identifier(s) => Ok(Expr::Variable(s)),

            TokenKind::Fn => {
                let name = if let TokenKind::Identifier(_) = self.peek().kind {
                    if let TokenKind::Identifier(s) = self.advance().kind.clone() { s } else { "".to_string() }
                } else {
                    "".to_string()
                };

                self.consume(TokenKind::LParen, "expected '(' after function name")?;
                
                let mut params = Vec::new();

                if !self.check(&TokenKind::RParen) {
                    loop {
                        match self.advance().kind.clone() {
                            TokenKind::Identifier(s) => params.push(s),
                            _ => return Err("expected parameter name".into()),
                        }
                        if !self.match_kind(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RParen, "expected ')' after parameters")?;
                self.consume(TokenKind::LBrace, "expected '{' before function body")?;
                let body = self.block()?;

                Ok(Expr::Function { name, params, body })
            }

            TokenKind::LParen => {
                let expr = self.expression()?;

                self.consume(TokenKind::RParen, "expected ')'")?;

                Ok(expr)
            }

            TokenKind::LBracket => {
                let mut elements = Vec::new();

                if !self.check(&TokenKind::RBracket) {
                    loop {
                        elements.push(self.expression()?);

                        if !self.match_kind(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(TokenKind::RBracket, "expected ']'")?;

                Ok(Expr::Array(elements))
            }
            _ => Err("unexpected token".into()),
        }
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.equality()?;

        if self.match_kind(&[TokenKind::Equal]) {
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign { name, value: Box::new(value) });
            }

            return Err("Invalid assignment target".into());
        }

        // Compound assignments
        let compound_map = [
            (TokenKind::PlusEqual, TokenKind::Plus),
            (TokenKind::MinusEqual, TokenKind::Minus),
            (TokenKind::StarEqual, TokenKind::Star),
            (TokenKind::SlashEqual, TokenKind::Slash),
        ];

        for (compound, operator) in compound_map {
            if self.match_kind(&[compound]) {
                let value = self.assignment()?;

                if let Expr::Variable(name) = expr.clone() {
                    return Ok(Expr::Assign {
                        name: name.clone(),
                        value: Box::new(Expr::Binary {
                            left: Box::new(Expr::Variable(name)),
                            operator,
                            right: Box::new(value),
                        }),
                    });
                } else {
                    return Err("Invalid assignment target".into());
                }
            }
        }

        Ok(expr)
    }

    // utilities

    #[inline(always)]
    fn match_kind(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    #[inline(always)]
    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    #[inline(always)]
    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<(), String> {
        if self.check(&kind) {
            self.advance();

            Ok(())
        } else {
            Err(msg.into())
        }
    }

    #[inline(always)]
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    #[inline(always)]
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    #[inline(always)]
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    #[inline(always)]
    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::EOF)
    }
}
