use super::*;

/// <Local>      ::= <TableField> | <Variable>
/// <TableField> ::= <Ident> '.' <Ident> { '.' <Ident> }
/// <Variable>   ::= <Ident>
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Local<'src> {
    TableField {
        name: Ident<'src>,
        keys: Vec<Ident<'src>>,
    },
    Variable {
        name: Ident<'src>,
    },
}
pub(super) fn local<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Local<'src>, ParserError<'tokens, 'src>> + Clone
{
    let variable = ident().map(|name| Local::Variable { name });
    let table = ident()
        .then(
            just(Token::Dot)
                .ignore_then(ident())
                .repeated()
                .at_least(1)
                .collect(),
        )
        .map(|(name, keys)| Local::TableField { name, keys });

    table.or(variable)
}

/// <Primitive> ::= <Int> | <Float> | <String> | <Bool> | <Nil>
/// <Int>       ::= __int
/// <Float>     ::= __float
/// <String>    ::= __string
/// <Bool>      ::= __bool
/// <Nil>       ::= __nil
#[derive(Clone, Debug, PartialEq)]
pub enum Primitive<'src> {
    Int(i64),
    Float(f64),
    String(&'src str),
    Bool(bool),
    Nil,
}
pub(super) fn primitive<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Primitive<'src>, ParserError<'tokens, 'src>> + Clone
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
    .map_with_span(|str, span| Ident { str, span })
}
