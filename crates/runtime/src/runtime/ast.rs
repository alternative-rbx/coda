use crate::frontend::token::TokenKind;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(ValueLiteral),
    Variable(String),
    Binary { left: Box<Expr>, operator: TokenKind, right: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Assign { name: String, value: Box<Expr> },
    Array(Vec<Expr>),

    Function { name: String, params: Vec<String>, body: Vec<Stmt> },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
        is_const: bool,
        is_exported: bool,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        is_exported: bool,
    },
    Return(Option<Expr>),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    Import(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum ValueLiteral {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}
