use super::*;
use chumsky::pratt::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'src> {
    Unary {
        op: UnaryOp,
        expr: Box<Expression<'src>>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expression<'src>>,
        rhs: Box<Expression<'src>>,
    },
    Ident(Ident<'src>),
    Primitive(Primitive<'src>),
    TableObject(TableObject<'src>),
    ArrayObject(ArrayObject<'src>),
    FunctionObject(FunctionObject<'src>),
    Invoke {
        expr: Box<Expression<'src>>,
        args: Vec<Expression<'src>>,
    },
    CallMethod {
        expr: Box<Expression<'src>>,
        name: Ident<'src>,
        args: Vec<Expression<'src>>,
    },
    IndexAccess {
        expr: Box<Expression<'src>>,
        accesser: Box<Expression<'src>>,
    },
    DotAccess {
        expr: Box<Expression<'src>>,
        accesser: Ident<'src>,
    },
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
    Pow, // *

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
}

/// <Expression> ::= <Call> | <Unary> | <Binary> | <Primitive> | <TableObject> | <ArrayObject> | <FunctionObject> | <Local>
///
/// <Unary> and <Binary> operators priority:
/// (1 is lowest, 8 is highest.)
/// 7 : `Neg`, `Not`
/// 6 : `Pow`
/// 5 : `Mul`, `Div`, `Mod`
/// 4 : `Add`, `Sub`
/// 3 : `Less`, `LessEq`, `Greater`, `GreaterEq`, `Eq`, `NotEq`
/// 2 : `And`
/// 1 : `Or`
pub(super) fn expression<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>> + Clone
{
    recursive(|expr| {
        let primitive = primitive().map(Expression::Primitive);
        let table_object = table_object(expr.clone()).map(Expression::TableObject);
        let array_object = array_object(expr.clone()).map(Expression::ArrayObject);
        let function_object = function_object(block.clone()).map(Expression::FunctionObject);

        let pratt = {
            let atom = choice((
                primitive.clone(),
                table_object.clone(),
                array_object.clone(),
                function_object
                    .clone()
                    .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
                ident().map(Expression::Ident),
            ));
            recursive(|pratt| {
                let term = choice((
                    atom.clone()
                        .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
                    pratt.delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
                    atom,
                ));
                term.pratt((
                    postfix(
                        8,
                        expr.clone()
                            .separated_by(just(Token::Comma))
                            .allow_trailing()
                            .collect()
                            .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
                        |lhs, args| Expression::Invoke {
                            expr: Box::new(lhs),
                            args,
                        },
                    ),
                    postfix(
                        8,
                        just(Token::Arrow).ignore_then(ident()).then(
                            expr.clone()
                                .separated_by(just(Token::Comma))
                                .allow_trailing()
                                .collect()
                                .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
                        ),
                        |expr, (name, args)| Expression::CallMethod {
                            expr: Box::new(expr),
                            name,
                            args,
                        },
                    ),
                    postfix(
                        8,
                        just(Token::Dot).ignore_then(ident()),
                        |expr, accesser| Expression::DotAccess {
                            expr: Box::new(expr),
                            accesser,
                        },
                    ),
                    postfix(
                        8,
                        expr.clone()
                            .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket)),
                        |expr, accesser| Expression::IndexAccess {
                            expr: Box::new(expr),
                            accesser: Box::new(accesser),
                        },
                    ),
                    prefix(7, just(Token::Minus), |rhs| match rhs {
                        Expression::Primitive(Primitive::Int(x)) => {
                            Expression::Primitive(Primitive::Int(-x))
                        }
                        Expression::Primitive(Primitive::Float(x)) => {
                            Expression::Primitive(Primitive::Float(-x))
                        }
                        _ => Expression::Unary {
                            op: UnaryOp::Neg,
                            expr: Box::new(rhs),
                        },
                    }),
                    prefix(7, just(Token::Not), |rhs| match rhs {
                        Expression::Primitive(Primitive::Bool(x)) => {
                            Expression::Primitive(Primitive::Bool(!x))
                        }
                        _ => Expression::Unary {
                            op: UnaryOp::Not,
                            expr: Box::new(rhs),
                        },
                    }),
                    infix(right(6), just(Token::Pow), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Pow,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(5), just(Token::Mul), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Mul,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(5), just(Token::Div), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Div,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(5), just(Token::Mod), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Mod,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(4), just(Token::Pluss), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Add,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(4), just(Token::Minus), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Sub,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(3), just(Token::Less), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Less,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(3), just(Token::LessEq), |lhs, rhs| {
                        Expression::Binary {
                            op: BinaryOp::LessEq,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        }
                    }),
                    infix(left(3), just(Token::Greater), |lhs, rhs| {
                        Expression::Binary {
                            op: BinaryOp::Greater,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        }
                    }),
                    infix(left(3), just(Token::GreaterEq), |lhs, rhs| {
                        Expression::Binary {
                            op: BinaryOp::GreaterEq,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        }
                    }),
                    infix(left(3), just(Token::Eq), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Eq,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(3), just(Token::NotEq), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::NotEq,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(2), just(Token::And), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::And,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(1), just(Token::Or), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Or,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                ))
            })
        };

        choice((
            pratt,
            expr.delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
            primitive,
            table_object,
            array_object,
            function_object,
            ident().map(Expression::Ident),
        ))
    })
}

impl<'a> TreeWalker<'a> for Expression<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            Expression::Unary { expr, .. } => expr.analyze(tracker),
            Expression::Binary { rhs, lhs, .. } => {
                rhs.analyze(tracker);
                lhs.analyze(tracker);
            }
            Expression::Primitive(_) => {}
            Expression::TableObject(table_object) => table_object.analyze(tracker),
            Expression::ArrayObject(array_object) => array_object.analyze(tracker),
            Expression::FunctionObject(function_object) => function_object.analyze(tracker),
            Expression::Invoke { expr, args } => {
                expr.analyze(tracker);
                for arg in args {
                    arg.analyze(tracker);
                }
            }
            Expression::CallMethod { expr, args, .. } => {
                expr.analyze(tracker);
                for arg in args {
                    arg.analyze(tracker);
                }
            }
            Expression::IndexAccess {
                expr,
                accesser: index,
            } => {
                expr.analyze(tracker);
                index.analyze(tracker);
            }
            Expression::Ident(ident) => tracker.add_capture(ident.str),
            Expression::DotAccess { expr, .. } => {
                expr.analyze(tracker);
            }
        }
    }
}
