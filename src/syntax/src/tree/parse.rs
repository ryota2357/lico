use crate::lex::Token;
use crate::tree::node::*;
use chumsky::prelude::*;

type Span = SimpleSpan<usize>;
type ParseError<'tokens, 'src> = extra::Err<Rich<'tokens, Token<'src>, Span>>;
type ParserInput<'tokens, 'src> =
    chumsky::input::SpannedInput<Token<'src>, Span, &'tokens [(Token<'src>, Span)]>;

pub fn parser<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, (Block<'src>, Span), ParseError<'tokens, 'src>>
       + Clone {
    block()
        .map_with_span(|block, span| (block, span))
        .then_ignore(end())
}

/// Block := (Statement{...})*
fn block<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParseError<'tokens, 'src>> + Clone
{
    recursive(|block| {
        statement(block)
            .map_with_span(|stmt, span| (stmt, span))
            .repeated()
            .collect()
            .map(Block)
    })
}

/// Statement := Control{...} | Function{...} | Variable{...}
///
/// ```txt
/// Control{...}  starts with ( [If] | [For] | [While] | [Return] | [Continue] | [Break] )
/// Function{...} starts with ( [Func] | <Callable> )
/// Variable{...} starts with ( [Var] | [Let] | <Local> )
/// ```
fn statement<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParseError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Statement<'src>, ParseError<'tokens, 'src>> + Clone
{
    let control_statement = control_statement(block.clone()).map(Statement::Control);
    let function_statement = function_statement(block.clone()).map(Statement::Function);
    let variable_statement = variable_statement(block).map(Statement::Variable);

    control_statement
        .or(function_statement)
        .or(variable_statement)
}

/// FunctionStatement := Define{...} | Call{...}
///
/// ```txt
/// Define{...} : [Func] <Local> [OpenParen] <Expression> ([Comma] <Expression>)* [Comma]? [CloseParen] <Block> [End]
/// Call{...}   : <Callable> [OpenParen] <Expression> ([Comma] <Expression>)* [Comma]? [CloseParen]
/// ```
fn function_statement<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParseError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    FunctionStatement<'src>,
    ParseError<'tokens, 'src>,
> + Clone {
    let define = just(Token::Func)
        .ignore_then(local().map_with_span(|local, span| (local, span)))
        .then(function_arguments_parser(expression(block.clone())))
        .then(block.clone().map_with_span(|block, span| (block, span)))
        .then_ignore(just(Token::End))
        .map(|((name, args), body)| FunctionStatement::Define { name, args, body });

    let call = callable(block.clone(), expression(block.clone()))
        .map_with_span(|local, span| (local, span))
        .then(function_arguments_parser(expression(block)))
        .map(|(name, args)| FunctionStatement::Call { name, args });

    define.or(call)
}

/// VariableStatement := Var{...} | Let{...} | Assign{...}
///
/// ```txt
/// Var{...}    : [Var] [Identifier] [Assign] <Expression>
/// Let{...}    : [Let] [Identifier] [Assign] <Expression>
/// Assign{...} : <Local> [Assign] <Expression>
/// ```
fn variable_statement<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParseError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    VariableStatement<'src>,
    ParseError<'tokens, 'src>,
> + Clone {
    let spanned_local = local().map_with_span(|local, span| (local, span));
    let spanned_expression = expression(block.clone()).map_with_span(|expr, span| (expr, span));

    let var = just(Token::Var)
        .ignore_then(spanned_local.clone())
        .then_ignore(just(Token::Assign))
        .then(spanned_expression.clone());

    let let_ = just(Token::Let)
        .ignore_then(spanned_local.clone())
        .then_ignore(just(Token::Assign))
        .then(spanned_expression.clone());

    let assign = spanned_local
        .then_ignore(just(Token::Assign))
        .then(spanned_expression);

    choice((
        var.map(|(lhs, rhs)| VariableStatement::Var { lhs, rhs }),
        let_.map(|(lhs, rhs)| VariableStatement::Let { lhs, rhs }),
        assign.map(|(lhs, rhs)| VariableStatement::Assign { lhs, rhs }),
    ))
}

/// ControlStatement := If{...} | For{...} || While{...} | Return(...) | Continue | Break
///
/// ```txt
/// If{...}     : [If] <Expression> [Then] <Block> ([Elif] <Expression> [Then] <Block>)* ([Else] <Block>)? [End]
/// For{...}    : [For] <Local> [Assign] <Expression> [Comma] <Expression> ([Comma] <Expression>)? [Do] <Block> [End]
/// ForIn{...}  : [For] <Loca> [In] <Expression> [Do] <Block> [End]
/// While{...}  : [While] <Expression> [Do] <Block> [End]
/// Return(...) : [Return] <Expression>?
/// Continue    : [Continue]
/// Break       : [Break]
/// ```
fn control_statement<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParseError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    ControlStatement<'src>,
    ParseError<'tokens, 'src>,
> + Clone {
    let expr = expression(block.clone());
    let cond = expr.clone().map_with_span(|expr, span| (expr, span));
    let body = block.map_with_span(|block, span| (block, span));

    let if_ = {
        let elif = just(Token::Elif)
            .ignore_then(cond.clone())
            .then_ignore(just(Token::Then))
            .then(body.clone());
        let else_ = just(Token::Else).ignore_then(body.clone());

        just(Token::If)
            .ignore_then(cond.clone())
            .then_ignore(just(Token::Then))
            .then(body.clone())
            .then(elif.repeated().collect())
            .then(else_.or_not())
            .then_ignore(just(Token::End))
    };

    let for_in = just(Token::For)
        .ignore_then(local().map_with_span(|ident, span| (ident, span)))
        .then_ignore(just(Token::In))
        .then(expr.clone().map_with_span(|expr, span| (expr, span)))
        .then_ignore(just(Token::Do))
        .then(body.clone())
        .then_ignore(just(Token::End));

    let for_ = just(Token::For)
        .ignore_then(local().map_with_span(|ident, span| (ident, span)))
        .then_ignore(just(Token::Assign))
        .then(expr.clone().map_with_span(|expr, span| (expr, span)))
        .then_ignore(just(Token::Comma))
        .then(expr.clone().map_with_span(|expr, span| (expr, span)))
        .then(
            just(Token::Comma)
                .ignore_then(expr.clone().map_with_span(|expr, span| (expr, span)))
                .or_not(),
        )
        .then_ignore(just(Token::Do))
        .then(body.clone())
        .then_ignore(just(Token::End));

    let while_ = just(Token::While)
        .ignore_then(cond)
        .then_ignore(just(Token::Do))
        .then(body)
        .then_ignore(just(Token::End));

    let return_ =
        just(Token::Return).ignore_then(expr.map_with_span(|expr, span| (expr, span)).or_not());

    choice((
        if_.map(|(((cond, body), elif), else_)| ControlStatement::If {
            cond,
            body,
            elifs: elif,
            else_,
        }),
        for_.map(
            |((((value, start), end), step), body)| ControlStatement::For {
                value,
                start,
                end,
                step,
                body,
            },
        ),
        for_in.map(|((value, iter), body)| ControlStatement::ForIn { value, iter, body }),
        while_.map(|(cond, body)| ControlStatement::While { cond, body }),
        return_.map(ControlStatement::Return),
        select! {
            Token::Continue => ControlStatement::Continue,
            Token::Break => ControlStatement::Break
        },
    ))
}

/// Expression := Binary_or_Unary |  Primitive(...) | Table(...) | AnonymousFunc{...} | FunctionCall{...} | Local
///
/// ```txt
/// Binary_or_Unary    : ...
/// Primitive(...)     : <Primitive>
/// Table(...)         : [OpenBrace] ([Identifier] [Assign] [Expression] [Comma])* ([Identifier] [Assign] [Expression])? [CloseBrace]
/// AnonymousFunc{...} : [Func] [OpenParen] <Expression> ([Comma] <Expression>)* [Comma]? [CloseParen] <Block> [End]
/// FunctionCall{...}  : <Callable> [OpenParen] <Expression> ([Comma] <Expression>)* [Comma]? [CloseParen]
/// ```
fn expression<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParseError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParseError<'tokens, 'src>> + Clone
{
    recursive(|expr| {
        let table = {
            let key = ident().map_with_span(|ident, span| (ident, span));
            let value = expr.clone().map_with_span(|expr, span| (expr, span));

            just(Token::OpenBrace)
                .ignore_then(
                    key.then_ignore(just(Token::Assign))
                        .then(value)
                        .separated_by(just(Token::Comma))
                        .allow_trailing()
                        .collect(),
                )
                .then_ignore(just(Token::CloseBrace))
        };

        let anonymous_func = just(Token::Func)
            .ignore_then(function_arguments_parser(expr.clone()))
            .then(block.clone().map_with_span(|block, span| (block, span)))
            .then_ignore(just(Token::End));

        let function_call = callable(block.clone(), expr.clone())
            .map_with_span(|ident, span| (ident, span))
            .then(function_arguments_parser(expr.clone()));

        let unary_or_binary_expr = {
            let term = choice((
                primitive().map(Expression::Primitive),
                table.clone().map(Expression::Table),
                function_call
                    .clone()
                    .map(|(func, args)| Expression::FunctionCall { func, args }),
                local().map(Expression::Local),
                expr.clone()
                    .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
            ));

            let unary_expr = {
                let unary_op = select! {
                    Token::Not => UnaryOp::Not,
                    Token::Sub => UnaryOp::Neg,
                };
                unary_op
                    .map_with_span(|op, span| (op, span))
                    .then(term.clone().map_with_span(|expr, span| (expr, span)))
                    .map(|((op, op_span), (expr, expr_span))| match (op, expr) {
                        (UnaryOp::Neg, Expression::Primitive(Primitive::Int(x))) => {
                            Expression::Primitive(Primitive::Int(-x))
                        }
                        (UnaryOp::Neg, Expression::Primitive(Primitive::Float(x))) => {
                            Expression::Primitive(Primitive::Float(-x))
                        }
                        (op, expr) => Expression::Unary {
                            op: (op, op_span),
                            expr: (Box::new(expr), expr_span),
                        },
                    })
            };

            fn create_binary_parser<'tokens, 'src: 'tokens>(
                term: impl Parser<
                        'tokens,
                        ParserInput<'tokens, 'src>,
                        (Expression<'src>, Span),
                        ParseError<'tokens, 'src>,
                    > + Clone,
                op: impl Parser<'tokens, ParserInput<'tokens, 'src>, BinaryOp, ParseError<'tokens, 'src>>
                    + Clone,
            ) -> impl Parser<
                'tokens,
                ParserInput<'tokens, 'src>,
                (Expression<'src>, Span),
                ParseError<'tokens, 'src>,
            > + Clone {
                term.clone().foldl(
                    op.map_with_span(|op, span| (op, span))
                        .then(term)
                        .repeated(),
                    |(lhs, lhs_span), (op, (rhs, rhs_span))| {
                        (
                            Expression::Binary {
                                op,
                                lhs: (Box::new(lhs), lhs_span),
                                rhs: (Box::new(rhs), rhs_span),
                            },
                            (lhs_span.start..rhs_span.end).into(),
                        )
                    },
                )
            }

            let unary_or_term_expr = unary_expr.or(term).map_with_span(|expr, span| (expr, span));

            // priority: 1
            let b_dot_expr = create_binary_parser(
                unary_or_term_expr, /*format*/
                select! { Token::Dot => BinaryOp::Dot, },
            )
            .boxed(); // to sppedup rust's compile time

            // priority: 2
            let b_pow_expr = create_binary_parser(
                b_dot_expr, /*format*/
                select! { Token::Pow => BinaryOp::Pow },
            );

            // priority: 3
            let binary_product_expr = create_binary_parser(
                b_pow_expr,
                select! {
                    Token::Mul => BinaryOp::Mul,
                    Token::Div => BinaryOp::Div,
                    Token::Mod => BinaryOp::Mod
                },
            );

            // priority: 4
            let b_sum_expr = create_binary_parser(
                binary_product_expr,
                select! {
                    Token::Add => BinaryOp::Add,
                    Token::Sub => BinaryOp::Sub
                },
            )
            .boxed(); // to sppedup rust's compile time

            // priority: 5
            let b_compare1_expr = create_binary_parser(
                b_sum_expr,
                select! {
                    Token::Less => BinaryOp::Less,
                    Token::LessEq => BinaryOp::LessEq,
                    Token::Greater => BinaryOp::Greater,
                    Token::GreaterEq => BinaryOp::GreaterEq,
                },
            );

            // priority: 6
            let b_compare2_expr = create_binary_parser(
                b_compare1_expr,
                select! {
                    Token::Eq => BinaryOp::Eq,
                    Token::NotEq => BinaryOp::NotEq
                },
            );

            // priority: 7
            let b_and_expr = create_binary_parser(
                b_compare2_expr, /*format*/
                select! { Token::And => BinaryOp::And },
            );

            // priority: 8
            let b_or_expr = create_binary_parser(
                b_and_expr, /*format*/
                select! { Token::Or => BinaryOp::Or },
            );

            b_or_expr.map(|(expr, _)| expr)
        };

        choice((
            unary_or_binary_expr,
            primitive().map(Expression::Primitive),
            table.map(Expression::Table),
            anonymous_func.map(|(args, program)| Expression::AnonymousFunc {
                args,
                body: program,
            }),
            function_call.map(|(func, args)| Expression::FunctionCall { func, args }),
            local().map(Expression::Local),
        ))
    })
}

/// Callable := Local | AnonymousFunc{...} | FunctionCall{...}
///
/// ```txt
/// Local              : <Local>
/// AnonymousFunc{...} : [OpenParen] [Func] [OpenParen] <Expression> ([Comma] <Expression>)* [Comma]? [CloseParen] <Block> [End] [CloseParen]
/// FunctionCall{...}  : [OpenParen] <Callable> [OpenParen] <Expression> ([Comma] <Expression>)* [Comma]? [CloseParen] [CloseParen]
/// ```
fn callable<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParseError<'tokens, 'src>>
        + Clone
        + 'tokens,
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParseError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Callable<'src>, ParseError<'tokens, 'src>> + Clone
{
    let anonymous_func = just(Token::Func)
        .ignore_then(function_arguments_parser(expression.clone()))
        .then(block.clone().map_with_span(|block, span| (block, span)))
        .then_ignore(just(Token::End))
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen));

    recursive(|callable| {
        let function_call = callable
            .map_with_span(|callable, span| (Box::new(callable), span))
            .then(function_arguments_parser(expression))
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen));

        choice((
            local().map(Callable::Local),
            anonymous_func
                .clone()
                .map(|(args, program)| Callable::AnonymousFunc {
                    args,
                    body: program,
                }),
            function_call.map(|(func, args)| Callable::FunctionCall { func, args }),
        ))
    })
}

