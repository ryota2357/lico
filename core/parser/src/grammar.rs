use crate::{parser::*, token_set::TokenSet};
use foundation::syntax::{SyntaxKind, SyntaxKind::*, T};

mod atom;
mod expression;
mod statement;
mod util;

struct EndWith<const C: char>;

pub(crate) fn entry(p: &mut Parser) {
    let m = p.start();
    while !p.at_eof() {
        util::loop_stmt_until_st(p, TokenSet::new(&[]));
        p.eat_trivia();
    }
    m.complete(p, PROGRAM);
}

fn program(p: &mut Parser) {
    let m = p.start();
    util::loop_stmt_until_st(p, TokenSet::one(T![end]));
    p.eat_trivia();
    m.complete(p, PROGRAM);
}

/// Precondition: `assert!(p.at(IDENT))`
fn name(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(IDENT);
    m.complete(p, NAME)
}

/// Precondition: `assert!(p.at(IDENT))`
fn name_path(p: &mut Parser) -> Result<(), EndWith<'.'>> {
    let m = p.start();
    name(p);
    let result = if p.eat(T![.]) {
        if p.at(IDENT) {
            name_path(p)
        } else {
            Err(EndWith::<'.'>)
        }
    } else {
        Ok(())
    };
    m.complete(p, NAME_PATH);
    result
}

/// Precondition: `assert!(p.at(T!['(']))`
fn param_list(p: &mut Parser) {
    let m = p.start();
    p.bump(T!['(']);

    p.eat_trivia();

    while p.current().map(|t| t != T![')']).unwrap_or(false) {
        if p.at(T![,]) {
            p.error("Missing <name>");
            p.bump(T![,]);
            p.eat_trivia();
            continue;
        }
        if p.at(IDENT) {
            name(p);
        } else {
            break;
        }
        p.eat_trivia();
        if !p.eat(T![,]) {
            if p.at(IDENT) {
                p.error("Missing ','");
            } else {
                break;
            }
        }
        p.eat_trivia();
    }

    p.eat_trivia();

    if !p.eat(T![')']) {
        p.error_with(|p| {
            const NEXT_FIRST: TokenSet = statement::STMT_FIRST.unions(&[T![')']]);
            if p.at_ts(NEXT_FIRST) {
                "Missing closing ')'"
            } else {
                let m = p.start();
                util::skip_while_st(p, NEXT_FIRST);
                m.complete(p, ERROR);
                "Expected closing ')'"
            }
        });
    }
    m.complete(p, PARAM_LIST);
}

/// Precondition: `assert!(p.at(T!['(']))`
fn arg_list(p: &mut Parser) {
    let m = p.start();
    p.bump(T!['(']);

    p.eat_trivia();

    while p.current().map(|t| t != T![')']).unwrap_or(false) {
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

    if !p.eat(T![')']) {
        p.error_with(|p| {
            const NEXT_FIRST: TokenSet = statement::STMT_FIRST.unions(&[T![')']]);
            if p.at_ts(NEXT_FIRST) {
                "Missing closing ')'"
            } else {
                let m = p.start();
                util::skip_while_st(p, NEXT_FIRST);
                m.complete(p, ERROR);
                "Expected closing ')'"
            }
        });
    }
    m.complete(p, ARG_LIST);
}
