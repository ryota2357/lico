use crate::Token;
use chumsky::prelude::*;

type Span = SimpleSpan<usize>;
type LexerInput<'a> = &'a str;
type LexerError<'a> = extra::Err<Simple<'a, char, Span>>;
type LexerOutput<'a> = Vec<(Token<'a>, Span)>;

pub(crate) fn lexer<'src>(
) -> impl Parser<'src, LexerInput<'src>, LexerOutput<'src>, LexerError<'src>> {
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

    let string = {
        // TODO: Support escape sequences
        // let escape = just('\\').ignore_then(
        //     just('\\')
        //         .or(just('/'))
        //         .or(just('"'))
        //         .or(just('\''))
        //         .or(just('b').to('\x08'))
        //         .or(just('f').to('\x0C'))
        //         .or(just('n').to('\n'))
        //         .or(just('r').to('\r'))
        //         .or(just('t').to('\t')),
        // );
        // let str1 = just('"')
        //     .ignore_then(none_of("\\\"").or(escape).repeated().slice())
        //     .then_ignore(just('"'));
        // let str2 = just('\'')
        //     .ignore_then(none_of("\\\'").or(escape).repeated().slice())
        //     .then_ignore(just('\''));

        let str1 = just('"')
            .ignore_then(none_of("\"").repeated().to_slice())
            .then_ignore(just('"'));
        let str2 = just('\'')
            .ignore_then(none_of("'").repeated().to_slice())
            .then_ignore(just('\''));

        str1.or(str2).map(Token::String)
    };

    let symbol = choice((
        // operator
        just('+').to(Token::Add),
        just('-').to(Token::Sub),
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
        just("..").to(Token::SrtJoin),
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
        // other
        just('@').to(Token::AtMark),
    ));

    let word = text::ascii::ident().map(|ident: &str| match ident {
        // literals
        "true" => Token::Bool(true),
        "false" => Token::Bool(false),
        "nil" => Token::Nil,

        // keywords
        "var" => Token::Var,
        "let" => Token::Let,
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

    let token = choice((float, int, string, symbol, word))
        .map_with(|token, ext| (token, ext.span()))
        .padded();

    token.repeated().collect()
}
