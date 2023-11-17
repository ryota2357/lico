use super::*;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq)]
pub struct TableObject<'src>(pub Vec<((Expression<'src>, Span), (Expression<'src>, Span))>);

/// <TableObject> ::= '{' [ <table filed> { ',' <table filed> } [ ',' ] ] '}'
/// <table filed> ::= <Ident> '=' <Expression>
///
///  TODO: 次に対応する
///  <table filed> ::= ( <Ident> | '[' <Expression> ']' ) '=' <Expression>
pub(super) fn table_object<'tokens, 'src: 'tokens>(
    expression: impl Parser<
            'tokens,
            ParserInput<'tokens, 'src>,
            (Expression<'src>, Span),
            ParserError<'tokens, 'src>,
        > + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, TableObject<'src>, ParserError<'tokens, 'src>>
       + Clone {
    let key = ident()
        .map(Expression::Ident)
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

impl<'a> TreeWalker<'a> for TableObject<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        for (_, (value, _)) in &mut self.0 {
            value.analyze(tracker);
        }
    }
}
