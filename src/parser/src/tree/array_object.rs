use super::*;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayObject<'src>(pub Vec<(Expression<'src>, Span)>);

/// <ArrayObject> ::= '[' [ <Expression> { ',' <Expression> } [ ',' ] ] ']'
pub(super) fn array_object<'tokens, 'src: 'tokens>(
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, (Expression<'src>, Span), ParserError<'src>>
        + Clone,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, ArrayObject<'src>, ParserError<'src>> + Clone
{
    let elements = expression
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect();
    elements
        .delimited_by(just(Token::OpenBracket), just(Token::CloseBracket))
        .map(ArrayObject)
}

impl<'a> Deref for ArrayObject<'a> {
    type Target = Vec<(Expression<'a>, Span)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ArrayObject<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
