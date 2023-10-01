use crate::tree::*;
use chumsky::prelude::*;
use lexer::Token;

type Span = SimpleSpan<usize>;

pub(crate) fn parse_token<'tokens, 'src: 'tokens>(
    tokens: &'tokens [(Token<'src>, Span)],
    eoi: Span,
) -> (Option<Program<'src>>, Vec<Rich<'tokens, Token<'src>, Span>>) {
    let (mut program, errors) = program().parse(tokens.spanned(eoi)).into_output_errors();
    if errors.is_empty() {
        if let Some(ref mut program) = program {
            analyze_tree(program);
        }
    }
    (program, errors)
}
