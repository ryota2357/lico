use chumsky::prelude::*;

mod lexer;
use lexer::lexer;

mod token;
pub use token::Token;

pub mod error;

type Span = chumsky::span::SimpleSpan<usize>;

pub fn parse(src: &str) -> (Vec<(Token, Span)>, Vec<error::Error>) {
    let result = lexer().parse(src);
    let (output, errors) = result.into_output_errors();
    let tokens = output.unwrap_or(Vec::new());
    (tokens, errors)
}
