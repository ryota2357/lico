use super::*;
use std::{num::ParseIntError, ops::RangeInclusive};

use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("invalid input sequence")]
    InvalidInputSequence(String, TextSpan),

    #[error("unknown number literal")]
    UnknownNumberLiteral(String, TextSpan),

    #[error("{reason}")]
    InvalidFloatLiteral {
        info: (String, TextSpan),
        reason: String,
    },

    #[error("{source}")]
    InvalidIntLiteral {
        info: (String, TextSpan),
        #[source]
        source: ParseIntError,
    },

    #[error("missing a closing `{expected}`")]
    MissingClosingDelimiter {
        info: (Option<char>, TextSpan),
        expected: char,
    },

    #[error("invalid escape sequence")]
    InvalidEscapeSequence {
        info: (String, TextSpan),
        reason: String,
    },

    #[error("unexpected character `{}` in escape sequence", .info.0)]
    UnexpectedCharInEscapeSequence {
        info: (char, TextSpan),
        expected: Vec<RangeInclusive<char>>,
    },
}
