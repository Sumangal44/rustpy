#[derive(Debug, Clone, PartialEq)]
pub enum FStringSegment {
    Text(String),
    Expr {
        expr: Box<Expr>,
        format_spec: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Identifier(String),
    IntLiteral(String),
    FloatLiteral(f64),
    ImagLiteral(f64),
    StringLiteral(String),
    BytesLiteral(Vec<u8>),
    BooleanLiteral(bool),
    NoneLiteral,
    Ellipsis,
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Subscript {
        value: Box<Expr>,
        slice: Box<Expr>,
    },
    Slice {
        value: Box<Expr>,
        start: Option<Box<Expr>>,
        stop: Option<Box<Expr>>,
        step: Option<Box<Expr>>,
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
    YieldFrom(Box<Expr>),
    IfExp {
        test: Box<Expr>,
        body: Box<Expr>,
        orelse: Box<Expr>,
    },
    Comprehension {
        kind: CompKind,
        elt: Box<Expr>,
        key: Option<Box<Expr>>,
        target: Box<Expr>,
        iter: Box<Expr>,
        ifs: Vec<Expr>,
    },
    Lambda {
        params: Vec<String>,
        posonly_params: Vec<String>,
        kwonly_params: Vec<String>,
        vararg: Option<String>,
        kwarg: Option<String>,
        body: Box<Expr>,
    },
    Starred {
        value: Box<Expr>,
    },
    Await(Box<Expr>),
    FString(Vec<FStringSegment>),
    NamedExpr {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Set(Vec<Expr>),
    Tuple(Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompKind {
    List,
    Dict,
    Generator,
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    In,
    NotIn,
    Is,
    IsNot,
    And,
    Or,
    MatMul,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    RShift,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpKind {
    Plus,
    Minus,
    Not,
    Invert,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    FunctionDef {
        name: String,
        posonly_params: Vec<String>,
        params: Vec<String>,
        kwonly_params: Vec<String>,
        defaults: Vec<Option<Expr>>,
        vararg: Option<String>,
        kwarg: Option<String>,
        body: Vec<Stmt>,
        decorators: Vec<Expr>,
        is_async: bool,
        returns: Option<Box<Expr>>,
    },
    ClassDef {
        name: String,
        bases: Vec<Expr>,
        body: Vec<Stmt>,
        decorators: Vec<Expr>,
    },
    Try {
        body: Vec<Stmt>,
        handlers: Vec<(Option<String>, Option<String>, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
        finally_body: Option<Vec<Stmt>>,
    },
    Raise {
        exc: Option<Box<Expr>>,
        cause: Option<Box<Expr>>,
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
        orelse: Vec<Stmt>,
    },
    For {
        target: Expr,
        iter: Expr,
        body: Vec<Stmt>,
        orelse: Vec<Stmt>,
        is_async: bool,
    },
    With {
        items: Vec<WithItem>,
        body: Vec<Stmt>,
        is_async: bool,
    },
    Assign {
        targets: Vec<Expr>,
        value: Expr,
    },
    AugAssign {
        target: Box<Expr>,
        op: BinOpKind,
        value: Expr,
    },
    AnnAssign {
        target: Box<Expr>,
        annotation: Box<Expr>,
        value: Option<Box<Expr>>,
    },
    Break,
    Continue,
    Del {
        target: Box<Expr>,
    },
    Global {
        names: Vec<String>,
    },
    Nonlocal {
        names: Vec<String>,
    },
    Match {
        subject: Box<Expr>,
        cases: Vec<MatchCase>,
    },
    Assert {
        test: Expr,
        msg: Option<Box<Expr>>,
    },
    Import {
        names: Vec<Alias>,
    },
    ImportFrom {
        module: String,
        names: Vec<Alias>,
        level: usize,
    },
    ExprStmt {
        value: Expr,
    },
    YieldStmt(Expr),
    Pass,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Box<Expr>),
    Capture(String),
    Or(Vec<Pattern>),
    Sequence(Vec<Pattern>),
    Mapping(Vec<(Expr, Pattern)>),
    Class(String, Vec<Pattern>, Vec<(String, Pattern)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Alias {
    pub name: String,
    pub asname: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithItem {
    pub context_expr: Expr,
    pub optional_vars: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub body: Vec<Stmt>,
}
