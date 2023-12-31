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
pub(super) fn attribute_statement<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, AttributeStatement<'src>, ParserError<'src>> + Clone
{
    let attr_name = select! {
        Token::Attribute(x) => x
    }
    .map_with(|str, ext| {
        let span: SimpleSpan = ext.span();
        Ident(str, span.into())
    });

    let function = attr_name
        .then(
            choice((
                ident(),
                select! { Token::Bool(x) => if x { "true" } else { "false" } }.map_with(
                    |ident, ext| {
                        let span: SimpleSpan = ext.span();
                        Ident(ident, span.into())
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

impl<'walker, 'src: 'walker> Walkable<'walker, 'src> for AttributeStatement<'src> {
    fn accept(&mut self, walker: &mut Walker<'walker, 'src>) {
        match self {
            AttributeStatement::Function {
                name: Ident(name, span),
                args: _,
            } => walker.record_variable_usage(name, span),
            AttributeStatement::Variable {
                name: Ident(name, span),
            } => {
                walker.record_variable_usage(name, span);
            }
        }
    }
}
