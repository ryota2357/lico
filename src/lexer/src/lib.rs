use foundation::*;

mod error;
mod lexer;
mod tokenize;

use lexer::Lexer;
use tokenize::tokenize;

pub use error::Error;

pub fn parse(source: &str) -> (Vec<(Token, TextSpan)>, Vec<Error>) {
    let mut lexer = Lexer::new(source);
    tokenize(&mut lexer);
    lexer.into_tokens_errors()
}
