use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ControlStatement<'src> {
    If {
        cond: (Expression<'src>, Span),
        body: Block<'src>,
        elifs: Vec<((Expression<'src>, Span), Block<'src>)>,
        else_: Option<Block<'src>>,
    },
    For {
        value: Ident<'src>,
        iter: (Expression<'src>, Span),
        body: Block<'src>,
    },
    While {
        cond: (Expression<'src>, Span),
        body: Block<'src>,
    },
    Do {
        body: Block<'src>,
    },
    Return {
        value: Option<(Expression<'src>, Span)>,
    },
    Continue,
    Break,
}

/// <ControlStatement> ::= <If> | <For> | <ForIn> | <While> | <Return> | <Continue> | <Break>
/// <If>               ::= 'if' <Expression> 'then' <Block> { 'elif' <Expression> 'then' <Block> } [ 'else' <Block> ] 'end'
/// <ForIn>            ::= 'for' <Local> 'in' <Expression> 'do' <Block> 'end'
/// <While>            ::= 'while' <Expression> 'do' <Block> 'end'
/// <Return>           ::= 'return' [ <Expression> ]
/// <Continue>         ::= 'continue'
/// <Break>            ::= 'break'
pub(super) fn control_statement<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'src>> + Clone,
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, (Expression<'src>, Span), ParserError<'src>>
        + Clone,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, ControlStatement<'src>, ParserError<'src>> + Clone
{
    let r#if = just(Token::If)
        .ignore_then(expression.clone())
        .then_ignore(just(Token::Then))
        .then(block.clone())
        .then(
            just(Token::Elif)
                .ignore_then(expression.clone())
                .then_ignore(just(Token::Then))
                .then(block.clone())
                .repeated()
                .collect(),
        )
        .then(just(Token::Else).ignore_then(block.clone()).or_not())
        .then_ignore(just(Token::End))
        .map(|(((cond, body), elifs), else_)| ControlStatement::If {
            cond,
            body,
            elifs,
            else_,
        });
    let r#for = just(Token::For)
        .ignore_then(ident())
        .then_ignore(just(Token::In))
        .then(expression.clone())
        .then_ignore(just(Token::Do))
        .then(block.clone())
        .then_ignore(just(Token::End))
        .map(|((value, iter), body)| ControlStatement::For { value, iter, body });
    let r#while = just(Token::While)
        .ignore_then(expression.clone())
        .then_ignore(just(Token::Do))
        .then(block.clone())
        .then_ignore(just(Token::End))
        .map(|(cond, body)| ControlStatement::While { cond, body });
    let r#do = just(Token::Do)
        .ignore_then(block)
        .then_ignore(just(Token::End))
        .map(|body| ControlStatement::Do { body });
    let r#return = just(Token::Return)
        .ignore_then(expression.or_not())
        .map(|value| ControlStatement::Return { value });
    let r#continue = just(Token::Continue).to(ControlStatement::Continue);
    let r#break = just(Token::Break).to(ControlStatement::Break);

    choice((r#if, r#for, r#while, r#do, r#return, r#continue, r#break))
}

impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for ControlStatement<'src> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
        match self {
            ControlStatement::If {
                cond: (cond, _),
                body,
                elifs,
                else_,
            } => {
                walker.go(cond);
                walker.fork().go(body);
                for ((cond, _), body) in elifs {
                    walker.go(cond);
                    walker.fork().go(body);
                }
                if let Some(else_) = else_ {
                    walker.fork().go(else_);
                }
            }
            ControlStatement::For {
                value: Ident(value, _),
                iter: (iter, _),
                body,
            } => {
                walker.go(iter);
                walker.record_variable_definition(value);
                walker.fork().go(body);
            }
            ControlStatement::While {
                cond: (cond, _),
                body,
            } => {
                walker.go(cond);
                walker.fork().go(body);
            }
            ControlStatement::Do { body } => {
                walker.fork().go(body);
            }
            ControlStatement::Return { value } => {
                if let Some((value, _)) = value {
                    walker.go(value);
                }
            }
            ControlStatement::Continue => {}
            ControlStatement::Break => {}
        }
    }
}
