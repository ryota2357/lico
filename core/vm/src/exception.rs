use foundation::{il::SourceInfo, syntax::TextRange};
use std::sync::Mutex;

pub static EXCEPTION_LOG: Mutex<ExeptionLog> = Mutex::new(ExeptionLog {
    raw_start: 0,
    log: Vec::new(),
});

pub struct ExeptionLog {
    raw_start: usize,
    log: Vec<RawExeption>,
}

#[derive(Debug)]
pub struct Exeption {
    message: String,
    range: Option<TextRange>,
    // TODO: impliment path to il::SourceInfo or il::Module, then use it.
    // pub(crate) path: Option<String>,
}

impl Exeption {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn range(&self) -> Option<TextRange> {
        self.range
    }
}

impl ExeptionLog {
    pub fn error(&mut self, message: impl Into<String>) {
        self.log.push(RawExeption::TextError {
            message: message.into(),
        })
    }

    pub fn read(&mut self, count: usize) -> impl Iterator<Item = &Exeption> {
        if self.raw_start != self.log.len() {
            for raw in &mut self.log[self.raw_start..] {
                *raw = RawExeption::Processed(Exeption {
                    message: format!("[BUG] EXEPTION_LOG: fixup is not called for: {:?}", raw),
                    range: None,
                });
            }
        }
        self.raw_start = self.log.len();
        self.log.iter().rev().take(count).map(|raw| match raw {
            RawExeption::Processed(exeption) => exeption,
            _ => unreachable!(),
        })
    }

    pub fn read_all(&mut self) -> impl Iterator<Item = &Exeption> {
        self.read(self.log.len())
    }

    pub(crate) fn push_raw(&mut self, message: String, index: usize, extra: usize) {
        self.log.push(RawExeption::Raw {
            message,
            index,
            extra,
        })
    }

    pub(crate) fn fixup(&mut self, info: &SourceInfo) {
        for raw in &mut self.log[self.raw_start..] {
            match raw {
                RawExeption::Raw {
                    message,
                    index,
                    extra,
                } => {
                    let exeption = Exeption {
                        message: message.clone(),
                        range: info.get(*index, *extra),
                    };
                    *raw = RawExeption::Processed(exeption);
                }
                RawExeption::TextError { message } => {
                    let exeption = Exeption {
                        message: message.clone(),
                        range: None,
                    };
                    *raw = RawExeption::Processed(exeption);
                }
                RawExeption::Processed(_) => unreachable!(),
            }
        }
        self.raw_start = self.log.len();
    }
}

#[derive(Debug)]
enum RawExeption {
    Raw {
        message: String,
        index: usize,
        extra: usize,
    },
    TextError {
        // for after implementation of `path` field in Exeption
        message: String,
    },
    Processed(Exeption),
}