/// Local := TableField{...} | Variable(str)
///
/// ```txt
/// TableField{...} : [Identifier] ([Dot] [Identifier])+
/// Variable        : [Identifier]
/// ```
fn local<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Local<'src>, ParseError<'tokens, 'src>> + Clone
{
    let spanned_ident = ident().map_with_span(|ident, span| (ident, span));

    let variable = ident().map(Local::Variable);
    let table = spanned_ident
        .clone()
        .then(
            just(Token::Dot)
                .ignore_then(spanned_ident)
                .repeated()
                .at_least(1)
                .collect(),
        )
        .map(|(table, keys)| Local::TableField { table, keys });

    table.or(variable)
}

/// Primitive := Int(i64) | Float(f64) | String(str) | Bool(bool) | Nil
fn primitive<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Primitive<'src>, ParseError<'tokens, 'src>> + Clone
{
    select! {
        Token::Int(x) => Primitive::Int(x),
        Token::Float(x) => Primitive::Float(x),
        Token::String(x) => Primitive::String(x),
        Token::Bool(x) => Primitive::Bool(x),
        Token::Nil => Primitive::Nil,
    }
}

fn ident<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, &'src str, ParseError<'tokens, 'src>> + Clone
{
    select! {
        Token::Identifier(x) => x,
    }
}

/// ```txt
/// [OpenParen] <Expression> ([Comma] <Expression>)* [Comma]? [CloseParen]
/// ```
fn function_arguments_parser<'tokens, 'src: 'tokens>(
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParseError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    (Vec<(Expression<'src>, Span)>, Span),
    ParseError<'tokens, 'src>,
> + Clone {
    expression
        .map_with_span(|expr, span| (expr, span))
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .map_with_span(|args, span| (args, span))
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen))
}
