use super::*;
use chumsky::input::SpannedInput;
use lexer::Token;

#[derive(Clone, Debug, PartialEq)]
pub struct Error<'src> {
    pub kind: ErrorKind<'src>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind<'src> {
    ExpectedFound(Vec<Token<'src>>, Option<Token<'src>>),
}

impl<'src> Error<'src> {
    pub fn expected_found(
        expected: Vec<Token<'src>>,
        found: Option<Token<'src>>,
        span: Span,
    ) -> Self {
        Self {
            kind: ErrorKind::ExpectedFound(expected, found),
            span,
        }
    }
}

impl<'tokens, 'src: 'tokens>
    chumsky::error::Error<
        'tokens,
        SpannedInput<Token<'src>, SimpleSpan, &'tokens [(Token<'src>, SimpleSpan)]>,
    > for Error<'src>
{
    fn expected_found<E: IntoIterator<Item = Option<chumsky::util::MaybeRef<'tokens, <SpannedInput<Token<'src>, SimpleSpan, &'tokens [(Token<'src>, SimpleSpan)]> as chumsky::prelude::Input<'tokens>>::Token>>>>(
        expected: E,
        found: Option<chumsky::util::MaybeRef<'tokens, <SpannedInput<Token<'src>, SimpleSpan, &'tokens [(Token<'src>, SimpleSpan)]> as chumsky::prelude::Input<'tokens>>::Token>>,
        span: <SpannedInput<Token<'src>, SimpleSpan, &'tokens [(Token<'src>, SimpleSpan)]> as chumsky::prelude::Input<'tokens>>::Span,
    ) -> Self{
        let expected = expected
            .into_iter()
            .flatten()
            .map(|x| x.into_inner())
            .collect();
        let found = found.map(|x| x.into_inner());
        Self::expected_found(expected, found, span.into())
    }
}
