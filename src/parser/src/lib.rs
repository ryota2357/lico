pub mod tree;
use lexer::TextSpan;

#[derive(Clone, Debug, PartialEq)]
pub struct Program<'src> {
    pub attributes: Vec<(&'src str, Vec<TextSpan>)>,
    pub body: tree::Chunk<'src>,
}
