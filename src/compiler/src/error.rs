use foundation::TextSpan;

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: TextSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    NoLoopToBreak,
    NoLoopToContinue,
    UndefinedVariable(String),
}

impl Error {
    pub fn no_loop_to_break(span: TextSpan) -> Self {
        Self {
            kind: ErrorKind::NoLoopToBreak,
            span,
        }
    }

    pub fn no_loop_to_continue(span: TextSpan) -> Self {
        Self {
            kind: ErrorKind::NoLoopToContinue,
            span,
        }
    }

    pub fn undefined_variable(name: String, span: TextSpan) -> Self {
        Self {
            kind: ErrorKind::UndefinedVariable(name),
            span,
        }
    }
}
