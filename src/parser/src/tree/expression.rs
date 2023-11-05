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
    Primitive(Primitive),
    TableObject(TableObject<'src>),
    ArrayObject(ArrayObject<'src>),
    FunctionObject(FunctionObject<'src>),
    Invoke {
        expr: Box<Expression<'src>>,
        args: Vec<Expression<'src>>,
    },
    MethodCall {
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

macro_rules! infix_binary {
    ($associativity:expr, $op_parser:expr => $op:ident) => {
        infix($associativity, $op_parser, |lhs, rhs| Expression::Binary {
            op: BinaryOp::$op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    };
}

macro_rules! prefix_unary {
    ($binding_power:expr, $op_parser:expr => $op:ident $(, [ $($rhs:pat => $to:expr),* $(,)? ] )?) => {
        prefix($binding_power, $op_parser, |rhs| match rhs {
            $($($rhs => $to,)*)?
            _ => Expression::Unary {
                op: UnaryOp::$op,
                expr: Box::new(rhs),
            },
        })
    };
}

/// <Expression> ::= <Call> | <Unary> | <Binary> | <Primitive> | <TableObject> | <ArrayObject> | <FunctionObject> | <Local>
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

                let invoke_post_op = expr
                    .clone()
                    .separated_by(just(Token::Comma))
                    .allow_trailing()
                    .collect()
                    .delimited_by(just(Token::OpenParen), just(Token::CloseParen));
                let method_call_post_op = just(Token::Arrow)
                    .ignore_then(ident())
                    .then(invoke_post_op.clone());
                let dot_access_post_op = just(Token::Dot).ignore_then(ident());
                let index_access_post_op = expr
                    .clone()
                    .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket));

                term.pratt((
                    // 8: Invoke, MethodCall, DotAccess, IndexAccess
                    postfix(8, invoke_post_op, |lhs, args| Expression::Invoke {
                        expr: Box::new(lhs),
                        args,
                    }),
                    postfix(8, method_call_post_op, |expr, (name, args)| Expression::MethodCall {
                        expr: Box::new(expr),
                        name,
                        args,
                    }),
                    postfix(8, dot_access_post_op, |expr, accesser| Expression::DotAccess {
                        expr: Box::new(expr),
                        accesser,
                    }),
                    postfix(8, index_access_post_op, |expr, accesser| Expression::IndexAccess {
                        expr: Box::new(expr),
                        accesser: Box::new(accesser),
                    }),

                    // 7: Unary (-, not)
                    prefix_unary!(7, just(Token::Minus) => Neg, [
                        Expression::Primitive(Primitive::Int(x)) => Expression::Primitive(Primitive::Int(-x)),
                        Expression::Primitive(Primitive::Float(x)) => Expression::Primitive(Primitive::Float(-x))
                    ]),
                    prefix_unary!(7, just(Token::Not) => Not),

                    // 6: Exponential (**)
                    infix_binary!(right(6), just(Token::Pow) => Pow),

                    // 5: Multiplicative (*, /, %)
                    infix_binary!(left(5), just(Token::Mul) => Mul),
                    infix_binary!(left(5), just(Token::Div) => Div),
                    infix_binary!(left(5), just(Token::Mod) => Mod),

                    // 4: Additive (+, -)
                    infix_binary!(left(4), just(Token::Pluss) => Add),
                    infix_binary!(left(4), just(Token::Minus) => Sub),

                    // 3: Relational (<, <=, >, >=)
                    infix_binary!(left(3), just(Token::Less)      => Less),
                    infix_binary!(left(3), just(Token::LessEq)    => LessEq),
                    infix_binary!(left(3), just(Token::Greater)   => Greater),
                    infix_binary!(left(3), just(Token::GreaterEq) => GreaterEq),

                    // 2: Equality (==, !=)
                    infix_binary!(left(2), just(Token::Eq)    => Eq),
                    infix_binary!(left(2), just(Token::NotEq) => NotEq),

                    // 1; Logical-AND (and)
                    infix_binary!(left(1), just(Token::And) => And),

                    // 0: Logical-OR (or)
                    infix_binary!(left(0), just(Token::Or) => Or),
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
            Expression::MethodCall { expr, args, .. } => {
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
