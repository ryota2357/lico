pub mod lex;
pub mod tree;

use chumsky::{input::Input, span::SimpleSpan, Parser};

type Span = SimpleSpan<usize>;

/// Parse a string into an Token list
pub fn parse_src_to_token(src: &str) -> Vec<(lex::Token<'_>, Span)> {
    lex::lexer().parse(src).unwrap()
}

/// Parse a Token list into a syntax tree
pub fn parse_token_to_syntax_tree<'tokens, 'src: 'tokens>(
    tokens: &'tokens [(lex::Token<'src>, Span)],
    eoi: std::ops::Range<usize>,
) -> (tree::node::Block<'src>, Span) {
    tree::parse::parser()
        .parse(tokens.spanned(eoi.into()))
        .unwrap()
}
