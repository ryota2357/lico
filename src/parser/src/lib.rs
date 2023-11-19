use chumsky::prelude::*;

type Span = std::ops::Range<usize>;

pub mod error;
pub mod tree;

pub fn parse<'tokens, 'src: 'tokens>(
    tokens: &'tokens [(lexer::Token<'src>, SimpleSpan<usize>)],
    eoi: std::ops::Range<usize>,
) -> (Option<tree::Program<'src>>, Vec<error::Error<'src>>) {
    let eoi = eoi.into();
    let tokens = tokens.spanned(eoi);

    let (mut program, errors) = tree::program().parse(tokens).into_output_errors();
    if errors.is_empty() {
        if let Some(ref mut program) = program {
            tree::analyze_tree(program);
        }
    }
    (program, errors)
}
