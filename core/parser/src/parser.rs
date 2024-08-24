use crate::token_set::TokenSet;
use core::{mem, num::NonZeroU32};
use foundation::syntax::SyntaxKind;
use std::borrow::Cow;

pub(crate) struct Parser {
    input: Vec<SyntaxKind>,
    pos: usize,
    events: Vec<EventRaw>,
    errors: Vec<Cow<'static, str>>,
}

impl Parser {
    pub(crate) fn new(input: Vec<SyntaxKind>) -> Self {
        Parser {
            input,
            pos: 0,
            events: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub(crate) fn current(&self) -> Option<SyntaxKind> {
        self.input.get(self.pos).copied()
    }

    pub(crate) fn at(&self, kind: SyntaxKind) -> bool {
        self.input.get(self.pos) == Some(&kind)
    }

    pub(crate) fn at_ts(&self, kinds: TokenSet) -> bool {
        let Some(current) = self.current() else {
            return false;
        };
        kinds.contains(current)
    }

    pub(crate) fn nth_at(&self, n: usize, kind: SyntaxKind) -> bool {
        self.input.get(self.pos + n) == Some(&kind)
    }

    pub(crate) fn start(&mut self) -> Marker {
        let pos = self.events.len() as u32;
        self.events.push(EventRaw::Tombstone);
        Marker { pos }
    }

    pub(crate) fn eat(&mut self, kind: SyntaxKind) -> bool {
        if !self.at(kind) {
            return false;
        }
        self._push_token(kind);
        true
    }

    pub(crate) fn eat_trivia(&mut self) {
        while let Some(kind) = self.current() {
            if kind.is_trivia() {
                self._push_token(kind);
            } else {
                break;
            }
        }
    }

    pub(crate) fn bump(&mut self, kind: SyntaxKind) {
        assert!(self.eat(kind));
    }

    pub(crate) fn bump_any(&mut self) {
        let Some(kind) = self.current() else {
            unreachable!("Unexpected EOF");
        };
        self._push_token(kind);
    }

    pub(crate) fn error(&mut self, message: impl Into<Cow<'static, str>>) {
        let message_index = self.errors.len() as u32;
        self.errors.push(message.into());
        self.events.push(EventRaw::EmptyError { message_index });
    }

    pub(crate) fn error_with<F, M>(&mut self, func: F)
    where
        F: FnOnce(&mut Parser) -> M,
        M: Into<Cow<'static, str>>,
    {
        self.events.push(EventRaw::StartError);
        let message = func(self);
        let message_index = self.errors.len() as u32;
        self.errors.push(message.into());
        self.events.push(EventRaw::FinishError { message_index });
    }

    pub(crate) fn finish(self) -> Output {
        assert_eq!(self.input.len(), self.pos);
        Output::new(self.events, self.errors)
    }

    fn _push_token(&mut self, kind: SyntaxKind) {
        self.events.push(EventRaw::Token { kind });
        self.pos += 1;
    }
}

#[derive(Debug)]
enum EventRaw {
    StartNode {
        kind: SyntaxKind,
        forward_parent: Option<NonZeroU32>,
    },
    FinishNode,
    Token {
        kind: SyntaxKind,
    },
    Tombstone,
    EmptyError {
        message_index: u32,
    },
    StartError,
    FinishError {
        message_index: u32,
    },
}

pub(crate) enum Event {
    StartNode {
        kind: SyntaxKind,
        forward_parent: Option<NonZeroU32>,
    },
    FinishNode,
    Token {
        kind: SyntaxKind,
    },
    None,
    EmptyError {
        message: Cow<'static, str>,
    },
    StartError,
    FinishError {
        message: Cow<'static, str>,
    },
}

#[derive(Debug)]
pub(crate) struct Output {
    events: Vec<EventRaw>,
    errors: Vec<Cow<'static, str>>,
}

impl Output {
    fn new(events: Vec<EventRaw>, errors: Vec<Cow<'static, str>>) -> Self {
        Output { events, errors }
    }

    pub(crate) fn event_count(&self) -> usize {
        self.events.len()
    }

    pub(crate) fn take_event(&mut self, index: usize) -> Event {
        match mem::replace(&mut self.events[index], EventRaw::Tombstone) {
            EventRaw::StartNode {
                kind,
                forward_parent,
            } => Event::StartNode {
                kind,
                forward_parent,
            },
            EventRaw::FinishNode => Event::FinishNode,
            EventRaw::Token { kind } => Event::Token { kind },
            EventRaw::Tombstone => Event::None,
            EventRaw::EmptyError { message_index } => {
                let message = mem::take(&mut self.errors[message_index as usize]);
                Event::EmptyError { message }
            }
            EventRaw::StartError => Event::StartError,
            EventRaw::FinishError { message_index } => {
                let message = mem::take(&mut self.errors[message_index as usize]);
                Event::FinishError { message }
            }
        }
    }
}

#[must_use]
pub(crate) struct Marker {
    pos: u32,
}

impl Marker {
    pub(crate) fn complete(self, p: &mut Parser, kind: SyntaxKind) -> CompletedMarker {
        let pos = self.pos;
        mem::forget(self);
        match &mut p.events[pos as usize] {
            slot @ EventRaw::Tombstone => {
                *slot = EventRaw::StartNode {
                    kind,
                    forward_parent: None,
                }
            }
            _ => unreachable!(),
        }
        p.events.push(EventRaw::FinishNode);
        CompletedMarker { pos }
    }
}

impl Drop for Marker {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!("Marker must be completed")
        }
    }
}

pub(crate) struct CompletedMarker {
    pos: u32,
}

impl CompletedMarker {
    pub(crate) fn precede(self, p: &mut Parser) -> Marker {
        let new_pos = p.start();
        let idx = self.pos as usize;
        match &mut p.events[idx] {
            EventRaw::StartNode { forward_parent, .. } => {
                let forward = new_pos.pos - self.pos;
                assert!(forward > 0);
                *forward_parent = Some(unsafe { NonZeroU32::new_unchecked(forward) });
            }
            _ => unreachable!(),
        }
        new_pos
    }
}
