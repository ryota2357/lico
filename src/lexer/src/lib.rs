#![allow(dead_code)]

mod token;
pub use token::Token;

mod textspan;
pub use textspan::TextSpan;

mod lexer;
use lexer::Lexer;

mod tokenize;
use tokenize::tokenize;

pub mod error;
use error::Error;

pub fn parse(source: &str) -> (Vec<(Token, TextSpan)>, Vec<Error>) {
    let mut lexer = Lexer::new(source);
    tokenize(&mut lexer);
    lexer.into_tokens_errors()
}
