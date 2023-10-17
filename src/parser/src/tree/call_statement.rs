use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum CallStatement<'src> {
    Invoke {
        expr: Expression<'src>,
        args: Vec<Expression<'src>>,
    },
    CallMethod {
        expr: Expression<'src>,
        name: Ident<'src>,
        args: Vec<Expression<'src>>,
    },
}

pub(super) fn call_statement<'tokens, 'src: 'tokens>(
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, CallStatement<'src>, ParserError<'tokens, 'src>>
       + Clone {
    let expr_args_multi = expression
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen))
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>();
    let ident_or_expr = {
        let ident = ident().map(|ident| Expression::Local(Local::Ident(ident)));
        let delimited_expr = expression
            .clone()
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen));
        ident.or(delimited_expr)
    };

    let invoke = ident_or_expr
        .clone()
        .then(expr_args_multi.clone())
        .map(|(expr, mut argslist)| {
            // let last_args = unsafe { argslist.pop().unwrap_unchecked() };
            let last_args = argslist.pop().unwrap();
            let expr = argslist
                .into_iter()
                .fold(expr, |expr, args| Expression::Invoke {
                    expr: Box::new(expr),
                    args,
                });
            CallStatement::Invoke {
                expr,
                args: last_args,
            }
        });
    let method = ident_or_expr
        .then_ignore(just(Token::Arrow))
        .then(ident())
        .then(expr_args_multi)
        .map(|((expr, name), mut argslist)| {
            // let last_args = unsafe { argslist.pop().unwrap_unchecked() };
            let last_args = argslist.pop().unwrap();
            let expr = argslist
                .into_iter()
                .fold(expr, |expr, args| Expression::Invoke {
                    expr: Box::new(expr),
                    args,
                });
            CallStatement::CallMethod {
                expr,
                name,
                args: last_args,
            }
        });
    invoke.or(method)
}

impl<'a> TreeWalker<'a> for CallStatement<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            CallStatement::Invoke { expr, args } => {
                expr.analyze(tracker);
                for arg in args {
                    arg.analyze(tracker);
                }
            }
            CallStatement::CallMethod { expr, args, .. } => {
                expr.analyze(tracker);
                for arg in args {
                    arg.analyze(tracker);
                }
            }
        }
    }
}
