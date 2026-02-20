#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // literals
    Number(f64),
    String(String),
    Identifier(String),

    // operators
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    EqualEqual,
    BangEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    // punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Semicolon,

    // keywords
    Let,
    Const,
    Fn,
    If,
    Else,
    While,
    Return,
    Import,
    Export,
    True,
    False,
    Null,

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
}
