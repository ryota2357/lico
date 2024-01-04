pub mod tree;

mod error;
mod parser;
mod walker;

use error::Error;
use parser::Parser;
use tree::*;

use lexer::{TextSpan, Token};

pub fn parse<'tokens, 'src: 'tokens>(
    tokens: &'tokens [(Token<'src>, TextSpan)],
) -> (Program<'src>, Vec<Error>) {
    let (mut program, errors) = Parser::parse(tokens);

    let mut walker = walker::Walker::new();
    walker.go(&mut program.body.block);
    let mut result = walker.finish();
    program.body.captures = result.captures();
    program.attributes = result.take_attributes();

    (program, errors)
}
