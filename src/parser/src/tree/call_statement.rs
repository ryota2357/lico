use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum CallStatement<'src> {
    Invoke {
        expr: (Expression<'src>, Span),
        args: Vec<(Expression<'src>, Span)>,
    },
    MethodCall {
        expr: (Expression<'src>, Span),
        name: Ident<'src>,
        args: Vec<(Expression<'src>, Span)>,
    },
}

pub(super) fn call_statement<'tokens, 'src: 'tokens>(
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, (Expression<'src>, Span), ParserError<'src>>
        + Clone,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, CallStatement<'src>, ParserError<'src>> + Clone
{
    let ident_or_expr = {
        let ident = ident().map(Expression::Ident);
        let primitive = primitive().map(Expression::Primitive);
        let tabel_obj = table_object(expression.clone()).map(Expression::TableObject);
        let arrya_obj = array_object(expression.clone()).map(Expression::ArrayObject);
        let delimited_expr = expression
            .clone()
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen));
        choice((ident, primitive, tabel_obj, arrya_obj))
            .map_with(|expr, extra| (expr, extra.span().into()))
            .or(delimited_expr)
    };
    let trigger = {
        let expr_args = expression
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen));
        let invoke = expr_args
            .clone()
            .map_with(|args, extra| (None, args, extra.span()));
        let method = just(Token::Arrow)
            .ignore_then(ident())
            .then(expr_args)
            .map_with(|(name, args), extra| (Some(name), args, extra.span()));
        invoke.or(method)
    };

    ident_or_expr
        .then(trigger.repeated().at_least(1).collect::<Vec<_>>())
        .map_with(|(expr, mut triggers), _extra| {
            // SAFETY: `repeated().at_least(1)` ensures that `triggers` is not empty.
            let (name, args, _) = unsafe { triggers.pop().unwrap_unchecked() };
            let expr = triggers
                .into_iter()
                .fold(expr, |(expr, expr_span), (name, args, span)| {
                    let span = expr_span.start..span.end;
                    let expr = (Box::new(expr), expr_span);
                    match name {
                        Some(name) => (Expression::MethodCall { expr, name, args }, span),
                        None => (Expression::Invoke { expr, args }, span),
                    }
                });
            match name {
                Some(name) => CallStatement::MethodCall { expr, name, args },
                None => CallStatement::Invoke { expr, args },
            }
        })
}

impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for CallStatement<'src> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
        match self {
            CallStatement::Invoke {
                expr: (expr, _),
                args,
            } => {
                walker.go(expr);
                for (expr, _) in args {
                    walker.go(expr);
                }
            }
            CallStatement::MethodCall {
                expr: (expr, _),
                name: _,
                args,
            } => {
                walker.go(expr);
                for (expr, _) in args {
                    walker.go(expr);
                }
            }
        }
    }
}
