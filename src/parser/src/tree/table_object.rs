use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct TableObject<'src> {
    pub keys: Vec<Ident<'src>>,
    pub values: Vec<Expression<'src>>,
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
    let table_field = ident().then_ignore(just(Token::Assign)).then(expression);
    let elements = table_field
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect();
    elements
        .delimited_by(just(Token::OpenBrace), just(Token::CloseBrace))
        .map(|x: Vec<_>| x.into_iter().unzip())
        .map(|(keys, values)| TableObject { keys, values })
}

impl<'a> TreeWalker<'a> for TableObject<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        for value in &mut self.values {
            value.analyze(tracker);
        }
    }
}
