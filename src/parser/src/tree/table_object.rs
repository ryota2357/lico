use super::*;
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq)]
pub struct TableObject<'src> {
    pub key_values: Vec<(Expression<'src>, Expression<'src>)>,
}

/// <TableObject> ::= '{' [ <table filed> { ',' <table filed> } [ ',' ] ] '}'
/// <table filed> ::= <Ident> '=' <Expression>
///
///  TODO: 次に対応する
///  <table filed> ::= ( <Ident> | '[' <Expression> ']' ) '=' <Expression>
pub(super) fn table_object<'tokens, 'src: 'tokens>(
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, TableObject<'src>, ParserError<'tokens, 'src>>
       + Clone {
    let key = ident().map(Expression::Ident);
    let table_field = key.then_ignore(just(Token::Assign)).then(expression);
    let elements = table_field
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect();
    elements
        .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace))
        .map(|key_values| TableObject { key_values })
}

impl<'a> Deref for TableObject<'a> {
    type Target = Vec<(Expression<'a>, Expression<'a>)>;

    fn deref(&self) -> &Self::Target {
        &self.key_values
    }
}

impl<'a> TreeWalker<'a> for TableObject<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        for (_, value) in &mut self.key_values {
            value.analyze(tracker);
        }
    }
}
