mod lexer;
mod token;

use chumsky::prelude::*;

pub use token::Token;

pub fn parse(src: &str) -> Vec<(Token<'_>, chumsky::span::SimpleSpan<usize>)> {
    lexer::lexer().parse(src).unwrap()
}
