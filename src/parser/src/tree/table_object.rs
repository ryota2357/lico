use super::*;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq)]
pub struct TableObject<'src>(pub Vec<((Expression<'src>, Span), (Expression<'src>, Span))>);

/// <TableObject> ::= '{' [ <table filed> { ',' <table filed> } [ ',' ] ] '}'
/// <table filed> ::= <Ident> '=' <Expression>
///
///  TODO: 次に対応する
///  <table filed> ::= ( <Ident> | '[' <Expression> ']' ) '=' <Expression>
pub(super) fn table_object<'tokens, 'src: 'tokens>(
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, (Expression<'src>, Span), ParserError<'src>>
        + Clone,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, TableObject<'src>, ParserError<'src>> + Clone
{
    let key = ident()
        .map(|ident| Expression::Primitive(Primitive::String(ident.0.to_string())))
        .map_with(|expr, ext| (expr, ext.span().into()));
    let table_field = key.then_ignore(just(Token::Assign)).then(expression);
    let elements = table_field
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect();
    elements
        .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace))
        .map(TableObject)
}

impl<'a> Deref for TableObject<'a> {
    type Target = Vec<((Expression<'a>, Span), (Expression<'a>, Span))>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TableObject<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
