use super::*;

pub(super) const LITERA_FIRST: TokenSet =
    TokenSet::new(&[INT, FLOAT, STRING, T![true], T![false], T![nil]]);

/// Precondition: `assert!(p.at_ts(LITERA_FIRST))`
pub(super) fn literal(p: &mut Parser) -> CompletedMarker {
    assert!(p.at_ts(LITERA_FIRST));
    let m = p.start();
    p.bump_any();
    m.complete(p, LITERAL)
}

/// Precondition: `assert!(p.at(T!['[']))`
pub(super) fn array_const(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(T!['[']);

    p.eat_trivia();

    while p.current().map(|t| t != T![']']).unwrap_or(false) {
        if p.at(T![,]) {
            p.error("Missing <expr>");
            p.bump(T![,]);
            p.eat_trivia();
            continue;
        }
        if p.at_ts(expression::EXPR_FIRST) {
            expression::expr(p);
        } else {
            break;
        }
        p.eat_trivia();
        if !p.eat(T![,]) {
            if p.at_ts(expression::EXPR_FIRST) {
                p.error("Missing ','");
            } else {
                break;
            }
        }
        p.eat_trivia();
    }

    p.eat_trivia();

    if !p.eat(T![']']) {
        p.error_with(|p| {
            const NEXT_FIRST: TokenSet = statement::STMT_FIRST.unions(&[T![']']]);
            if p.at_ts(NEXT_FIRST) {
                "Missing closing ']'"
            } else {
                let m = p.start();
                util::skip_while_st(p, NEXT_FIRST);
                m.complete(p, ERROR);
                "Expected closing ']'"
            }
        });
    }
    m.complete(p, ARRAY_CONST)
}

pub(super) fn table_const(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(T!['{']);

    p.eat_trivia();

    while p.current().map(|t| t != T!['}']).unwrap_or(false) {
        if p.at(T![,]) {
            p.error("Missing <field>");
            p.bump(T![,]);
            p.eat_trivia();
            continue;
        }
        const FIELD_ELEM: TokenSet =
            TokenSet::new(&[T![func], IDENT, T![=]]).union(expression::EXPR_FIRST);
        if p.at_ts(FIELD_ELEM) {
            table_field(p);
        } else {
            break;
        }
        p.eat_trivia();
        if !p.eat(T![,]) {
            if p.at_ts(FIELD_ELEM) {
                p.error("Missing ','");
            } else {
                break;
            }
        }
        p.eat_trivia();
    }

    p.eat_trivia();

    m.complete(p, TABLE_CONST)
}

fn table_field(p: &mut Parser) {
    let m = p.start();
    if p.eat(T![func]) {
        p.eat_trivia();
    }
    table_filed_name(p);
    p.eat_trivia();
    match p.current() {
        Some(T![=]) => {
            p.bump(T![=]);
        }
        Some(T![:]) => p.error_with(|p| {
            p.bump(T![:]);
            "Should use '=' instead of ':'"
        }),
        Some(t) if expression::EXPR_FIRST.contains(t) => {
            p.error("Missing '='");
        }
        _ => {
            todo!();
        }
    }
    p.eat_trivia();
    if p.at_ts(expression::EXPR_FIRST) {
        expression::expr(p);
    } else {
        p.error("Missing <expr>");
    }
    m.complete(p, TABLE_FIELD);
}

fn table_filed_name(p: &mut Parser) {
    let m = p.start();
    match p.current() {
        Some(IDENT) => {
            p.bump(IDENT);
            m.complete(p, TABLE_FIELD_NAME_IDENT);
        }
        Some(T!['[']) => {
            p.bump(T!['[']);
            p.eat_trivia();
            if p.at_ts(expression::EXPR_FIRST) {
                expression::expr(p);
            } else {
                p.error("Missing <expr>");
            }
            p.eat_trivia();
            if !p.eat(T![']']) {
                p.error("Missing closing ']'");
            }
            m.complete(p, TABLE_FIELD_NAME_EXPR);
        }
        _ => {
            p.error("Missing <field-name>");
        }
    }
}

/// Precondition: `assert!(p.at(IDENT))`
pub(super) fn func_const(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(T![func]);

    p.eat_trivia();

    if p.at(T!['(']) {
        param_list(p);
    } else {
        todo!();
    }

    program(p);

    if !p.eat(T![end]) {
        p.error("Missing 'end'");
    }
    m.complete(p, FUNC_CONST)
}

/// Precondition: `assert!(p.at(IDENT))`
pub(super) fn local_var(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(IDENT);
    m.complete(p, LOCAL_VAR)
}
