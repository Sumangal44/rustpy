#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Identifier(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
    NoneLiteral,
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Subscript {
        value: Box<Expr>,
        slice: Box<Expr>,
    },
    Attribute {
        value: Box<Expr>,
        attr: String,
    },
    BinOp {
        left: Box<Expr>,
        op: BinOpKind,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOpKind,
        operand: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        kwargs: Vec<(String, Expr)>,
        starargs: Vec<Expr>,
        kwargs_unpack: Vec<Expr>,
    },
    Yield(Option<Box<Expr>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mult,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpKind {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    FunctionDef {
        name: String,
        params: Vec<String>,
        vararg: Option<String>,
        kwarg: Option<String>,
        body: Vec<Stmt>,
    },
    ClassDef {
        name: String,
        bases: Vec<Expr>,
        body: Vec<Stmt>,
    },
    Try {
        body: Vec<Stmt>,
        handlers: Vec<(String, Vec<Stmt>)>,
    },
    Raise {
        exc: Box<Expr>,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        test: Expr,
        body: Vec<Stmt>,
        orelse: Vec<Stmt>,
    },
    While {
        test: Expr,
        body: Vec<Stmt>,
    },
    For {
        target: Expr,
        iter: Expr,
        body: Vec<Stmt>,
    },
    Assign {
        targets: Vec<Expr>,
        value: Expr,
    },
    ExprStmt {
        value: Expr,
    },
    YieldStmt(Expr),
    Pass,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub body: Vec<Stmt>,
}
