pub use foundation::Token;

pub fn parse_ok(s: &str) -> Vec<(Token, std::ops::Range<u32>)> {
    let (tok, err) = lexer::parse(s);
    assert!(err.is_empty());
    tok.into_iter().map(|(t, s)| (t, s.into_range())).collect()
}
