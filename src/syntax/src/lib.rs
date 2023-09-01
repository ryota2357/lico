pub mod lex;

use chumsky::{span::SimpleSpan, Parser};

type Span = SimpleSpan<usize>;

/// Parse a string into an Token list
pub fn parse_src_to_token(src: &str) -> Vec<(lex::Token<'_>, Span)> {
    lex::lexer().parse(src).unwrap()
}
