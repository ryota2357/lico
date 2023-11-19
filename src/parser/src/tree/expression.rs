use super::*;
use chumsky::pratt::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'src> {
    Unary {
        op: UnaryOp,
        expr: (Box<Expression<'src>>, Span),
    },
    Binary {
        op: BinaryOp,
        lhs: (Box<Expression<'src>>, Span),
        rhs: (Box<Expression<'src>>, Span),
    },
    Ident(Ident<'src>),
    Primitive(Primitive),
    TableObject(TableObject<'src>),
    ArrayObject(ArrayObject<'src>),
    FunctionObject(FunctionObject<'src>),
    Invoke {
        expr: (Box<Expression<'src>>, Span),
        args: Vec<(Expression<'src>, Span)>,
    },
    MethodCall {
        expr: (Box<Expression<'src>>, Span),
        name: (Ident<'src>, Span),
        args: Vec<(Expression<'src>, Span)>,
    },
    IndexAccess {
        expr: (Box<Expression<'src>>, Span),
        accesser: (Box<Expression<'src>>, Span),
    },
    DotAccess {
        expr: (Box<Expression<'src>>, Span),
        accesser: (Ident<'src>, Span),
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
        infix(
            $associativity,
            $op_parser,
            |(lhs, lhs_span): (_, Span), (rhs, rhs_span): (_, Span)| {
                let span = lhs_span.start..rhs_span.end;
                (
                    Expression::Binary {
                        op: BinaryOp::$op,
                        lhs: (Box::new(lhs), lhs_span),
                        rhs: (Box::new(rhs), rhs_span),
                    },
                    span.into(),
                )
            },
        )
    };
}

macro_rules! prefix_unary {
    ($binding_power:expr, $op_parser:expr => $op:ident $(, [ $($rhs:pat => $to:expr),* $(,)? ] )?) => {
        prefix(
            $binding_power,
            $op_parser,
            |_, (rhs, rhs_span), extra: &mut chumsky::input::MapExtra<'tokens, '_, _, _>| {
            let span: SimpleSpan = extra.span();
            match rhs {
                $($($rhs => ($to, span.into()),)*)?
                _ => (
                    Expression::Unary {
                        op: UnaryOp::$op,
                        expr: (Box::new(rhs), rhs_span),
                    },
                    span.into()
                )
            }
        })
    };
}

macro_rules! postfix_expr {
    ($binding_power:expr, $op_parser:expr, ($($args:tt),*) => { $node:expr }) => {
        postfix(
            $binding_power,
            $op_parser,
            |$($args),*, extra: &mut chumsky::input::MapExtra<'tokens, '_, _, _>| {
                let span: SimpleSpan = extra.span();
                ($node, span.into())
            },
        )
    };
}

/// <Expression> ::= <Call> | <Unary> | <Binary> | <Primitive> | <TableObject> | <ArrayObject> | <FunctionObject> | <Local>
pub(super) fn expression<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    (Expression<'src>, Span),
    ParserError<'tokens, 'src>,
> + Clone {
    recursive(|expr| {
        let primitive = primitive()
            .map(Expression::Primitive)
            .map_with(|expr, ext| (expr, ext.span().into()));
        let table_object = table_object(expr.clone())
            .map(Expression::TableObject)
            .map_with(|expr, ext| (expr, ext.span().into()));
        let array_object = array_object(expr.clone())
            .map(Expression::ArrayObject)
            .map_with(|expr, ext| (expr, ext.span().into()));
        let function_object = function_object(block)
            .map(Expression::FunctionObject)
            .map_with(|expr, ext| (expr, ext.span().into()));

        let pratt = {
            let atom = choice((
                primitive.clone(),
                table_object.clone(),
                array_object.clone(),
                function_object
                    .clone()
                    .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
                ident()
                    .map(Expression::Ident)
                    .map_with(|expr, ext| (expr, ext.span().into())),
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
                    .ignore_then(spanned_ident())
                    .then(invoke_post_op.clone());
                let dot_access_post_op = just(Token::Dot).ignore_then(spanned_ident());
                let index_access_post_op = expr
                    .clone()
                    .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket));

                term.pratt((
                    postfix_expr!(
                        8,
                        invoke_post_op,
                        ((expr, expr_span), args) => {
                            Expression::Invoke {
                                expr: (Box::new(expr), expr_span),
                                args,
                            }
                        }
                    ),
                    postfix_expr!(
                        8,
                        method_call_post_op,
                        ((expr, expr_span), (name, args)) => {
                            Expression::MethodCall {
                                expr: (Box::new(expr), expr_span),
                                name,
                                args,
                            }
                        }
                    ),
                    postfix_expr!(
                        8,
                        dot_access_post_op,
                        ((expr, expr_span), (accesser,accesser_span)) => {
                            Expression::DotAccess {
                                expr: (Box::new(expr), expr_span),
                                accesser: (accesser, accesser_span),
                            }
                        }
                    ),
                    postfix_expr!(
                        8,
                        index_access_post_op,
                        ((expr, expr_span), (accesser,accesser_span)) => {
                            Expression::IndexAccess {
                                expr: (Box::new(expr), expr_span),
                                accesser: (Box::new(accesser), accesser_span),
                            }
                        }
                    ),

                    // 7: Unary (-, not)
                    prefix_unary!(7, just(Token::Minus) => Neg, [
                        Expression::Primitive(Primitive::Int(x)) => Expression::Primitive(Primitive::Int(-x)),
                        Expression::Primitive(Primitive::Float(x)) => Expression::Primitive(Primitive::Float(-x))
                    ]),
                    prefix_unary!(7, just(Token::Not) => Not),

                    // 6: Exponential (**)
                    infix_binary!(right(6), just(Token::Star2) => Pow),

                    // 5: Multiplicative (*, /, %)
                    infix_binary!(left(5), just(Token::Star) => Mul),
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
            ident()
                .map(Expression::Ident)
                .map_with(|expr, ext| (expr, ext.span().into())),
        ))
    })
}

impl<'a> TreeWalker<'a> for Expression<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            Expression::Unary {
                expr: (expr, _), ..
            } => expr.analyze(tracker),
            Expression::Binary {
                rhs: (rhs, _),
                lhs: (lhs, _),
                ..
            } => {
                rhs.analyze(tracker);
                lhs.analyze(tracker);
            }
            Expression::Primitive(_) => {}
            Expression::TableObject(table_object) => table_object.analyze(tracker),
            Expression::ArrayObject(array_object) => array_object.analyze(tracker),
            Expression::FunctionObject(function_object) => function_object.analyze(tracker),
            Expression::Invoke {
                expr: (expr, _),
                args,
            } => {
                expr.analyze(tracker);
                for (arg, _) in args {
                    arg.analyze(tracker);
                }
            }
            Expression::MethodCall {
                expr: (expr, _),
                args,
                ..
            } => {
                expr.analyze(tracker);
                for (arg, _) in args {
                    arg.analyze(tracker);
                }
            }
            Expression::IndexAccess {
                expr: (expr, _),
                accesser: (index, _),
            } => {
                expr.analyze(tracker);
                index.analyze(tracker);
            }
            Expression::Ident(ident) => tracker.add_capture(ident),
            Expression::DotAccess {
                expr: (expr, _), ..
            } => {
                expr.analyze(tracker);
            }
        }
    }
}
