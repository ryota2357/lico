use foundation::syntax::token::*;

macro_rules! assert_token {
    ($src:literal, [$( $kind:ident { $len:literal $(, $($field_name:ident: $field_expr:expr),* $(,)? )?} ),*]) => {
        let tokens = lexer::tokenize($src).collect::<Vec<_>>();
        let mut i = 0;
        #[allow(unused_assignments)]
        {$(
            let token = &tokens[i];
            pretty_assertions::assert_eq!(token, &Token {
                len: $len,
                kind: TokenKind::$kind $({ $($field_name: $field_expr),* })?
            });
            i += 1;
        )*}
    };
}

#[test]
fn line_comment() {
    assert_token!("# comment", [LineComment { 9 }]);
}

#[test]
fn whitespace() {
    assert_token!(" \t\n", [Whitespace { 3 }]);
}

#[test]
fn int() {
    assert_token!(
        "42",
        [Int {
            2,
            base: NumBase::Decimal,
            empty_int: false,
        }]
    );
}

#[test]
fn float() {
    assert_token!(
        "3.14",
        [Float {
            4,
            base: NumBase::Decimal,
            empty_exponent: false,
        }]
    );
}

#[test]
fn string() {
    assert_token!(
        r#""foo""#,
        [String {
            5,
            terminated: true,
            quote_kind: QuoteKind::Double,
        }]
    );
    assert_token!(
        r#"'piyo piyo'"#,
        [String {
            11,
            terminated: true,
            quote_kind: QuoteKind::Single,
        }]
    );
}

#[test]
fn bool() {
    assert_token!("true", [True { 4 }]);
    assert_token!("false", [False { 5 }]);
}

#[test]
fn nil() {
    assert_token!("nil", [Nil { 3 }]);
}

#[test]
fn keyword() {
    assert_token!("var", [Var { 3 }]);
    assert_token!("func", [Func { 4 }]);
    assert_token!("if", [If { 2 }]);
    assert_token!("then", [Then { 4 }]);
    assert_token!("elif", [Elif { 4 }]);
    assert_token!("else", [Else { 4 }]);
    assert_token!("for", [For { 3 }]);
    assert_token!("while", [While { 5 }]);
    assert_token!("in", [In { 2 }]);
    assert_token!("do", [Do { 2 }]);
    assert_token!("end", [End { 3 }]);
    assert_token!("return", [Return { 6 }]);
    assert_token!("break", [Break { 5 }]);
    assert_token!("continue", [Continue { 8 }]);
    assert_token!("and", [And { 3 }]);
    assert_token!("or", [Or { 2 }]);
    assert_token!("not", [Not { 3 }]);
}

#[test]
fn symbol() {
    assert_token!("+", [Plus { 1 }]);
    assert_token!("-", [Minus { 1 }]);
    assert_token!("*", [Star { 1 }]);
    assert_token!("/", [Slash { 1 }]);
    assert_token!("%", [Percent { 1 }]);
    assert_token!("&", [Amp { 1 }]);
    assert_token!("|", [Pipe { 1 }]);
    assert_token!("^", [Caret { 1 }]);
    assert_token!("~", [Tilde { 1 }]);
    assert_token!("!", [Bang { 1 }]);
    assert_token!("=", [Eq { 1 }]);
    assert_token!("<", [Lt { 1 }]);
    assert_token!(">", [Gt { 1 }]);
    assert_token!(".", [Dot { 1 }]);
    assert_token!("@", [At { 1 }]);
    assert_token!(",", [Comma { 1 }]);
    assert_token!(":", [Colon { 1 }]);
    assert_token!("(", [OpenParen { 1 }]);
    assert_token!(")", [CloseParen { 1 }]);
    assert_token!("{", [OpenBrace { 1 }]);
    assert_token!("}", [CloseBrace { 1 }]);
    assert_token!("[", [OpenBracket { 1 }]);
    assert_token!("]", [CloseBracket { 1 }]);
    assert_token!("->", [Arrow { 2 }]);
    assert_token!("!=", [BangEq { 2 }]);
    assert_token!("==", [Eq2 { 2 }]);
    assert_token!("<<", [Lt2 { 2 }]);
    assert_token!("<=", [LtEq { 2 }]);
    assert_token!(">>", [Gt2 { 2 }]);
    assert_token!(">=", [GtEq { 2 }]);
    assert_token!("..", [Dot2 { 2 }]);
}

#[test]
fn ident() {
    assert_token!("foo", [Ident { 3 }]);
    assert_token!("varl", [Ident { 4 }]);
}

#[test]
fn invalid_ident() {
    assert_token!("🦀", [InvalidIdent { 4 }]);
}
