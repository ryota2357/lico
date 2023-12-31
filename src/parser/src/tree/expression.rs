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
        name: Ident<'src>,
        args: Vec<(Expression<'src>, Span)>,
    },
    IndexAccess {
        expr: (Box<Expression<'src>>, Span),
        accesser: (Box<Expression<'src>>, Span),
    },
    DotAccess {
        expr: (Box<Expression<'src>>, Span),
        accesser: Ident<'src>,
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

    // other
    Concat, // ..
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
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, (Expression<'src>, Span), ParserError<'src>> + Clone
{
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
                    .ignore_then(ident())
                    .then(invoke_post_op.clone());
                let dot_access_post_op = just(Token::Dot).ignore_then(ident());
                let index_access_post_op = expr
                    .clone()
                    .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket));

                term.pratt((
                    postfix_expr!(
                        9,
                        invoke_post_op,
                        ((expr, expr_span), args) => {
                            Expression::Invoke {
                                expr: (Box::new(expr), expr_span),
                                args,
                            }
                        }
                    ),
                    postfix_expr!(
                        9,
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
                        9,
                        dot_access_post_op,
                        ((expr, expr_span), accesser) => {
                            Expression::DotAccess {
                                expr: (Box::new(expr), expr_span),
                                accesser,
                            }
                        }
                    ),
                    postfix_expr!(
                        9,
                        index_access_post_op,
                        ((expr, expr_span), (accesser, accesser_span)) => {
                            Expression::IndexAccess {
                                expr: (Box::new(expr), expr_span),
                                accesser: (Box::new(accesser), accesser_span),
                            }
                        }
                    ),

                    // 8: Unary (-, not)
                    prefix_unary!(8, just(Token::Minus) => Neg, [
                        Expression::Primitive(Primitive::Int(x)) => Expression::Primitive(Primitive::Int(-x)),
                        Expression::Primitive(Primitive::Float(x)) => Expression::Primitive(Primitive::Float(-x))
                    ]),
                    prefix_unary!(8, just(Token::Not) => Not),

                    // 7: Exponential (**)
                    infix_binary!(right(7), just(Token::Star2) => Pow),

                    // 6: Multiplicative (*, /, %)
                    infix_binary!(left(6), just(Token::Star) => Mul),
                    infix_binary!(left(6), just(Token::Div) => Div),
                    infix_binary!(left(6), just(Token::Mod) => Mod),

                    // 5: Additive (+, -)
                    infix_binary!(left(5), just(Token::Plus) => Add),
                    infix_binary!(left(5), just(Token::Minus) => Sub),

                    // 4: String concatenation (..)
                    infix_binary!(right(4), just(Token::Dot2) => Concat),

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

impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for Expression<'src> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
        walker_accept(self, walker);
    }
}

impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for Box<Expression<'src>> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
        walker_accept(self, walker);
    }
}

fn walker_accept<'walker, 'src: 'walker>(
    expr: &mut Expression<'src>,
    walker: &mut Walker<'walker, 'src>,
) {
    match expr {
        Expression::Unary {
            op: _,
            expr: (expr, _),
        } => {
            walker.go(expr);
        }
        Expression::Binary {
            op: _,
            lhs: (lhs, _),
            rhs: (rhs, _),
        } => {
            walker.go(lhs);
            walker.go(rhs);
        }
        Expression::Ident(ident) => {
            let Ident(name, span) = ident;
            walker.record_variable_usage(name, span);
        }
        Expression::Primitive(_) => {}
        Expression::TableObject(table) => {
            for ((key, _), (value, _)) in table.iter_mut() {
                walker.go(key);
                walker.go(value);
            }
        }
        Expression::ArrayObject(array) => {
            for (expr, _) in array.iter_mut() {
                walker.go(expr);
            }
        }
        Expression::FunctionObject(func) => {
            let result = {
                let mut waker = Walker::new();
                for Ident(arg, _) in func.args.iter() {
                    waker.record_variable_definition(arg);
                }
                waker.go(&mut func.body.block);
                let result = waker.finish();
                func.body.captures = result.captures();
                result
            };
            walker.merge(result);
        }
        Expression::Invoke {
            expr: (expr, _),
            args,
        } => {
            walker.go(expr);
            for (arg, _) in args {
                walker.go(arg);
            }
        }
        Expression::MethodCall {
            expr: (expr, _),
            name: _,
            args,
        } => {
            walker.go(expr);
            for (arg, _) in args {
                walker.go(arg);
            }
        }
        Expression::IndexAccess {
            expr: (expr, _),
            accesser: (accesser, _),
        } => {
            walker.go(expr);
            walker.go(accesser);
        }
        Expression::DotAccess {
            expr: (expr, _),
            accesser: _,
        } => {
            walker.go(expr);
        }
        Expression::Error => panic!("Error expression found."),
    }
}
