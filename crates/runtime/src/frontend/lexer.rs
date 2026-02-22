use crate::{
    frontend::token::{Token, TokenKind},
    utils::{is_alpha, is_alphanumeric, slice_to_string},
};

pub fn scan(source: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer::new(source);
    lexer.scan_tokens()
}

struct Lexer<'a> {
    src: &'a [u8],
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    #[inline(always)]
    fn new(source: &'a str) -> Self {
        Self {
            src: source.as_bytes(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    #[inline(always)]
    fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;

            if let Some(token) = self.scan_token()? {
                tokens.push(token);
            }
        }

        tokens.push(Token {
            kind: TokenKind::EOF,
            line: self.line,
        });

        Ok(tokens)
    }

    fn scan_token(&mut self) -> Result<Option<Token>, String> {
        let c = self.advance();

        match c {
            b'(' => Ok(Some(self.token(TokenKind::LParen))),
            b')' => Ok(Some(self.token(TokenKind::RParen))),
            b'{' => Ok(Some(self.token(TokenKind::LBrace))),
            b'}' => Ok(Some(self.token(TokenKind::RBrace))),
            b',' => Ok(Some(self.token(TokenKind::Comma))),
            b'.' => Ok(Some(self.token(TokenKind::Dot))),

            b'+' => {
                if self.peek() == b'=' {
                    self.advance();

                    Ok(Some(self.token(TokenKind::PlusEqual)))
                } else {
                    Ok(Some(self.token(TokenKind::Plus)))
                }
            }

            b'-' => {
                if self.peek() == b'=' {
                    self.advance();

                    Ok(Some(self.token(TokenKind::MinusEqual)))
                } else {
                    Ok(Some(self.token(TokenKind::Minus)))
                }
            }

            b'*' => {
                if self.peek() == b'=' {
                    self.advance();

                    Ok(Some(self.token(TokenKind::StarEqual)))
                } else {
                    Ok(Some(self.token(TokenKind::Star)))
                }
            }

            b'/' => {
                if self.peek() == b'=' {
                    self.advance();

                    Ok(Some(self.token(TokenKind::SlashEqual)))
                } else if self.peek() == b'/' {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }

                    Ok(None)
                } else {
                    Ok(Some(self.token(TokenKind::Slash)))
                }
            }

            b'=' => {
                if self.peek() == b'=' {
                    self.advance();

                    Ok(Some(self.token(TokenKind::EqualEqual)))
                } else {
                    Ok(Some(self.token(TokenKind::Equal)))
                }
            }

            b' ' | b'\t' | b'\r' => Ok(None),

            b'\n' => {
                self.line += 1;

                Ok(None)
            }

            b'"' => self.string(),

            b'>' => {
                if self.peek() == b'=' {
                    self.advance();

                    Ok(Some(self.token(TokenKind::GreaterEqual)))
                } else {
                    Ok(Some(self.token(TokenKind::Greater)))
                }
            }

            b'<' => {
                if self.peek() == b'=' {
                    self.advance();

                    Ok(Some(self.token(TokenKind::LessEqual)))
                } else {
                    Ok(Some(self.token(TokenKind::Less)))
                }
            }

            b'[' => Ok(Some(self.token(TokenKind::LBracket))),
            b']' => Ok(Some(self.token(TokenKind::RBracket))),

            c if c.is_ascii_digit() => self.number(),
            c if is_alpha(c) => self.identifier(),

            _ => Err(format!("unexpected character at line {}", self.line)),
        }
    }

    fn string(&mut self) -> Result<Option<Token>, String> {
        while !self.is_at_end() && self.peek() != b'"' {
            if self.peek() == b'\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            return Err("unterminated string".into());
        }

        self.advance();

        let value = slice_to_string(&self.src[self.start + 1..self.current - 1]);

        Ok(Some(self.token(TokenKind::String(value))))
    }

    fn number(&mut self) -> Result<Option<Token>, String> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value = std::str::from_utf8(&self.src[self.start..self.current]).unwrap().parse::<f64>().unwrap();

        Ok(Some(self.token(TokenKind::Number(value))))
    }

    fn identifier(&mut self) -> Result<Option<Token>, String> {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = slice_to_string(&self.src[self.start..self.current]);

        let kind = match text.as_str() {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "import" => TokenKind::Import,
            "export" => TokenKind::Export,

            _ => TokenKind::Identifier(text),
        };

        Ok(Some(self.token(kind)))
    }

    #[inline(always)]
    fn token(&self, kind: TokenKind) -> Token {
        Token { kind, line: self.line }
    }

    #[inline(always)]
    fn advance(&mut self) -> u8 {
        let c = self.src[self.current];

        self.current += 1;

        c
    }

    #[inline(always)]
    fn peek(&self) -> u8 {
        if self.is_at_end() { 0 } else { self.src[self.current] }
    }

    #[inline(always)]
    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.src.len() { 0 } else { self.src[self.current + 1] }
    }

    #[inline(always)]
    fn is_at_end(&self) -> bool {
        self.current >= self.src.len()
    }
}
