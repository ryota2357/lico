use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum CallStatement<'src> {
    Invoke {
        expr: Expression<'src>,
        args: Vec<Expression<'src>>,
    },
    MethodCall {
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
    let ident_or_expr = {
        let ident = ident().map(Expression::Ident);
        let primitive = primitive().map(Expression::Primitive);
        let tabel_obj = table_object(expression.clone()).map(Expression::TableObject);
        let arrya_obj = array_object(expression.clone()).map(Expression::ArrayObject);
        let delimited_expr = expression
            .clone()
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen));
        choice((ident, primitive, tabel_obj, arrya_obj, delimited_expr))
    };
    let trigger = {
        let expr_args = expression
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen));
        let invoke = expr_args.clone().map(|args| (None, args));
        let method = just(Token::Arrow)
            .ignore_then(ident())
            .then(expr_args)
            .map(|(name, args)| (Some(name), args));
        invoke.or(method)
    };

    ident_or_expr
        .then(trigger.repeated().at_least(1).collect::<Vec<_>>())
        .map(|(expr, mut triggers)| {
            // let (name, args) = unsafe { triggers.pop().unwrap_unchecked() };
            let (name, args) = triggers.pop().unwrap();
            let expr = triggers
                .into_iter()
                .fold(expr, |expr, (name, args)| match name {
                    Some(name) => Expression::MethodCall {
                        expr: Box::new(expr),
                        name,
                        args,
                    },
                    None => Expression::Invoke {
                        expr: Box::new(expr),
                        args,
                    },
                });
            match name {
                Some(name) => CallStatement::MethodCall { expr, name, args },
                None => CallStatement::Invoke { expr, args },
            }
        })
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
            CallStatement::MethodCall { expr, args, .. } => {
                expr.analyze(tracker);
                for arg in args {
                    arg.analyze(tracker);
                }
            }
        }
    }
}
