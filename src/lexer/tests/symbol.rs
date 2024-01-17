mod common;
use common::*;
pub use pretty_assertions::assert_eq;

#[test]
fn bool() {
    assert_eq!(parse_ok("true"), vec![(Token::Bool(true), 0..4)]);
    assert_eq!(parse_ok("false"), vec![(Token::Bool(false), 0..5)]);
}

#[test]
fn nil() {
    assert_eq!(parse_ok("nil"), vec![(Token::Nil, 0..3)]);
}

#[test]
fn keyword() {
    assert_eq!(parse_ok("var"), vec![(Token::Var, 0..3)]);
    assert_eq!(parse_ok("func"), vec![(Token::Func, 0..4)]);
    assert_eq!(parse_ok("if"), vec![(Token::If, 0..2)]);
    assert_eq!(parse_ok("then"), vec![(Token::Then, 0..4)]);
    assert_eq!(parse_ok("elif"), vec![(Token::Elif, 0..4)]);
    assert_eq!(parse_ok("else"), vec![(Token::Else, 0..4)]);
    assert_eq!(parse_ok("for"), vec![(Token::For, 0..3)]);
    assert_eq!(parse_ok("while"), vec![(Token::While, 0..5)]);
    assert_eq!(parse_ok("in"), vec![(Token::In, 0..2)]);
    assert_eq!(parse_ok("ref"), vec![(Token::Ref, 0..3)]);
    assert_eq!(parse_ok("do"), vec![(Token::Do, 0..2)]);
    assert_eq!(parse_ok("end"), vec![(Token::End, 0..3)]);
    assert_eq!(parse_ok("return"), vec![(Token::Return, 0..6)]);
    assert_eq!(parse_ok("break"), vec![(Token::Break, 0..5)]);
    assert_eq!(parse_ok("continue"), vec![(Token::Continue, 0..8)]);
}

#[test]
fn operator() {
    assert_eq!(parse_ok("+"), vec![(Token::Plus, 0..1)]);
    assert_eq!(parse_ok("-"), vec![(Token::Minus, 0..1)]);
    assert_eq!(parse_ok("*"), vec![(Token::Star, 0..1)]);
    assert_eq!(parse_ok("/"), vec![(Token::Div, 0..1)]);
    assert_eq!(parse_ok("%"), vec![(Token::Mod, 0..1)]);
    assert_eq!(parse_ok("=="), vec![(Token::Eq, 0..2)]);
    assert_eq!(parse_ok("!="), vec![(Token::NotEq, 0..2)]);
    assert_eq!(parse_ok("<"), vec![(Token::Less, 0..1)]);
    assert_eq!(parse_ok("<="), vec![(Token::LessEq, 0..2)]);
    assert_eq!(parse_ok(">"), vec![(Token::Greater, 0..1)]);
    assert_eq!(parse_ok(">="), vec![(Token::GreaterEq, 0..2)]);
    assert_eq!(parse_ok("."), vec![(Token::Dot, 0..1)]);
    assert_eq!(parse_ok("->"), vec![(Token::Arrow, 0..2)]);
    assert_eq!(parse_ok(".."), vec![(Token::Dot2, 0..2)]);
    assert_eq!(parse_ok("="), vec![(Token::Assign, 0..1)]);
}

#[test]
fn keyword_operator() {
    assert_eq!(parse_ok("and"), vec![(Token::And, 0..3)]);
    assert_eq!(parse_ok("or"), vec![(Token::Or, 0..2)]);
    assert_eq!(parse_ok("not"), vec![(Token::Not, 0..3)]);
}

#[test]
fn delimiter() {
    assert_eq!(parse_ok(","), vec![(Token::Comma, 0..1)]);
    assert_eq!(parse_ok(":"), vec![(Token::Colon, 0..1)]);
    assert_eq!(parse_ok("("), vec![(Token::OpenParen, 0..1)]);
    assert_eq!(parse_ok(")"), vec![(Token::CloseParen, 0..1)]);
    assert_eq!(parse_ok("{"), vec![(Token::OpenBrace, 0..1)]);
    assert_eq!(parse_ok("}"), vec![(Token::CloseBrace, 0..1)]);
    assert_eq!(parse_ok("["), vec![(Token::OpenBracket, 0..1)]);
    assert_eq!(parse_ok("]"), vec![(Token::CloseBracket, 0..1)]);
}

#[test]
fn ident() {
    assert_eq!(parse_ok("abc"), vec![(Token::Ident("abc"), 0..3)]);
    assert_eq!(parse_ok("a1"), vec![(Token::Ident("a1"), 0..2)]);
    assert_eq!(parse_ok("a_1"), vec![(Token::Ident("a_1"), 0..3)]);
    assert_eq!(parse_ok("_foo"), vec![(Token::Ident("_foo"), 0..4)]);
    assert_eq!(parse_ok("bar_"), vec![(Token::Ident("bar_"), 0..4)]);
    assert_eq!(parse_ok("あ"), vec![(Token::Ident("あ"), 0..3)]);
}

#[test]
fn attribute() {
    assert_eq!(parse_ok("@abc"), vec![(Token::Attribute("abc"), 0..4)]);
    assert_eq!(parse_ok("@_a"), vec![(Token::Attribute("_a"), 0..3)]);
    assert_eq!(parse_ok("@hoge_"), vec![(Token::Attribute("hoge_"), 0..6)]);
}
