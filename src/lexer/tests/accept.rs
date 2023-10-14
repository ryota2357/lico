mod common;

use common::do_test;
use lexer::Token;

#[test]
fn int() {
    do_test("0", vec![Token::Int(0)]);
    do_test("7", vec![Token::Int(7)]);
    do_test("1234567890", vec![Token::Int(1234567890)]);
    do_test("01", vec![Token::Int(1)]);
    do_test("0010", vec![Token::Int(10)]);
}

#[test]
fn float() {
    do_test("0.0", vec![Token::Float(0.0)]);
    do_test("0.3", vec![Token::Float(0.3)]);
    do_test("12.34", vec![Token::Float(12.34)]);
    do_test("7.0", vec![Token::Float(7.0)]);
    do_test("01.23", vec![Token::Float(1.23)]);
    do_test("0010.00", vec![Token::Float(10.0)]);
}

#[test]
fn string() {
    do_test(r#""abc de g""#, vec![Token::String("abc de g")]);
    do_test(r#""""#, vec![Token::String("")]);

    do_test("'abc de g'", vec![Token::String("abc de g")]);
    do_test("''", vec![Token::String("")]);
}

#[test]
fn bool() {
    do_test("true", vec![Token::Bool(true)]);
    do_test("false", vec![Token::Bool(false)]);
}

#[test]
fn nil() {
    do_test("nil", vec![Token::Nil]);
}

#[test]
fn keyword() {
    do_test("var", vec![Token::Var]);
    do_test("let", vec![Token::Let]);
    do_test("func", vec![Token::Func]);
    do_test("if", vec![Token::If]);
    do_test("then", vec![Token::Then]);
    do_test("elif", vec![Token::Elif]);
    do_test("else", vec![Token::Else]);
    do_test("for", vec![Token::For]);
    do_test("while", vec![Token::While]);
    do_test("in", vec![Token::In]);
    do_test("do", vec![Token::Do]);
    do_test("end", vec![Token::End]);
    do_test("return", vec![Token::Return]);
    do_test("break", vec![Token::Break]);
    do_test("continue", vec![Token::Continue]);
}

#[test]
fn operator() {
    do_test("+", vec![Token::Pluss]);
    do_test("-", vec![Token::Minus]);
    do_test("*", vec![Token::Mul]);
    do_test("/", vec![Token::Div]);
    do_test("%", vec![Token::Mod]);
    do_test("**", vec![Token::Pow]);
    do_test("==", vec![Token::Eq]);
    do_test("!=", vec![Token::NotEq]);
    do_test("<", vec![Token::Less]);
    do_test("<=", vec![Token::LessEq]);
    do_test(">", vec![Token::Greater]);
    do_test(">=", vec![Token::GreaterEq]);
    do_test(".", vec![Token::Dot]);
    do_test("..", vec![Token::SrtJoin]);
    do_test("=", vec![Token::Assign]);
}

#[test]
fn keyword_operator() {
    do_test("and", vec![Token::And]);
    do_test("or", vec![Token::Or]);
    do_test("not", vec![Token::Not]);
}

#[test]
fn delimiter() {
    do_test(",", vec![Token::Comma]);
    do_test(":", vec![Token::Colon]);
    do_test("(", vec![Token::OpenParen]);
    do_test(")", vec![Token::CloseParen]);
    do_test("{", vec![Token::OpenBrace]);
    do_test("}", vec![Token::CloseBrace]);
    do_test("[", vec![Token::OpenBracket]);
    do_test("]", vec![Token::CloseBracket]);
}

#[test]
fn identifier() {
    do_test("abc", vec![Token::Identifier("abc")]);
    do_test("a1", vec![Token::Identifier("a1")]);
    do_test("a_1", vec![Token::Identifier("a_1")]);
    do_test("_foo", vec![Token::Identifier("_foo")]);
    do_test("bar_", vec![Token::Identifier("bar_")]);
}

#[test]
fn attribute() {
    do_test("@abc", vec![Token::Attribute("abc")]);
    do_test("@_a", vec![Token::Attribute("_a")]);
    do_test("@hoge_", vec![Token::Attribute("hoge_")]);
}
