use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    NoLoopToBreak,
    NoLoopToContinue,
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
}
