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
    Assign,    // =

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
    let int = text::digits(10)
        .slice()
        .from_str()
        .unwrapped()
        .map(Token::Int);

    let float = text::digits(10)
        .then(just('.').then(text::digits(10)))
        .slice()
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
            .ignore_then(none_of("\"").repeated().slice())
            .then_ignore(just('"'));
        let str2 = just('\'')
            .ignore_then(none_of("'").repeated().slice())
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
        just('=').to(Token::Assign),
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
        assert_eq!(to_token("1234567890"), vec![Token::Int(1234567890)]);
        assert_eq!(to_token("01"), vec![Token::Int(1)]);
        assert_eq!(to_token("0010"), vec![Token::Int(10)]);
    }

    #[test]
    fn float() {
        assert_eq!(to_token("0.0"), vec![Token::Float(0.0)]);
        assert_eq!(to_token("0.3"), vec![Token::Float(0.3)]);
        assert_eq!(to_token("12.34"), vec![Token::Float(12.34)]);
        assert_eq!(to_token("7.0"), vec![Token::Float(7.0)]);
        assert_eq!(to_token("01.23"), vec![Token::Float(1.23)]);
        assert_eq!(to_token("0010.00"), vec![Token::Float(10.0)]);
    }

    #[test]
    fn string() {
        assert_eq!(to_token(r#""abc de g""#), vec![Token::String("abc de g")]);
        assert_eq!(to_token(r#""""#), vec![Token::String("")]);

        assert_eq!(to_token("'abc de g'"), vec![Token::String("abc de g")]);
        assert_eq!(to_token("''"), vec![Token::String("")]);
    }

    #[test]
    fn bool() {
        assert_eq!(to_token("true"), vec![Token::Bool(true)]);
        assert_eq!(to_token("false"), vec![Token::Bool(false)]);
    }

    #[test]
    fn nil() {
        assert_eq!(to_token("nil"), vec![Token::Nil]);
    }

    #[test]
    fn keyword() {
        assert_eq!(to_token("var"), vec![Token::Var]);
        assert_eq!(to_token("let"), vec![Token::Let]);
        assert_eq!(to_token("func"), vec![Token::Func]);
        assert_eq!(to_token("if"), vec![Token::If]);
        assert_eq!(to_token("then"), vec![Token::Then]);
        assert_eq!(to_token("elif"), vec![Token::Elif]);
        assert_eq!(to_token("else"), vec![Token::Else]);
        assert_eq!(to_token("for"), vec![Token::For]);
        assert_eq!(to_token("while"), vec![Token::While]);
        assert_eq!(to_token("in"), vec![Token::In]);
        assert_eq!(to_token("do"), vec![Token::Do]);
        assert_eq!(to_token("end"), vec![Token::End]);
        assert_eq!(to_token("return"), vec![Token::Return]);
        assert_eq!(to_token("break"), vec![Token::Break]);
        assert_eq!(to_token("continue"), vec![Token::Continue]);
    }

    #[test]
    fn operator() {
        assert_eq!(to_token("+"), vec![Token::Add]);
        assert_eq!(to_token("-"), vec![Token::Sub]);
        assert_eq!(to_token("*"), vec![Token::Mul]);
        assert_eq!(to_token("/"), vec![Token::Div]);
        assert_eq!(to_token("%"), vec![Token::Mod]);
        assert_eq!(to_token("^"), vec![Token::Pow]);
        assert_eq!(to_token("=="), vec![Token::Eq]);
        assert_eq!(to_token("!="), vec![Token::NotEq]);
        assert_eq!(to_token("<"), vec![Token::Less]);
        assert_eq!(to_token("<="), vec![Token::LessEq]);
        assert_eq!(to_token(">"), vec![Token::Greater]);
        assert_eq!(to_token(">="), vec![Token::GreaterEq]);
        assert_eq!(to_token("."), vec![Token::Dot]);
        assert_eq!(to_token("="), vec![Token::Assign]);
    }

    #[test]
    fn keyword_operator() {
        assert_eq!(to_token("and"), vec![Token::And]);
        assert_eq!(to_token("or"), vec![Token::Or]);
        assert_eq!(to_token("not"), vec![Token::Not]);
    }

    #[test]
    fn delimiter() {
        assert_eq!(to_token(","), vec![Token::Comma]);
        assert_eq!(to_token("("), vec![Token::OpenParen]);
        assert_eq!(to_token(")"), vec![Token::CloseParen]);
        assert_eq!(to_token("{"), vec![Token::OpenBrace]);
        assert_eq!(to_token("}"), vec![Token::CloseBrace]);
        assert_eq!(to_token("["), vec![Token::OpenBracket]);
        assert_eq!(to_token("]"), vec![Token::CloseBracket]);
    }

    #[test]
    fn identifier() {
        assert_eq!(to_token("abc"), vec![Token::Identifier("abc")]);
        assert_eq!(to_token("a1"), vec![Token::Identifier("a1")]);
        assert_eq!(to_token("a_1"), vec![Token::Identifier("a_1")]);
        assert_eq!(to_token("_foo"), vec![Token::Identifier("_foo")]);
        assert_eq!(to_token("bar_"), vec![Token::Identifier("bar_")]);
    }
}
