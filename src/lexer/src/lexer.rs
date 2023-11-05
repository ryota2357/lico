use super::*;

use crate::error::Error;

type LexerInput<'a> = &'a str;
type LexerOutput<'a> = Vec<(Token<'a>, Span)>;
type LexerExtra<'a> = extra::Err<Error>;

pub(super) fn lexer<'src>(
) -> impl Parser<'src, LexerInput<'src>, LexerOutput<'src>, LexerExtra<'src>> {
    let int = text::digits(10)
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::Int);

    let float = text::digits(10)
        .then(just('.').then(text::digits(10)))
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::Float);

    let attribute = just('@')
        .ignore_then(text::ascii::ident())
        .map(Token::Attribute);

    let string = {
        // TODO: Improve error handling in \x and \u
        let escape = just('\\').ignore_then(choice((
            just('\\'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('0').to('\0'),
            just('x')
                .ignore_then(one_of("01234567"))
                .then(one_of("0123456789abcdefABCDEF"))
                .map(|(o, x): (char, char)| {
                    let o = o.to_digit(16).unwrap();
                    let x = x.to_digit(16).unwrap();
                    std::char::from_u32(o * 16 + x).unwrap()
                }),
            just('u')
                .ignore_then(just('{'))
                .ignore_then(
                    one_of("0123456789abcdefABCDEF")
                        .repeated()
                        .at_least(1)
                        .at_most(6)
                        .to_slice(),
                )
                .then_ignore(just('}'))
                .validate(|digits, extra, emitter| {
                    let num = u32::from_str_radix(digits, 16).unwrap();
                    // NOTE: If the number is greater than 10FFFF, it is invalid.
                    std::char::from_u32(num).unwrap_or_else(|| {
                        let span = {
                            let s: Span = extra.span();
                            (s.start - 1..s.end).into() //  -1 means the position of `\`
                        };
                        emitter.emit(Error::invalid_escape_sequence(
                            format!("\\u{{{}}}", digits).chars().collect::<Vec<_>>(),
                            span,
                        ));
                        ' '
                    })
                }),
            any().validate(|c, extra, emitter| {
                let span = {
                    let s: Span = extra.span();
                    (s.start - 1..s.end).into() //  -1 means the position of `\`
                };
                emitter.emit(Error::invalid_escape_sequence(['\\', c], span));
                c
            }),
        )));

        let str1 = just('"')
            .ignore_then(
                none_of(r#"\""#)
                    .or(just(r#"\""#).to('"'))
                    .or(escape)
                    .repeated()
                    .collect(),
            )
            .then_ignore(just('"'));
        let str2 = just('\'')
            .ignore_then(
                none_of(r"\'")
                    .or(just(r"\'").to('\''))
                    .or(escape)
                    .repeated()
                    .collect(),
            )
            .then_ignore(just('\''));

        str1.or(str2).map(Token::String)
    };

    let symbol = choice((
        // operator
        just('+').to(Token::Pluss),
        just("->").to(Token::Arrow),
        just('-').to(Token::Minus),
        just("**").to(Token::Pow),
        just('*').to(Token::Mul),
        just('/').to(Token::Div),
        just('%').to(Token::Mod),
        just("==").to(Token::Eq),
        just("!=").to(Token::NotEq),
        just("<=").to(Token::LessEq),
        just('<').to(Token::Less),
        just(">=").to(Token::GreaterEq),
        just('>').to(Token::Greater),
        just("..").to(Token::StrJoin),
        just('.').to(Token::Dot),
        just('=').to(Token::Assign),
        // delimiter
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just('(').to(Token::OpenParen),
        just(')').to(Token::CloseParen),
        just('{').to(Token::OpenBrace),
        just('}').to(Token::CloseBrace),
        just('[').to(Token::OpenBracket),
        just(']').to(Token::CloseBracket),
    ));

    let word = text::ascii::ident().map(|ident: &str| match ident {
        // literals
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "nil" => Token::Nil,

        // keywords
        "var" => Token::Var,
        "func" => Token::Func,
        "if" => Token::If,
        "then" => Token::Then,
        "elif" => Token::Elif,
        "else" => Token::Else,
        "for" => Token::For,
        "while" => Token::While,
        "in" => Token::In,
        "do" => Token::Do,
        "end" => Token::End,
        "return" => Token::Return,
        "break" => Token::Break,
        "continue" => Token::Continue,

        // keyword operators
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,

        // other
        _ => Token::Identifier(ident),
    });

    let token = choice((float, int, string, symbol, attribute, word))
        .or(any().validate(|c, extra, emitter| {
            emitter.emit(Error::invalid_character(c, extra.span()));
            Token::Error(c)
        }))
        .map_with(|token, ext| (token, ext.span()))
        .padded();

    token.repeated().collect()
}
