pub mod error;
mod lexer;
mod token;

use chumsky::prelude::*;

pub use token::Token;

pub fn parse(
    src: &str,
) -> (
    Vec<(Token, chumsky::span::SimpleSpan<usize>)>,
    Vec<error::Error>,
) {
    let result = lexer::lexer().parse(src);
    let (tokens, errors) = result.into_output_errors();
    (tokens.unwrap_or(Vec::new()), errors)
}
