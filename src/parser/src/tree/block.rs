use super::*;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq)]
pub struct Block<'src>(pub Vec<(Statement<'src>, Span)>);

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk<'src> {
    pub captures: Vec<(&'src str, Span)>,
    pub block: Block<'src>,
}

/// <Block> ::= { <Statement> }
pub(super) fn block<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'src>> + Clone {
    recursive(|block| statement(block).repeated().collect().map(Block))
}

impl<'a> From<Block<'a>> for Chunk<'a> {
    fn from(value: Block<'a>) -> Self {
        Self {
            captures: vec![],
            block: value,
        }
    }
}

impl<'a> Deref for Block<'a> {
    type Target = Vec<(Statement<'a>, Span)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for Block<'src> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
        for (statement, _) in self.0.iter_mut() {
            walker.go(statement);
        }
    }
}
