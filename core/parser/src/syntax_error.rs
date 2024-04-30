use core::fmt;
use rowan::TextRange;
use std::{borrow::Cow, error::Error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SyntaxError {
    message: Cow<'static, str>,
    range: TextRange,
}

impl SyntaxError {
    pub(crate) fn new(message: Cow<'static, str>, range: TextRange) -> Self {
        Self { message, range }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn range(&self) -> TextRange {
        self.range
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for SyntaxError {}
