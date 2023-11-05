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
pub struct Ident<'src> {
    pub str: &'src str,
    pub span: Span,
}
pub(super) fn ident<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Ident<'src>, ParserError<'tokens, 'src>> + Clone
{
    select! {
        Token::Identifier(x) => x
    }
    .map_with(|str, ext| Ident {
        str,
        span: ext.span(),
    })
}
