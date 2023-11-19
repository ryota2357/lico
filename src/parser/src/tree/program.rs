use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Program<'src> {
    pub attributes: Vec<(&'src str, Vec<Span>)>,
    pub body: Chunk<'src>,
}

/// <Program> ::= <Chunk>
pub(crate) fn program<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Program<'src>, ParserError<'src>> + Clone {
    block().then_ignore(end()).map(|block| Program {
        attributes: vec![],
        body: block.into(),
    })
}

impl<'a> TreeWalker<'a> for Program<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        self.body.analyze(tracker);
        self.attributes = tracker.get_all_attributes();
    }
}
