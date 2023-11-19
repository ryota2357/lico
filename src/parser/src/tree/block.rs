use super::*;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq)]
pub struct Block<'src>(pub Vec<(Statement<'src>, Span)>);

#[derive(Clone, Debug, PartialEq)]
pub struct Chunk<'src> {
    pub captures: Vec<&'src str>,
    pub block: Vec<(Statement<'src>, Span)>,
}

/// <Block> ::= { <Statement> }
/// <Chunk> ::= <Block>
pub(super) fn block<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'src>> + Clone {
    recursive(|block| statement(block).repeated().collect().map(Block))
}

impl<'a> From<Block<'a>> for Chunk<'a> {
    fn from(value: Block<'a>) -> Self {
        Self {
            captures: vec![],
            block: value.0,
        }
    }
}

impl<'a> Deref for Block<'a> {
    type Target = Vec<(Statement<'a>, Span)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> TreeWalker<'a> for Block<'a> {
    /// This function does not call `tracker.push_new_definition_scope` and `tracker.pop_current_definition_scope` internally.
    /// Therefore, you need to call them appropriately before and after `analyze` the `Block`.
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        for (statement, _) in self.0.iter_mut() {
            statement.analyze(tracker);
        }
    }
}

impl<'a> TreeWalker<'a> for Chunk<'a> {
    /// This function does not call `tracker.push_new_definition_scope()` and `tracker.pop_current_definition_scope()` internally.
    /// Therefore, you need to call them appropriately before and after `analyze` the `Block`.
    /// (But call `tracker.begin_new_capture_section()` and `tracker.end_current_capture_section()` internally.)
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        tracker.begin_new_capture_section();
        for (statement, _) in self.block.iter_mut() {
            statement.analyze(tracker);
        }
        self.captures = tracker.end_current_capture_section();
    }
}
