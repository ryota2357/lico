use super::*;
use ecow::EcoString;

#[derive(Clone, Debug, PartialEq)]
pub struct Program<'src> {
    // NOTE: `attributes` should be sorted by span (TextSpan)
    pub attributes: Vec<(&'src str, Vec<TextSpan>)>,
    pub body: Chunk<'src>,
}

macro_rules! unit_object {
    (#[$meta:meta] $name:ident <=> $target:ty) => {
        #[$meta]
        pub struct $name<'src>(pub $target);
        impl<'src> ::std::ops::Deref for $name<'src> {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<'src> ::std::ops::DerefMut for $name<'src> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

unit_object!(
    #[derive(Clone, Debug, PartialEq)]
    Block <=> Vec<(Statement<'src>, TextSpan)>
);

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk<'src> {
    // NOTE: `captures` should be sorted by name (str)
    pub captures: Vec<(&'src str, TextSpan)>,
    pub block: Block<'src>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'src> {
    // variable
    Var {
        name: (&'src str, TextSpan),
        expr: (Expression<'src>, TextSpan),
    },
    Func {
        name: (&'src str, TextSpan),
        args: Vec<(FunctArgAnnotation, &'src str, TextSpan)>,
        body: Chunk<'src>,
    },
    FieldFunc {
        table: (&'src str, TextSpan),
        fields: Vec<(&'src str, TextSpan)>,
        args: Vec<(FunctArgAnnotation, &'src str, TextSpan)>,
        body: Chunk<'src>,
    },
    Assign {
        name: (&'src str, TextSpan),
        expr: (Expression<'src>, TextSpan),
    },
    FieldAssign {
        table: (Expression<'src>, TextSpan),
        field: (Expression<'src>, TextSpan),
        expr: (Expression<'src>, TextSpan),
    },

    // control
    If {
        cond: (Expression<'src>, TextSpan),
        body: Block<'src>,
        elifs: Vec<((Expression<'src>, TextSpan), Block<'src>)>,
        else_: Option<Block<'src>>,
    },
    For {
        value: (&'src str, TextSpan),
        iter: (Expression<'src>, TextSpan),
        body: Block<'src>,
    },
    While {
        cond: (Expression<'src>, TextSpan),
        body: Block<'src>,
    },
    Do {
        body: Block<'src>,
    },
    Return {
        value: Option<(Expression<'src>, TextSpan)>,
    },
    Continue,
    Break,

    // call
    Call {
        expr: (Expression<'src>, TextSpan),
        args: Vec<(Expression<'src>, TextSpan)>,
    },
    MethodCall {
        expr: (Expression<'src>, TextSpan),
        name: (&'src str, TextSpan),
        args: Vec<(Expression<'src>, TextSpan)>,
    },

    // attribute
    Attribute {
        name: (&'src str, TextSpan),
        args: Option<Vec<(&'src str, TextSpan)>>,
    },

    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'src> {
    Unary {
        op: UnaryOp,
        expr: (Box<Expression<'src>>, TextSpan),
    },
    Binary {
        op: BinaryOp,
        lhs: (Box<Expression<'src>>, TextSpan),
        rhs: (Box<Expression<'src>>, TextSpan),
    },
    Local(&'src str, TextSpan),
    Primitive(Primitive, TextSpan),
    TableObject(TableObject<'src>),
    ArrayObject(ArrayObject<'src>),
    FunctionObject(FunctionObject<'src>),
    Call {
        expr: (Box<Expression<'src>>, TextSpan),
        args: Vec<(Expression<'src>, TextSpan)>,
    },
    MethodCall {
        expr: (Box<Expression<'src>>, TextSpan),
        name: (&'src str, TextSpan),
        args: Vec<(Expression<'src>, TextSpan)>,
    },
    IndexAccess {
        expr: (Box<Expression<'src>>, TextSpan),
        accessor: (Box<Expression<'src>>, TextSpan),
    },
    DotAccess {
        expr: (Box<Expression<'src>>, TextSpan),
        accessor: (&'src str, TextSpan),
    },
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg, // -
    Not, // not
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    // arithmetic
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %

    // comparison
    Eq,        // ==
    NotEq,     // !=
    Less,      // <
    LessEq,    // <=
    Greater,   // >
    GreaterEq, // >=

    // logical
    And, // and
    Or,  // or

    // other
    Concat, // ..
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Int(i64),
    Float(f64),
    String(EcoString),
    Bool(bool),
    Nil,
}

unit_object!(
    #[derive(Clone, Debug, PartialEq)]
    TableObject <=> Vec<(TableFieldKey<'src>, (Expression<'src>, TextSpan))>
);

#[derive(Clone, Debug, PartialEq)]
pub enum TableFieldKey<'src> {
    Ident(&'src str, TextSpan),
    Expr(Expression<'src>, TextSpan),
    // Func(&'src str)
}

unit_object!(
    #[derive(Clone, Debug, PartialEq)]
    ArrayObject <=> Vec<(Expression<'src>, TextSpan)>
);

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionObject<'src> {
    pub args: Vec<(FunctArgAnnotation, &'src str, TextSpan)>,
    pub body: Chunk<'src>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctArgAnnotation {
    None,
    Ref,
    In,
}
