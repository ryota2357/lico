use super::*;
use std::{num::ParseIntError, ops::RangeInclusive};
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Invalid input sequence")]
    InvalidInputSequence(String, TextSpan),

    #[error("Unknown number literal")]
    UnknownNumberLiteral(String, TextSpan),

    #[error("{reason}")]
    InvalidFloatLiteral {
        info: (String, TextSpan),
        reason: String,
    },

    #[error("Invalid int literal: {source}")]
    InvalidIntLiteral {
        info: (String, TextSpan),
        #[source]
        source: ParseIntError,
    },

    #[error("Missing a closing `{expected}`")]
    MissingClosingDelimiter {
        info: (Option<char>, TextSpan),
        expected: char,
    },

    #[error("Invalid escape sequence")]
    InvalidEscapeSequence {
        info: (String, TextSpan),
        reason: String,
    },

    #[error("Unexpected character `{}` in escape sequence", .info.0)]
    UnexpectedCharInEscapeSequence {
        info: (char, TextSpan),
        expected: Vec<RangeInclusive<char>>,
    },
}

impl Error {
    pub fn text(&self) -> Option<String> {
        use Error::*;
        match self {
            InvalidInputSequence(x, _) => Some(x.clone()),
            UnknownNumberLiteral(x, _) => Some(x.clone()),
            InvalidFloatLiteral { info: (x, _), .. } => Some(x.clone()),
            InvalidIntLiteral { info: (x, _), .. } => Some(x.clone()),
            MissingClosingDelimiter { info: (x, _), .. } => x.map(|x| x.to_string()),
            InvalidEscapeSequence { info: (x, _), .. } => Some(x.clone()),
            UnexpectedCharInEscapeSequence { info: (x, _), .. } => Some(x.to_string()),
        }
    }

    pub fn span(&self) -> TextSpan {
        use Error::*;
        match self {
            InvalidInputSequence(_, x) => *x,
            UnknownNumberLiteral(_, x) => *x,
            InvalidFloatLiteral { info: (_, x), .. } => *x,
            InvalidIntLiteral { info: (_, x), .. } => *x,
            MissingClosingDelimiter { info: (_, x), .. } => *x,
            InvalidEscapeSequence { info: (_, x), .. } => *x,
            UnexpectedCharInEscapeSequence { info: (_, x), .. } => *x,
        }
    }
}
