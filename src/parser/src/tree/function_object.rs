use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionObject<'src> {
    pub args: Vec<(Ident<'src>, Span)>,
    pub body: Chunk<'src>,
}

/// <FunctionObject> ::= 'func' '(' [ <Ident> { ',' <Ident> } [ ',' ] ] ')' <Chunk> 'end'
pub(super) fn function_object<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, FunctionObject<'src>, ParserError<'tokens, 'src>>
       + Clone {
    let args = spanned_ident()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect();
    just(Token::Func)
        .ignore_then(args.delimited_by(just(Token::OpenParen), just(Token::CloseParen)))
        .then(block)
        .then_ignore(just(Token::End))
        .map(|(args, block)| FunctionObject {
            args,
            body: block.into(),
        })
}

impl<'a> TreeWalker<'a> for FunctionObject<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        tracker.push_new_definition_scope();
        for (arg, _) in self.args.iter_mut() {
            tracker.add_definition(arg);
        }
        self.body.analyze(tracker);
        tracker.pop_current_definition_scope();
    }
}
