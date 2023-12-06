use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionObject<'src> {
    pub args: Vec<Ident<'src>>,
    pub body: Chunk<'src>,
}

/// <FunctionObject> ::= 'func' '(' [ <Ident> { ',' <Ident> } [ ',' ] ] ')' <Chunk> 'end'
pub(super) fn function_object<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'src>> + Clone,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, FunctionObject<'src>, ParserError<'src>> + Clone
{
    let args = ident()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect();
    just(Token::Func)
        .ignore_then(args.delimited_by(just(Token::OpenParen), just(Token::CloseParen)))
        .then(block)
        .then_ignore(just(Token::End))
        .map(|(args, block)| FunctionObject {
            args,
            body: block.into(),
        })
}
