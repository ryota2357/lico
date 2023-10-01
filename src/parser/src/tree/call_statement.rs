use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum CallStatement<'src> {
    Do { body: Block<'src> },
    Call(Call<'src>),
}

/// <CallStatement> ::= <Do> | <Call>
/// <Do>            ::= 'do' <Block> 'end'
pub(super) fn call_statement<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, CallStatement<'src>, ParserError<'tokens, 'src>>
       + Clone {
    let r#do = just(Token::Do)
        .ignore_then(block.clone())
        .then_ignore(just(Token::End))
        .map(|body| CallStatement::Do { body });
    let call = call(block, expression).map(CallStatement::Call);

    r#do.or(call)
}

impl<'a> TreeWalker<'a> for CallStatement<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            CallStatement::Do { body } => {
                tracker.push_new_definition_scope();
                body.analyze(tracker);
                tracker.pop_current_definition_scope();
            }
            CallStatement::Call(call) => call.analyze(tracker),
        }
    }
}
