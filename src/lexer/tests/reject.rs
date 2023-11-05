use lexer::{error::Error, Token};

fn do_test(code: &str, tokens: Vec<Token>, errors: Vec<Error>) {
    let (a_tokens, a_errors) = lexer::parse(code);
    let a_tokens = a_tokens
        .into_iter()
        .map(|(token, _)| token)
        .collect::<Vec<_>>();
    assert_eq!(a_tokens, tokens);
    assert_eq!(a_errors, errors);
}

#[test]
fn invalid_attribute() {
    do_test(
        "@ attr",
        vec![Token::Error('@'), Token::Identifier("attr")],
        vec![Error::invalid_character('@', (0..1).into())],
    );
}

#[test]
fn invalid_string_escape() {
    do_test(
        r#""\a""#,
        vec![Token::String("a".to_string())],
        vec![Error::invalid_escape_sequence(['\\', 'a'], (1..3).into())],
    );
    do_test(
        r#""\u{110000}""#,
        vec![Token::String(" ".to_string())],
        vec![Error::invalid_escape_sequence(
            ['\\', 'u', '{', '1', '1', '0', '0', '0', '0', '}'],
            (1..11).into(),
        )],
    );
}
