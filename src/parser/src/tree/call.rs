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
    Nested {
        call: Box<Call<'src>>,
        args: Vec<Expression<'src>>,
    },
}

/// <Call>   ::= <callee> '(' [ <Expression> { ',' <Expression> } [ ',' ] ] ')'
/// <callee> ::= <Loca> | '(' <FunctionObject> ')' | '(' <Call> ')' | <Call>
///
/// new bnf
/// <Call>                ::= <nested_call> | <local_call> | <immediate_call> | <delimited_call>
/// <nested_call>         ::= <Call> <call_expr_arguments>
/// <local_call>          ::= <Local> <call_expr_arguments>
/// <immediate_call>      ::= '(' <FunctionObject> ')' <call_expr_arguments>
/// <delimited_call>      ::= '(' <Call> ')' <call_expr_arguments>
/// <call_expr_arguments> ::= '(' <Expression> { ',' <Expression> } [ ',' ] ')'
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
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect()
        .delimited_by(just(Token::OpenParen), just(Token::CloseParen));

    recursive(|call| {
        let local_call = local()
            .then(expr_arguments.clone())
            .map(|(local, args)| Call::Local { local, args });
        let immediate_call = function_object(block)
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen))
            .then(expr_arguments.clone())
            .map(|(func, args)| Call::Immediate { func, args });
        let delimited_call = call
            .delimited_by(just(Token::OpenParen), just(Token::CloseParen))
            .then(expr_arguments.clone())
            .map(|(call, args)| Call::Nested {
                call: Box::new(call),
                args,
            });
        let nested_call = choice((
            local_call.clone(),
            immediate_call.clone(),
            delimited_call.clone(),
        ))
        .then(expr_arguments.repeated().at_least(1).collect())
        .map(|(call, args_list): (Call<'src>, Vec<Vec<_>>)| {
            let mut nested = call;
            for args in args_list {
                nested = Call::Nested {
                    call: Box::new(nested),
                    args,
                };
            }
            nested
        });

        choice((nested_call, local_call, immediate_call, delimited_call))
    })
}

impl<'a> TreeWalker<'a> for Call<'a> {
    fn analyze(&mut self, tracker: &mut Tracker<'a>) {
        match self {
            Call::Local { local, args } => {
                match local {
                    Local::TableField { name, .. } => tracker.add_capture(name.str),
                    Local::Ident(ident) => tracker.add_capture(ident.str),
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
            Call::Nested { call, args } => {
                call.analyze(tracker);
                for arg in args.iter_mut() {
                    arg.analyze(tracker);
                }
            }
        }
    }
}
