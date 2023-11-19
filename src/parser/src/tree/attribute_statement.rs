use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum AttributeStatement<'src> {
    Function {
        name: (Ident<'src>, Span),
        args: Vec<(Ident<'src>, Span)>,
    },
    Variable {
        name: (Ident<'src>, Span),
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
        (Ident(str), span.into())
    });

    let function = attr_name
        .then(
            choice((
                spanned_ident(),
                select! { Token::Bool(x) => if x { Ident("true") } else { Ident("false") } }
                    .map_with(|ident, ext| {
                        let span: SimpleSpan = ext.span();
                        (ident, span.into())
                    }),
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
            AttributeStatement::Function {
                name: (name, span), ..
            } => tracker.add_attribute(name.0, span.clone()),
            AttributeStatement::Variable { name: (name, span) } => {
                tracker.add_attribute(name.0, span.clone())
            }
        }
    }
}
