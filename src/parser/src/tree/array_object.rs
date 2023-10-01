use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayObject<'src> {
    pub elements: Vec<Expression<'src>>,
}

/// <ArrayObject> ::= '[' [ <Expression> { ',' <Expression> } [ ',' ] ] ']'
pub(super) fn array_object<'tokens, 'src: 'tokens>(
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, ArrayObject<'src>, ParserError<'tokens, 'src>>
       + Clone {
    just(Token::OpenBracket)
        .ignore_then(
            expression
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect(),
        )
        .then_ignore(just(Token::CloseBracket))
        .map(|values| ArrayObject { elements: values })
}

impl<'a> TreeWalker<'a> for ArrayObject<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        for value in &mut self.elements {
            value.analyze(tracker);
        }
    }
}
