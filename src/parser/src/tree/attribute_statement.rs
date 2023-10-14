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
/// <FunctionAttribute>  ::= __attribute '(' <attribute_fn_value> { ',' <attribute_fn_value> } ')'
/// <VariableAttribute>  ::= __attribute
/// <attribute_fn_value> ::= <Ident> | <Bool>
pub(super) fn attribute_statement<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    AttributeStatement<'src>,
    ParserError<'tokens, 'src>,
> + Clone {
    let attr_name = select! {
        Token::Attribute(x) => x
    }
    .map_with(|name, ext| Ident {
        str: name,
        span: ext.span(),
    });

    let function = attr_name
        .then(
            choice((
                ident(),
                select! { Token::Bool(x) => if x { "true" } else { "false" } }.map_with(
                    |str, ext| Ident {
                        str,
                        span: ext.span(),
                    },
                ),
            ))
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect()
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen)),
        )
        .map(|(name, args)| AttributeStatement::Function { name, args });
    let variable = attr_name.map(|name| AttributeStatement::Variable { name });

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
