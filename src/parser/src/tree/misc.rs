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
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Primitive, ParserError<'src>> + Clone {
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
pub struct Ident<'src>(pub &'src str, pub Span);

pub(super) fn ident<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Ident<'src>, ParserError<'src>> + Clone {
    select! {
        Token::Ident(x) = extra => Ident(x, {
            let span: SimpleSpan = extra.span();
            span.into()
        })
    }
}
