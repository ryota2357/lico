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
