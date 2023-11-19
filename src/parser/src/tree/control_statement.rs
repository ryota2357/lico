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
        value: (Ident<'src>, Span),
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
        .ignore_then(spanned_ident())
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

impl<'a> TreeWalker<'a> for ControlStatement<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            ControlStatement::If {
                cond: (cond, _),
                body,
                elifs,
                else_,
            } => {
                cond.analyze(tracker);

                tracker.push_new_definition_scope();
                body.analyze(tracker);
                tracker.pop_current_definition_scope();

                for ((cond, _), body) in elifs.iter_mut() {
                    cond.analyze(tracker);
                    tracker.push_new_definition_scope();
                    body.analyze(tracker);
                    tracker.pop_current_definition_scope();
                }

                if let Some(body) = else_ {
                    tracker.push_new_definition_scope();
                    body.analyze(tracker);
                    tracker.pop_current_definition_scope();
                }
            }
            ControlStatement::For {
                value: (value, _),
                iter: (iter, _),
                body,
            } => {
                iter.analyze(tracker);

                tracker.push_new_definition_scope();
                tracker.add_definition(value);
                body.analyze(tracker);
                tracker.pop_current_definition_scope();
            }
            ControlStatement::While {
                cond: (cond, _),
                body,
            } => {
                cond.analyze(tracker);
                tracker.push_new_definition_scope();
                body.analyze(tracker);
                tracker.pop_current_definition_scope();
            }
            ControlStatement::Do { body } => {
                tracker.push_new_definition_scope();
                body.analyze(tracker);
                tracker.pop_current_definition_scope();
            }
            ControlStatement::Return { value } => {
                if let Some((value, _)) = value {
                    value.analyze(tracker);
                }
            }
            ControlStatement::Continue => {}
            ControlStatement::Break => {}
        }
    }
}
