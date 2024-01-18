use foundation::{ast::*, TextSpan, Token};

mod error;
mod parser;
mod walker;

pub use error::Error;

pub fn parse<'tokens, 'src: 'tokens>(
    tokens: &'tokens [(Token<'src>, TextSpan)],
) -> (Program<'src>, Vec<Error>) {
    let (mut program, errors) = parser::parse(tokens);

    let mut walker = walker::Walker::new();
    walker.go(&mut program.body.block);
    let mut result = walker.finish();
    program.body.captures = result.captures();
    program.attributes = result.take_attributes();

    (program, errors)
}
