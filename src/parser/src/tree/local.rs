use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Local<'src> {
    Ident(Ident<'src>),
    Access {
        ident: Ident<'src>,
        keys: Vec<Expression<'src>>,
    },
}

/// <Local>          ::= <Access> | <Ident>
/// <Access>         ::= <Ident> <field_accessor> { <field_accessor> }
/// <field_accessor> ::= ( '[' <Expression> ']' ) | ( '.' <Ident> )
pub(super) fn local<'tokens, 'src: 'tokens>(
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Local<'src>, ParserError<'tokens, 'src>> + Clone
{
    let field_accessor = {
        let field_accessor1 =
            expression.delimited_by(just(Token::OpenBracket), just(Token::CloseBracket));
        let field_accessor2 = just(Token::Dot)
            .ignore_then(ident())
            .map(|ident| Expression::Local(Local::Ident(ident)));
        field_accessor1.or(field_accessor2)
    };

    let access = ident()
        .then(field_accessor.repeated().at_least(1).collect())
        .map(|(ident, keys)| Local::Access { ident, keys });
    let ident = ident().map(Local::Ident);
    access.or(ident)
}

impl<'a> TreeWalker<'a> for Local<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            Local::Ident(ident) => tracker.add_capture(ident.str),
            Local::Access { ident, keys } => {
                tracker.add_capture(ident.str);
                for key in keys {
                    key.analyze(tracker);
                }
            }
        }
    }
}
