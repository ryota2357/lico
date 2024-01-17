mod common;
use common::*;
pub use pretty_assertions::assert_eq;

fn parse_int(s: &str) -> (i64, std::ops::Range<u32>) {
    let toks = parse_ok(s);
    assert_eq!(toks.len(), 1);
    let (tok, span) = toks[0].clone();
    match tok {
        Token::Int(x) => (x, span),
        _ => panic!("Expected int literal"),
    }
}

fn parse_float(s: &str) -> (f64, std::ops::Range<u32>) {
    let toks = parse_ok(s);
    assert_eq!(toks.len(), 1);
    let (tok, span) = toks[0].clone();
    match tok {
        Token::Float(x) => (x, span),
        _ => panic!("Expected float literal"),
    }
}

#[test]
fn int_radix_2() {
    assert_eq!(parse_int("0b0"), (0, 0..3));
    assert_eq!(parse_int("0b01"), (1, 0..4));
    assert_eq!(parse_int("0b001"), (1, 0..5));
    assert_eq!(parse_int("0b101"), (5, 0..5));
}

#[test]
fn int_radix_8() {
    assert_eq!(parse_int("0o0"), (0, 0..3));
    assert_eq!(parse_int("0o01"), (1, 0..4));
    assert_eq!(parse_int("0o003"), (3, 0..5));
    assert_eq!(parse_int("0o12345670"), (0o12345670, 0..10));
}

#[test]
fn int_radix_10() {
    assert_eq!(parse_int("0"), (0, 0..1));
    assert_eq!(parse_int("0 "), (0, 0..1));
    assert_eq!(parse_int("01"), (1, 0..2));
    assert_eq!(parse_int("003"), (3, 0..3));
    assert_eq!(parse_int("1234567890"), (1234567890, 0..10));
}

#[test]
fn int_radix_16() {
    assert_eq!(parse_int("0x0"), (0, 0..3));
    assert_eq!(parse_int("0x01"), (1, 0..4));
    assert_eq!(parse_int("0x003"), (3, 0..5));
    assert_eq!(parse_int("0x123456789abcdef0"), (0x123456789abcdef0, 0..18));
}

#[test]
fn float() {
    assert_eq!(parse_float("0.0"), (0.0, 0..3));
    assert_eq!(parse_float("0.123"), (0.123, 0..5));
    assert_eq!(parse_float("123.456"), (123.456, 0..7));
    assert_eq!(parse_float("1."), (1., 0..2));
}
