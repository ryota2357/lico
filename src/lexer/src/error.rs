type Span = chumsky::span::SimpleSpan<usize>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    InvalidCharacter(char),
    ExpectedFound(Vec<char>, Option<char>),
}

impl Error {
    pub fn invalid_character(char: char, span: Span) -> Self {
        Self {
            kind: ErrorKind::InvalidCharacter(char),
            span,
        }
    }

    pub fn expected_found(expected: Vec<char>, found: Option<char>, span: Span) -> Self {
        Self {
            kind: ErrorKind::ExpectedFound(expected, found),
            span,
        }
    }
}

impl<'a> chumsky::error::Error<'a, &'a str> for Error {
    fn expected_found<E: IntoIterator<Item = Option<chumsky::util::Maybe<char, &'a char>>>>(
        expected: E,
        found: Option<chumsky::util::Maybe<char, &'a char>>,
        span: Span,
    ) -> Self {
        let expected = expected
            .into_iter()
            .flatten()
            .map(|x| x.into_inner())
            .collect();
        let found = found.map(|x| x.into_inner());
        Self::expected_found(expected, found, span)
    }
}
