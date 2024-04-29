mod common;
use common::*;

#[test]
fn spaced_one_ident() {
    assert_token!("a ", [Ident { 1 }, Whitespace { 1 }]);
    assert_token!(" b", [Whitespace { 1 }, Ident { 1 }]);
    assert_token!(" c ", [Whitespace { 1 }, Ident { 1 }, Whitespace { 1 }]);
    assert_token!("あ ", [Ident { 3 }, Whitespace { 1 }]);
    assert_token!("\nい", [Whitespace { 1 }, Ident { 3 }]);
    assert_token!(" う\n", [Whitespace { 1 }, Ident { 3 }, Whitespace { 1 }]);
}

#[test]
fn spaced_multi_ident() {
    assert_token!("foo ", [Ident { 3 }, Whitespace { 1 }]);
    assert_token!("　bar", [Whitespace { 3 }, Ident { 3 }]);
    assert_token!("　baz ", [Whitespace { 3 }, Ident { 3 }, Whitespace { 1 }]);
    assert_token!("あ ", [Ident { 3 }, Whitespace { 1 }]);
    assert_token!("\nい", [Whitespace { 1 }, Ident { 3 }]);
    assert_token!(" う\n", [Whitespace { 1 }, Ident { 3 }, Whitespace { 1 }]);
}

#[test]
fn spaced_keyword() {
    assert_token!("var ", [Var { 3 }, Whitespace { 1 }]);
}

#[test]
fn keyword_like() {
    // "true"
    assert_token!("tru ", [Ident { 3 }, Whitespace { 1 }]);
    assert_token!("tr@", [Ident { 2 }, At { 1 }]);

    // "false"
    assert_token!("fals ", [Ident { 4 }, Whitespace { 1 }]);
    assert_token!("fal@", [Ident { 3 }, At { 1 }]);

    // "nil"
    assert_token!("ni ", [Ident { 2 }, Whitespace { 1 }]);
    assert_token!("n@", [Ident { 1 }, At { 1 }]);

    // "var"
    assert_token!("va ", [Ident { 2 }, Whitespace { 1 }]);
    assert_token!("v@", [Ident { 1 }, At { 1 }]);

    // "func"
    assert_token!("fun ", [Ident { 3 }, Whitespace { 1 }]);
    assert_token!("fu@", [Ident { 2 }, At { 1 }]);

    // "if"
    assert_token!("i ", [Ident { 1 }, Whitespace { 1 }]);

    // "then"
    assert_token!("the ", [Ident { 3 }, Whitespace { 1 }]);
    assert_token!("th@", [Ident { 2 }, At { 1 }]);

    // "elif"
    assert_token!("eli ", [Ident { 3 }, Whitespace { 1 }]);
    assert_token!("el@", [Ident { 2 }, At { 1 }]);

    // "else"
    assert_token!("els ", [Ident { 3 }, Whitespace { 1 }]);
    assert_token!("el@", [Ident { 2 }, At { 1 }]);

    // "for"
    assert_token!("fo ", [Ident { 2 }, Whitespace { 1 }]);
    assert_token!("f@", [Ident { 1 }, At { 1 }]);

    // "while"
    assert_token!("whil ", [Ident { 4 }, Whitespace { 1 }]);
    assert_token!("wh@", [Ident { 2 }, At { 1 }]);

    // "in"
    assert_token!("i ", [Ident { 1 }, Whitespace { 1 }]);
    assert_token!("i@", [Ident { 1 }, At { 1 }]);

    // "do"
    assert_token!("d ", [Ident { 1 }, Whitespace { 1 }]);

    // "end"
    assert_token!("en ", [Ident { 2 }, Whitespace { 1 }]);
    assert_token!("e@", [Ident { 1 }, At { 1 }]);

    // "return"
    assert_token!("retur ", [Ident { 5 }, Whitespace { 1 }]);
    assert_token!("ret@", [Ident { 3 }, At { 1 }]);

    // "break"
    assert_token!("brea ", [Ident { 4 }, Whitespace { 1 }]);
    assert_token!("bre@", [Ident { 3 }, At { 1 }]);

    // "continue"
    assert_token!("continu ", [Ident { 7 }, Whitespace { 1 }]);
    assert_token!("cont@", [Ident { 4 }, At { 1 }]);

    // "and"
    assert_token!("an ", [Ident { 2 }, Whitespace { 1 }]);
    assert_token!("a@", [Ident { 1 }, At { 1 }]);

    // "or"
    assert_token!("o ", [Ident { 1 }, Whitespace { 1 }]);

    // "not"
    assert_token!("no ", [Ident { 2 }, Whitespace { 1 }]);
    assert_token!("n@", [Ident { 1 }, At { 1 }]);
}
