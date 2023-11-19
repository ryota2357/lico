use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum VariableStatement<'src> {
    Var {
        name: (Ident<'src>, Span),
        expr: (Expression<'src>, Span),
    },
    Func {
        name: (Ident<'src>, Span),
        args: Vec<(Ident<'src>, Span)>,
        body: Chunk<'src>,
    },
    FieldFunc {
        table: (Ident<'src>, Span),
        fields: Vec<(Ident<'src>, Span)>,
        args: Vec<(Ident<'src>, Span)>,
        body: Chunk<'src>,
    },
    Assign {
        name: (Ident<'src>, Span),
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
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'src>>
        + Clone
        + 'tokens,
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, (Expression<'src>, Span), ParserError<'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, VariableStatement<'src>, ParserError<'src>> + Clone
{
    let func_arguments = spanned_ident()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen));

    let var = just(Token::Var)
        .ignore_then(spanned_ident())
        .then_ignore(just(Token::Assign))
        .then(expression.clone())
        .map(|(name, expr)| VariableStatement::Var { name, expr });
    let func = just(Token::Func)
        .ignore_then(spanned_ident())
        .then(func_arguments.clone())
        .then(block.clone())
        .then_ignore(just(Token::End))
        .map(|((name, args), block)| VariableStatement::Func {
            name,
            args,
            body: block.into(),
        });
    let field_func = just(Token::Func)
        .ignore_then(spanned_ident())
        .then(
            just(Token::Dot)
                .ignore_then(spanned_ident())
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
    let assign = spanned_ident()
        .then(
            choice((
                expression
                    .clone()
                    .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket)),
                just(Token::Dot)
                    .ignore_then(ident())
                    .map_with(|ident, extra| (Expression::Ident(ident), extra.span().into())),
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

impl<'a> TreeWalker<'a> for VariableStatement<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            VariableStatement::Var {
                name: (name, _),
                expr: (expr, _),
            } => {
                tracker.add_definition(name);
                expr.analyze(tracker);
            }
            VariableStatement::Func {
                name: (name, _),
                args,
                body,
            } => {
                tracker.add_definition(name);
                tracker.push_new_definition_scope();
                for (arg, _) in args.iter() {
                    tracker.add_definition(arg);
                }
                body.analyze(tracker);
                tracker.pop_current_definition_scope();
            }
            VariableStatement::FieldFunc {
                table: (table, _),
                fields,
                args,
                body,
            } => {
                tracker.add_capture(table);
                for (field, _) in fields.iter() {
                    tracker.add_definition(field);
                }
                tracker.push_new_definition_scope();
                for (arg, _) in args.iter() {
                    tracker.add_definition(arg);
                }
                body.analyze(tracker);
                tracker.pop_current_definition_scope();
            }
            VariableStatement::Assign {
                name: (name, _),
                accesser,
                expr: (expr, _),
            } => {
                tracker.add_capture(name);
                for (access, _) in accesser.iter_mut() {
                    access.analyze(tracker);
                }
                expr.analyze(tracker);
            }
        }
    }
}
