use chumsky::prelude::*;

type Span = std::ops::Range<usize>;

pub mod error;
pub mod tree;

#[derive(Clone, Debug, PartialEq)]
pub struct Program<'src> {
    pub attributes: Vec<(&'src str, Vec<Span>)>,
    pub body: tree::Chunk<'src>,
}

pub fn parse<'tokens, 'src: 'tokens>(
    tokens: &'tokens [(lexer::Token<'src>, SimpleSpan<usize>)],
    eoi: std::ops::Range<usize>,
) -> (Option<Program<'src>>, Vec<error::Error<'src>>) {
    let eoi = eoi.into();
    let tokens = tokens.spanned(eoi);

    let (mut program, errors) = tree::program().parse(tokens).into_output_errors();
    if errors.is_empty() {
        if let Some(ref mut program) = program {
            let mut walker = tree::Walker::new();
            walker.go(&mut program.body.block);
            let result = walker.finish();
            program.body.captures = result.captures();
        }
    }
    (program, errors)
}
