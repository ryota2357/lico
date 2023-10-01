use chumsky::span::SimpleSpan;

mod parser;
pub mod tree;

pub fn parse<'tokens, 'src: 'tokens>(
    tokens: &'tokens [(lexer::Token<'src>, SimpleSpan<usize>)],
    eoi: std::ops::Range<usize>,
) -> tree::Program<'src> {
    parser::parse_token(tokens, eoi.into()).0.unwrap()
}
