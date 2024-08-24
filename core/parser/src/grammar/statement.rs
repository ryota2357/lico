use super::*;

pub(super) const STMT_FIRST: TokenSet = expression::EXPR_FIRST.unions(&[
    T![var],      // var_stmt
    T![func],     // func_stmt
    T![for],      // for_stmt
    T![while],    // while_stmt
    T![return],   // return_stmt
    T![break],    // break_stmt
    T![continue], // continue_stmt
]);

/// Precondition: `assert!(p.at_ts(STMT_FIRST))`
pub(super) fn stmt(p: &mut Parser) {
    assert!(p.at_ts(STMT_FIRST));

    // SAFETY: `p.at_ts(..)` is true, so `p.current()` is not None.
    match unsafe { p.current().unwrap_unchecked() } {
        T![var] => var_stmt(p),
        T![func] => func_stmt(p),
        T![for] => for_stmt(p),
        T![while] => while_stmt(p),
        T![return] => return_stmt(p),
        T![break] => break_stmt(p),
        T![continue] => continue_stmt(p),
        t if expression::EXPR_FIRST.contains(t) => {
            let m = p.start();
            expression::expr(p);
            m.complete(p, EXPR_STMT);
        }
        _ => unreachable!(),
    }
}

fn var_stmt(p: &mut Parser) {
    let m = p.start();
    p.bump(T![var]);

    p.eat_trivia();

    if p.at(IDENT) {
        name(p);
    } else {
        p.error_with(|p| -> std::borrow::Cow<_> {
            match p.current() {
                Some(T![=]) | None => "Missing <name>".into(),
                Some(..) => {
                    let m = p.start();
                    p.bump_any();
                    m.complete(p, ERROR);
                    format!("Expected <name>, but found {:?}", p.current()).into()
                }
            }
        });
    }

    p.eat_trivia();

    if p.eat(T![=]) {
        p.eat_trivia();
        expression::expr(p);
    } else {
        p.error("Missing '= <expr>': Variables must be initialized");
    }

    m.complete(p, VAR_STMT);
}

fn func_stmt(p: &mut Parser) {
    let m = p.start();
    p.bump(T![func]);

    p.eat_trivia();

    if p.at(IDENT) {
        name_path(p).unwrap_or_else(|_: EndWith<'.'>| {
            p.error_with(|_p| {
                // TODO: skip some tokens ?
                "Missing field name"
            })
        });
    }

    p.eat_trivia();

    if p.eat(T![->]) {
        p.eat_trivia();
        if p.at(IDENT) {
            name(p);
        } else {
            p.error_with(|p| {
                const NEXT: TokenSet = statement::STMT_FIRST.unions(&[T![end], T!['(']]);
                if p.at_ts(NEXT) {
                    "Missing <name>"
                } else {
                    let m = p.start();
                    util::skip_while_st(p, NEXT);
                    m.complete(p, ERROR);
                    "Expected <name>"
                }
            })
        }
    }

    p.eat_trivia();

    if p.at(T!['(']) {
        param_list(p);
    } else {
        p.error("Missing function parameters");
    }

    program(p);

    if !p.eat(T![end]) {
        p.error("Missing 'end' keyword");
    }
    m.complete(p, FUNC_STMT);
}

fn for_stmt(p: &mut Parser) {
    let m = p.start();
    p.bump(T![for]);

    p.eat_trivia();

    if p.at(IDENT) {
        name(p);
    } else {
        todo!("Error recovery");
    }

    p.eat_trivia();

    if !p.eat(T![in]) {
        todo!("Error recovery");
    }

    p.eat_trivia();

    if p.at_ts(expression::EXPR_FIRST) {
        expression::expr(p);
    } else {
        todo!("Error recovery");
    }

    p.eat_trivia();

    if !p.eat(T![do]) {
        todo!("Error recovery");
    }

    program(p);

    if !p.eat(T![end]) {
        p.error("Missing 'end' keyword");
    }
    m.complete(p, FOR_STMT);
}

fn while_stmt(p: &mut Parser) {
    let m = p.start();
    p.bump(T![while]);

    p.eat_trivia();

    if p.at_ts(expression::EXPR_FIRST) {
        expression::expr(p);
    } else {
        todo!("Error recovery");
    }

    p.eat_trivia();

    if !p.eat(T![do]) {
        todo!("Error recovery");
    }

    program(p);

    if !p.eat(T![end]) {
        p.error("Missing 'end' keyword");
    }
    m.complete(p, WHILE_STMT);
}

fn return_stmt(p: &mut Parser) {
    let m = p.start();
    p.bump(T![return]);
    p.eat_trivia();
    if p.at_ts(expression::EXPR_FIRST) {
        expression::expr(p);
    }
    m.complete(p, RETURN_STMT);
}

fn break_stmt(p: &mut Parser) {
    let m = p.start();
    p.bump(T![break]);
    m.complete(p, BREAK_STMT);
}

fn continue_stmt(p: &mut Parser) {
    let m = p.start();
    p.bump(T![continue]);
    m.complete(p, CONTINUE_STMT);
}
