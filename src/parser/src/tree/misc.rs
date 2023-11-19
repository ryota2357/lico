use std::ops::{Deref, DerefMut};

use super::*;

/// <Primitive> ::= <Int> | <Float> | <String> | <Bool> | <Nil>
/// <Int>       ::= __int
/// <Float>     ::= __float
/// <String>    ::= __string
/// <Bool>      ::= __bool
/// <Nil>       ::= __nil
#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
}
pub(super) fn primitive<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Primitive, ParserError<'tokens, 'src>> + Clone
{
    select! {
        Token::Int(x) => Primitive::Int(x),
        Token::Float(x) => Primitive::Float(x),
        Token::String(x) => Primitive::String(x),
        Token::Bool(x) => Primitive::Bool(x),
        Token::Nil => Primitive::Nil,
    }
}

/// <Ident> ::= __ident
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ident<'src>(pub &'src str);

impl<'a> Deref for Ident<'a> {
    type Target = &'a str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Ident<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(super) fn ident<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Ident<'src>, ParserError<'tokens, 'src>> + Clone
{
    select! {
        Token::Ident(x) => x
    }
    .map(Ident)
}

pub(super) fn spanned_ident<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, (Ident<'src>, Span), ParserError<'tokens, 'src>>
       + Clone {
    select! {
        Token::Ident(x) => x
    }
    .map_with(|str, ext| {
        let span: SimpleSpan = ext.span();
        (Ident(str), span.into())
    })
}
