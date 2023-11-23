use lexer::Token;

fn do_test(code: &str, expected: Vec<Token>) {
    let actual = lexer::parse(code)
        .0
        .into_iter()
        .map(|(token, _)| token)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn do_string_test(string: &str, expected: String) {
    let string1 = format!(r#""{}""#, string);
    let string2 = format!(r#"'{}'"#, string);
    do_test(&string1, vec![Token::String(expected.clone())]);
    do_test(&string2, vec![Token::String(expected)]);
}

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
    do_string_test("abc de f", String::from("abc de f"));
    do_string_test("", String::from(""));
    do_string_test("あいうえお", String::from("あいうえお"));

    do_test(r#""'""#, vec![Token::String("'".to_string())]);
    do_test(r#"'"'"#, vec![Token::String("\"".to_string())]);
}

#[test]
fn string_escape() {
    do_string_test(r"\x41", String::from("A"));
    do_string_test(r"\x7e", String::from("~"));
    do_string_test(r"\x7F", String::from("\x7f"));
    do_string_test(r"\n", String::from("\n"));
    do_string_test(r"\r", String::from("\r"));
    do_string_test(r"\t", String::from("\t"));
    do_string_test(r"\\", String::from("\\"));
    do_string_test(r"\0", String::from("\0"));
    do_string_test(r"\u{3042}", String::from("あ"));
    do_string_test(r"\u{1f600}", String::from("😀"));

    do_test(r"'\''", vec![Token::String("'".to_string())]);
    do_test(r#""\"""#, vec![Token::String("\"".to_string())]);
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
    do_test("*", vec![Token::Star]);
    do_test("/", vec![Token::Div]);
    do_test("%", vec![Token::Mod]);
    do_test("**", vec![Token::Star2]);
    do_test("==", vec![Token::Eq]);
    do_test("!=", vec![Token::NotEq]);
    do_test("<", vec![Token::Less]);
    do_test("<=", vec![Token::LessEq]);
    do_test(">", vec![Token::Greater]);
    do_test(">=", vec![Token::GreaterEq]);
    do_test(".", vec![Token::Dot]);
    do_test("->", vec![Token::Arrow]);
    do_test("..", vec![Token::Dot2]);
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
fn ident() {
    do_test("abc", vec![Token::Ident("abc")]);
    do_test("a1", vec![Token::Ident("a1")]);
    do_test("a_1", vec![Token::Ident("a_1")]);
    do_test("_foo", vec![Token::Ident("_foo")]);
    do_test("bar_", vec![Token::Ident("bar_")]);
}

#[test]
fn attribute() {
    do_test("@abc", vec![Token::Attribute("abc")]);
    do_test("@_a", vec![Token::Attribute("_a")]);
    do_test("@hoge_", vec![Token::Attribute("hoge_")]);
}

#[test]
fn comment() {
    do_test("# hoge fuga", vec![]);
    do_test("# hoge @# fuga", vec![]);
    do_test("a #b\n c", vec![Token::Ident("a"), Token::Ident("c")]);
}
