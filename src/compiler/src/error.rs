use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    NoLoopToBreak,
    NoLoopToContinue,
    UndefinedVariable(String),
}

impl Error {
    pub fn no_loop_to_break(span: Span) -> Self {
        Self {
            kind: ErrorKind::NoLoopToBreak,
            span,
        }
    }

    pub fn no_loop_to_continue(span: Span) -> Self {
        Self {
            kind: ErrorKind::NoLoopToContinue,
            span,
        }
    }

    pub fn undefined_variable(name: String, span: Span) -> Self {
        Self {
            kind: ErrorKind::UndefinedVariable(name.into()),
            span,
        }
    }
}
