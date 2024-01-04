use super::*;
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("unexpected symbol `{0}`")]
    UnexpectedSymbol(&'static str, TextSpan),

    #[error("missing symbol `{0}`")]
    UnexpectedEof(&'static str, TextSpan),

    #[error("expected {expected}, found `{}`", .found.0)]
    ExpectedFound {
        expected: &'static str,
        found: (String, TextSpan),
    },

    #[error("`{0}` is required")]
    MissingRequiredElement(&'static str, TextSpan),

    #[error("missing a closing `{expected}`")]
    MissingClosingSymbol {
        info: (Option<String>, TextSpan),
        expected: String,
    },

    #[error("{reason}")]
    InvalidStatement {
        info: (String, TextSpan),
        reason: String,
    },
}
