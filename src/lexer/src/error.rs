use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: TextSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    InvalidCharacter(char),
    ExpectedFound(Vec<char>, Option<char>),
    InvalidEscapeSequence(Vec<char>),
}

impl Error {
    pub fn invalid_character(char: char, span: TextSpan) -> Self {
        Self {
            kind: ErrorKind::InvalidCharacter(char),
            span,
        }
    }

    pub fn expected_found(expected: Vec<char>, found: Option<char>, span: TextSpan) -> Self {
        Self {
            kind: ErrorKind::ExpectedFound(expected, found),
            span,
        }
    }

    pub fn invalid_escape_sequence(escape: impl Into<Vec<char>>, span: TextSpan) -> Self {
        Self {
            kind: ErrorKind::InvalidEscapeSequence(escape.into()),
            span,
        }
    }
}
