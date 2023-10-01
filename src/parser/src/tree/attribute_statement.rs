use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum AttributeStatement<'src> {
    Function {
        name: Ident<'src>,
        args: Vec<Ident<'src>>,
    },
    Variable {
        name: Ident<'src>,
    },
}

/// <AttributeStatement> ::= <FunctionAttribute> | <VariableAttribute>
/// <FunctionAttribute>  ::= '@' <Ident> '(' <Ident> { ',' <Ident> } ')'
/// <VariableAttribute>  ::= '@' <Ident>
pub(super) fn attribute_statement<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    AttributeStatement<'src>,
    ParserError<'tokens, 'src>,
> + Clone {
    let function = just(Token::AtMark)
        .ignore_then(ident())
        .then(
            ident()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect()
                .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
        )
        .map(|(name, args)| AttributeStatement::Function { name, args });
    let variable = just(Token::AtMark)
        .ignore_then(ident())
        .map(|name| AttributeStatement::Variable { name });

    function.or(variable)
}

impl<'a> TreeWalker<'a> for AttributeStatement<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            AttributeStatement::Function { name, .. } => tracker.add_attribute(name.str, name.span),
            AttributeStatement::Variable { name } => tracker.add_attribute(name.str, name.span),
        }
    }
}
