use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum VariableStatement<'src> {
    Var {
        name: Ident<'src>,
        expr: (Expression<'src>, Span),
    },
    Func {
        name: Ident<'src>,
        args: Vec<Ident<'src>>,
        body: Chunk<'src>,
    },
    FieldFunc {
        table: Ident<'src>,
        fields: Vec<Ident<'src>>,
        args: Vec<Ident<'src>>,
        body: Chunk<'src>,
    },
    Assign {
        name: Ident<'src>,
        accesser: Vec<(Expression<'src>, Span)>,
        expr: (Expression<'src>, Span),
    },
}

/// <VariableStatement> ::= <Var> | <Let> | <Func> | <Assign>
/// <Var>               ::= 'var' <Ident> '=' <Expression>
/// <Let>               ::= 'let' <Ident> '=' <Expression>
/// <Func>              ::= 'func' <Local> '(' [ <Ident> { ',' <Ident> } [ ',' ] ] ')' <Block> 'end'
/// <Assign>            ::= <Ident> { ( '[' <Expression> ']' ) | ( '.' <Ident> ) } '=' <Expression>
pub(super) fn variable_statement<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'src>> + Clone,
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, (Expression<'src>, Span), ParserError<'src>>
        + Clone,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, VariableStatement<'src>, ParserError<'src>> + Clone
{
    let func_arguments = ident()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen));

    let var = just(Token::Var)
        .ignore_then(ident())
        .then_ignore(just(Token::Assign))
        .then(expression.clone())
        .map(|(name, expr)| VariableStatement::Var { name, expr });
    let func = just(Token::Func)
        .ignore_then(ident())
        .then(func_arguments.clone())
        .then(block.clone())
        .then_ignore(just(Token::End))
        .map(|((name, args), block)| VariableStatement::Func {
            name,
            args,
            body: block.into(),
        });
    let field_func = just(Token::Func)
        .ignore_then(ident())
        .then(
            just(Token::Dot)
                .ignore_then(ident())
                .repeated()
                .at_least(1)
                .collect(),
        )
        .then(func_arguments)
        .then(block)
        .then_ignore(just(Token::End))
        .map(
            |(((table, fields), args), block)| VariableStatement::FieldFunc {
                table,
                fields,
                args,
                body: block.into(),
            },
        );
    let assign = ident()
        .then(
            choice((
                expression
                    .clone()
                    .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket)),
                just(Token::Dot).ignore_then(select! { Token::Ident(x) => x }.map_with(
                    |ident, extra| {
                        let key = ident.to_string();
                        let span: SimpleSpan = extra.span();
                        (Expression::Primitive(Primitive::String(key)), span.into())
                    },
                )),
            ))
            .repeated()
            .collect(),
        )
        .then_ignore(just(Token::Assign))
        .then(expression)
        .map(|((name, accesser), expr)| VariableStatement::Assign {
            name,
            accesser,
            expr,
        });

    choice((var, func, field_func, assign))
}

impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for VariableStatement<'src> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
        match self {
            VariableStatement::Var {
                name: Ident(name, _),
                expr: (expr, _),
            } => {
                walker.record_variable_definition(name);
                walker.go(expr);
            }
            VariableStatement::Func {
                name: Ident(name, _),
                args,
                body,
            } => {
                walker.record_variable_definition(name);
                let result = {
                    let mut walker = Walker::new();
                    for Ident(arg, _) in args {
                        walker.record_variable_definition(arg);
                    }
                    walker.go(&mut body.block);
                    let result = walker.finish();
                    body.captures = result.captures();
                    result
                };
                walker.merge(result);
            }
            VariableStatement::FieldFunc {
                table: Ident(table, table_span),
                fields: _,
                args,
                body,
            } => {
                walker.record_variable_usage(table, table_span);
                let result = {
                    let mut walker = Walker::new();
                    for Ident(arg, _) in args {
                        walker.record_variable_definition(arg);
                    }
                    walker.go(&mut body.block);
                    let result = walker.finish();
                    body.captures = result.captures();
                    result
                };
                walker.merge(result);
            }
            VariableStatement::Assign {
                name: Ident(name, name_span),
                accesser,
                expr: (expr, _),
            } => {
                walker.go(expr);
                walker.record_variable_usage(name, name_span);
                for (expr, _) in accesser {
                    walker.go(expr);
                }
            }
        }
    }
}
