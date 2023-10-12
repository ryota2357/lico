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
    Primitive(Primitive<'src>),
    TableObject(TableObject<'src>),
    ArrayObject(ArrayObject<'src>),
    FunctionObject(FunctionObject<'src>),
    Call(Call<'src>),
    Local(Local<'src>),
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

/// TODO: '(' <Expression> ')' を処理できないんだけど、どうする？
/// <Expression> ::= '(' <Expression> ')' | <expression>
/// <expression> ::= <Unary> | <Binary> | <Primitive> | <TableObject> | <ArrayObject> | <FunctionObject> | <Call> | <Local>
///
/// <Unary> and <Binary> operators priority:
/// (1 is lowest, 8 is highest.)
/// 8 : `Neg`, `Not`
/// 7 : `Pow`
/// 6 : `Mul`, `Div`, `Mod`
/// 5 : `Add`, `Sub`
/// 4 : `Less`, `LessEq`, `Greater`, `GreaterEq`, `Eq`, `NotEq`
/// 2 : `And`
/// 1 : `Or`
pub(super) fn expression<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>> + Clone
{
    let expr = recursive(|expr| {
        // TODO: unary と binary は pratt パーサー (ver2) がマージされたら
        let primitive = primitive().map(Expression::Primitive);
        let table_object = table_object(expr.clone()).map(Expression::TableObject);
        let array_object = array_object(expr.clone()).map(Expression::ArrayObject);
        let function_object = function_object(block.clone()).map(Expression::FunctionObject);
        let call = call(block, expr).map(Expression::Call);
        let local = local().map(Expression::Local);

        let unary_or_binary = {
            let atom = choice((primitive.clone(), call.clone(), local.clone()));
            recursive(|pratt| {
                let term = choice((
                    atom.clone()
                        .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
                    pratt.delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
                    atom,
                ));
                term.pratt((
                    prefix(7, just(Token::Sub), |rhs| match rhs {
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
                            op: UnaryOp::Neg,
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
                    infix(left(4), just(Token::Add), |lhs, rhs| Expression::Binary {
                        op: BinaryOp::Add,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    }),
                    infix(left(4), just(Token::Sub), |lhs, rhs| Expression::Binary {
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
                ))
            })
        };

        choice((
            unary_or_binary,
            primitive,
            table_object,
            array_object,
            function_object,
            call,
            local,
        ))
    });

    expr
    // let delimited_expr = expr
    //     .clone()
    //     .delimited_by(just(Token::OpenParen), just(Token::CloseParen));
    //
    // delimited_expr.or(expr)
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
            Expression::Call(call) => call.analyze(tracker),
            Expression::Local(local) => match local {
                Local::TableField { name, .. } => tracker.add_capture(name.str),
                Local::Variable { name } => tracker.add_capture(name.str),
            },
        }
    }
}
