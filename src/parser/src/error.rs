use super::*;
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Unexpected symbol `{0}`")]
    UnexpectedSymbol(&'static str, TextSpan),

    #[error("Missing symbol `{0}`")]
    UnexpectedEof(&'static str, TextSpan),

    #[error("Expected {expected}, found `{}`", .found.0)]
    ExpectedFound {
        expected: &'static str,
        found: (String, TextSpan),
    },

    #[error("`{0}` is required")]
    MissingRequiredElement(&'static str, TextSpan),

    #[error("Missing a closing `{expected}`")]
    MissingClosingSymbol {
        info: (Option<String>, TextSpan),
        expected: String,
    },

    #[error("{reason}")]
    InvalidStatement {
        info: (String, TextSpan),
        reason: String,
    },

    #[error("{0}")]
    Contextual(String, TextSpan),
}
