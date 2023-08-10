use chumsky::prelude::*;

type Span = SimpleSpan<usize>;
type Error<'a> = Rich<'a, char, Span>;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'src> {
    // literals
    Int(i64),
    Float(f64),
    String(&'src str),
    Bool(bool),
    Nil,

    // keywords
    Var,
    Let,
    Func,
    If,
    Then,
    Elif,
    Else,
    For,
    While,
    In,
    Do,
    End,
    Return,
    Break,
    Continue,

    // operators
    Add,       // +
    Sub,       // -
    Mul,       // *
    Div,       // /
    Mod,       // %
    Pow,       // ^
    Eq,        // ==
    NotEq,     // !=
    Less,      // <
    LessEq,    // <=
    Greater,   // >
    GreaterEq, // >=
    Dot,       // .

    // keyword operators
    And,
    Or,
    Not,

    // delimiters
    Comma,        // ,
    OpenParen,    // (
    CloseParen,   // )
    OpenBrace,    // {
    CloseBrace,   // }
    OpenBracket,  // [
    CloseBracket, // ]

    // other
    Identifier(&'src str),
}

pub fn lexer<'src>(
) -> impl Parser<'src, &'src str, Vec<(Token<'src>, Span)>, extra::Err<Error<'src>>> {
    let int = text::int(10).from_str().unwrapped().map(Token::Int);

    let float = text::int(10)
        .then(just('.').then(text::digits(10)))
        .slice()
        .from_str()
        .unwrapped()
        .map(Token::Float);

    let string = {
        let escape = just('\\').ignore_then(
            just('\\')
                .or(just('/'))
                .or(just('"'))
                .or(just('\''))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t')),
        );
        let str1 = just('"')
            .ignore_then(none_of("\\\"").or(escape).repeated().slice())
            .then_ignore(just('"'));
        let str2 = just('\'')
            .ignore_then(none_of("\\\'").or(escape).repeated().slice())
            .then_ignore(just('\''));

        str1.or(str2).map(Token::String)
    };

    let operator = choice((
        just('+').to(Token::Add),
        just('-').to(Token::Sub),
        just('*').to(Token::Mul),
        just('/').to(Token::Div),
        just('%').to(Token::Mod),
        just('^').to(Token::Pow),
        just("==").to(Token::Eq),
        just("!=").to(Token::NotEq),
        just("<=").to(Token::LessEq),
        just('<').to(Token::Less),
        just(">=").to(Token::GreaterEq),
        just('>').to(Token::Greater),
        just('.').to(Token::Dot),
    ));

    let delimiter = choice((
        just(',').to(Token::Comma),
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

    let token = choice((float, int, string, operator, delimiter, word))
        .map_with_span(|token, span| (token, span))
        .padded();

    token.repeated().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_token(code: &str) -> Vec<Token<'_>> {
        let parse_result = lexer().parse(code);
        let output = parse_result.into_output();
        match output {
            Some(x) => x.into_iter().map(|(token, _)| token).collect(),
            None => vec![],
        }
    }

    #[test]
    fn int() {
        assert_eq!(to_token("0"), vec![Token::Int(0)]);
        assert_eq!(to_token("7"), vec![Token::Int(7)]);
    }

    #[test]
    fn float() {
        assert_eq!(to_token("0.0"), vec![Token::Float(0.0)]);
        assert_eq!(to_token("0.3"), vec![Token::Float(0.3)]);
    }

    #[test]
    fn string() {
        assert_eq!(to_token(r#""abc""#), vec![Token::String("abc")]);
    }
}
