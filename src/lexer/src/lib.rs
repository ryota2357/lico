use foundation::*;

mod lexer;
use lexer::Lexer;

mod tokenize;
use tokenize::tokenize;

mod error;
pub use error::Error;

pub fn parse(source: &str) -> (Vec<(Token, TextSpan)>, Vec<Error>) {
    let mut lexer = Lexer::new(source);
    tokenize(&mut lexer);
    lexer.into_tokens_errors()
}
