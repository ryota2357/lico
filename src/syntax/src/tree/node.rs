type S<T> = (T, chumsky::span::SimpleSpan<usize>);

type FunctionArguments<'a> = Vec<S<Expression<'a>>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Block<'src>(pub Vec<S<Statement<'src>>>);

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'src> {
    Function(FunctionStatement<'src>),
    Variable(VariableStatement<'src>),
    Control(ControlStatement<'src>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum FunctionStatement<'src> {
    Define {
        name: S<Local<'src>>,
        args: S<FunctionArguments<'src>>,
        body: S<Block<'src>>,
    },
    Call {
        name: S<Callable<'src>>,
        args: S<FunctionArguments<'src>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum VariableStatement<'src> {
    Var {
        lhs: S<Local<'src>>,
        rhs: S<Expression<'src>>,
    },
    Let {
        lhs: S<Local<'src>>,
        rhs: S<Expression<'src>>,
    },
    Assign {
        lhs: S<Local<'src>>,
        rhs: S<Expression<'src>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ControlStatement<'src> {
    If {
        cond: S<Expression<'src>>,
        body: S<Block<'src>>,
        elifs: Vec<(S<Expression<'src>>, S<Block<'src>>)>,
        else_: Option<S<Block<'src>>>,
    },
    For {
        value: S<Local<'src>>,
        start: S<Expression<'src>>,
        end: S<Expression<'src>>,
        step: Option<S<Expression<'src>>>,
        body: S<Block<'src>>,
    },
    ForIn {
        value: S<Local<'src>>,
        iter: S<Expression<'src>>,
        body: S<Block<'src>>,
    },
    While {
        cond: S<Expression<'src>>,
        body: S<Block<'src>>,
    },
    Return(Option<S<Expression<'src>>>),
    Continue,
    Break,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'src> {
    Local(Local<'src>),
    Primitive(Primitive<'src>),
    Table(Vec<(S<&'src str>, S<Expression<'src>>)>),
    Unary {
        op: S<UnaryOp>,
        expr: S<Box<Expression<'src>>>,
    },
    Binary {
        op: S<BinaryOp>,
        lhs: S<Box<Expression<'src>>>,
        rhs: S<Box<Expression<'src>>>,
    },
    AnonymousFunc {
        args: S<FunctionArguments<'src>>,
        body: S<Block<'src>>,
    },
    FunctionCall {
        func: S<Callable<'src>>,
        args: S<FunctionArguments<'src>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Callable<'src> {
    Local(Local<'src>),
    AnonymousFunc {
        args: S<FunctionArguments<'src>>,
        body: S<Block<'src>>,
    },
    FunctionCall {
        func: S<Box<Callable<'src>>>,
        args: S<FunctionArguments<'src>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Local<'src> {
    Variable(&'src str),
    TableField {
        table: S<&'src str>,
        keys: Vec<S<&'src str>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// # Priority
//
/// 1 is highest, 8 is lowest.
///
/// 1 : `Dot`
/// 2 : `Pow`
/// 3 : `Mul`, `Div`, `Mod`
/// 4 : `Add`, `Sub`
/// 5 : `Less`, `LessEq`, `Greater`, `GreaterEq`
/// 6 : `Eq`, `NotEq`
/// 7 : `And`
/// 8 : `Or`
#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOp {
    // arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    // table access
    Dot,

    // comparison
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,

    // logical
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive<'src> {
    Int(i64),
    Float(f64),
    String(&'src str),
    Bool(bool),
    Nil,
}
