use super::*;
use crate::error::Error;
use std::{iter::Peekable, str::CharIndices};

#[derive(Debug)]
pub struct Lexer<'src> {
    tokens: Vec<(Token<'src>, TextSpan)>,
    errors: Vec<Error>,
    chars: Peekable<CharIndices<'src>>,
    start_pos: Option<u32>,
    count: u32,
    rest: &'src str,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        if source.len() > u32::MAX as usize {
            panic!("Source code is too long");
        }
        let chars = source.char_indices().peekable();
        Self {
            tokens: Vec::new(),
            errors: Vec::new(),
            chars,
            start_pos: None,
            count: 0,
            rest: source,
        }
    }

    pub fn next(&mut self) -> Option<char> {
        let (i, c) = self.chars.next().map(|(i, c)| (i as u32, c))?;
        if self.start_pos.is_none() {
            self.start_pos = Some(i);
        }
        self.count += c.len_utf8() as u32;
        Some(c)
    }

    #[inline]
    pub fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    pub fn consume_ws(&mut self) {
        assert!(
            self.start_pos.is_none(),
            "Cannot consume whitespace during tokenization."
        );
        let mut count = 0;
        while let Some((_, c)) = self.chars.peek() {
            if !c.is_whitespace() {
                break;
            }
            count += c.len_utf8();
            self.chars.next();
        }
        self.rest = &self.rest[count..];
    }

    pub fn take_until(&mut self, pred: impl Fn(char) -> bool) {
        while let Some(c) = self.peek() {
            if pred(c) {
                break;
            }
            self.next();
        }
    }

    pub fn take_while(&mut self, pred: impl Fn(char) -> bool) {
        while let Some(c) = self.peek() {
            if pred(c) {
                self.next();
            } else {
                break;
            }
        }
    }

    #[inline]
    pub fn get_slice(&self) -> &'src str {
        &self.rest[..self.count as usize]
    }

    pub fn get_span(&self) -> TextSpan {
        let start_pos = self.start_pos.expect("No start position");
        let length = self.count;
        TextSpan::at(start_pos, length)
    }

    pub fn bump(&mut self, token: Token<'src>) {
        let span = {
            let start_pos = self.start_pos.take().expect("No start position");
            let length = self.count;
            TextSpan::at(start_pos, length)
        };
        self.tokens.push((token, span));
        self.rest = &self.rest[self.count as usize..];
        self.count = 0;
    }

    pub fn report(&mut self, reporter: impl FnOnce(TextSpan) -> Error) {
        let span = self.get_span();
        let error = reporter(span);
        self.errors.push(error);
    }

    #[inline]
    pub fn into_tokens_errors(self) -> (Vec<(Token<'src>, TextSpan)>, Vec<Error>) {
        (self.tokens, self.errors)
    }
}
