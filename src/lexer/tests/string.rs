mod common;
use common::*;
pub use pretty_assertions::assert_eq;

fn parse_comment(s: &str) -> String {
    let toks = parse_ok(s);
    assert_eq!(toks.len(), 1);
    let (tok, span) = toks[0].clone();
    assert_eq!(span, 0..s.len() as u32);
    match tok {
        Token::Comment(x) => x.to_string(),
        _ => panic!("Expected comment"),
    }
}

fn test_parse_string(s: &str, expected: &str) {
    for s in [format!(r#""{}""#, s), format!(r#"'{}'"#, s)] {
        let toks = parse_ok(&s);
        assert_eq!(toks.len(), 1);
        let (tok, span) = toks[0].clone();
        assert_eq!(span, 0..s.len() as u32);
        let actual = match tok {
            Token::String(x) => x.to_string(),
            _ => panic!("Expected comment"),
        };
        assert_eq!(actual, expected);
    }
}

#[test]
fn comment() {
    assert_eq!(parse_comment("# hoge fuga"), " hoge fuga");
    assert_eq!(parse_comment("# hoge @# fuga"), " hoge @# fuga");
    assert_eq!(
        parse_ok("a #b\n c"),
        vec![
            (Token::Ident("a"), 0..1),
            (Token::Comment("b"), 2..5),
            (Token::Ident("c"), 6..7)
        ]
    );
}

#[test]
fn string() {
    test_parse_string("abc de f", "abc de f");
    test_parse_string("", "");
    test_parse_string("あいうえお", "あいうえお");

    assert_eq!(parse_ok(r#""'""#), vec![(Token::String("'".into()), 0..3)]);
    assert_eq!(parse_ok(r#"'"'"#), vec![(Token::String("\"".into()), 0..3)]);
}

#[test]
fn string_escape() {
    test_parse_string(r"\x41", "A");
    test_parse_string(r"\x7e", "~");
    test_parse_string(r"\x7F", "\x7f");
    test_parse_string(r"\n", "\n");
    test_parse_string(r"\r", "\r");
    test_parse_string(r"\t", "\t");
    test_parse_string(r"\\", "\\");
    test_parse_string(r"\0", "\0");
    test_parse_string(r"\u{3042}", "あ");
    test_parse_string(r"\u{1f600}", "😀");

    test_parse_string(r#"\'"#, "'");
    test_parse_string(r#"\""#, "\"");
}
