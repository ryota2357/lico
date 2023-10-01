use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Call<'src> {
    Local {
        local: Local<'src>,
        args: Vec<Expression<'src>>,
    },
    Immediate {
        func: FunctionObject<'src>,
        args: Vec<Expression<'src>>,
    },
}

/// <Call> ::= ( <Local> | '(' <FunctionObject> ')' ) '(' [ <Expression> { ',' <Expression> } [ ',' ] ] ')'
///
/// TODO: foo()() や (bar())() などの呼び出しに対応する
pub(super) fn call<'tokens, 'src: 'tokens>(
    block: impl Parser<'tokens, ParserInput<'tokens, 'src>, Block<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
    expression: impl Parser<'tokens, ParserInput<'tokens, 'src>, Expression<'src>, ParserError<'tokens, 'src>>
        + Clone
        + 'tokens,
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Call<'src>, ParserError<'tokens, 'src>> + Clone
{
    let expr_arguments = expression
        .clone()
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen));

    let local_call = local()
        .then(expr_arguments.clone())
        .map(|(local, args)| Call::Local { local, args });
    let immediate_call = function_object(block.clone())
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen))
        .then(expr_arguments)
        .map(|(func, args)| Call::Immediate { func, args });

    local_call.or(immediate_call)
}

impl<'a> TreeWalker<'a> for Call<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            Call::Local { local, args } => {
                match local {
                    Local::TableField { name, .. } => tracker.add_capture(name.str),
                    Local::Variable { name } => tracker.add_capture(name.str),
                }
                for arg in args.iter_mut() {
                    arg.analyze(tracker);
                }
            }
            Call::Immediate { func, args } => {
                func.analyze(tracker);
                for arg in args.iter_mut() {
                    arg.analyze(tracker);
                }
            }
        }
    }
}
