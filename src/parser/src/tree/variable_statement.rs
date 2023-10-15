use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum VariableStatement<'src> {
    Var {
        name: Ident<'src>,
        expr: Expression<'src>,
    },
    Let {
        name: Ident<'src>,
        expr: Expression<'src>,
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
        lhs: Local<'src>,
        rhs: Expression<'src>,
    },
}

/// <VariableStatement> ::= <Var> | <Let> | <Func> | <Assign>
/// <Var>               ::= 'var' <Ident> '=' <Expression>
/// <Let>               ::= 'let' <Ident> '=' <Expression>
/// <Func>              ::= 'func' <Local> '(' [ <Ident> { ',' <Ident> } [ ',' ] ] ')' <Block> 'end'
/// <Assign>            ::= <Local> '=' <Expression>
pub(super) fn variable_statement<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    VariableStatement<'src>,
    ParserError<'tokens, 'src>,
> + Clone {
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
    let r#let = just(Token::Let)
        .ignore_then(ident())
        .then_ignore(just(Token::Assign))
        .then(expression.clone())
        .map(|(name, expr)| VariableStatement::Let { name, expr });
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
    let assign = local()
        .then_ignore(just(Token::Assign))
        .then(expression)
        .map(|(lhs, rhs)| VariableStatement::Assign { lhs, rhs });

    choice((var, r#let, func, field_func, assign))
}

impl<'a> TreeWalker<'a> for VariableStatement<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            VariableStatement::Var { name, expr } => {
                tracker.add_definition(name.str);
                expr.analyze(tracker);
            }
            VariableStatement::Let { name, expr } => {
                tracker.add_definition(name.str);
                expr.analyze(tracker);
            }
            VariableStatement::Func { name, args, body } => {
                tracker.add_definition(name.str);
                tracker.push_new_definition_scope();
                for arg in args.iter() {
                    tracker.add_definition(arg.str);
                }
                body.analyze(tracker);
                tracker.pop_current_definition_scope();
            }
            VariableStatement::FieldFunc {
                table,
                fields,
                args,
                body,
            } => {
                tracker.add_capture(table.str);
                for field in fields.iter() {
                    tracker.add_definition(field.str);
                }
                tracker.push_new_definition_scope();
                for arg in args.iter() {
                    tracker.add_definition(arg.str);
                }
                body.analyze(tracker);
                tracker.pop_current_definition_scope();
            }
            VariableStatement::Assign { lhs, rhs } => {
                match lhs {
                    Local::TableField { name, .. } => tracker.add_capture(name.str),
                    Local::Ident(ident) => tracker.add_capture(ident.str),
                }
                rhs.analyze(tracker);
            }
        }
    }
}
