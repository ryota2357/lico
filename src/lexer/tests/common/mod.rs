pub fn do_test(code: &str, expected: Vec<lexer::Token>) {
    let actual = lexer::parse(code)
        .into_iter()
        .map(|(token, _)| token)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
