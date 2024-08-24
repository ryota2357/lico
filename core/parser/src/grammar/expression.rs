use super::*;

pub(super) const EXPR_FIRST: TokenSet = ATOM_EXPR_FIRST.unions(&[
    T![+],   // prefix-op
    T![-],   // prefix-op
    T![not], // prefix-op
    T![~],   // prefix-op
    T![!],   // Invalid prefix-op, for error recovery
]);

/// Precondition: `assert!(p.at_ts(EXPR_FIRST))`
pub(super) fn expr(p: &mut Parser) {
    assert!(p.at_ts(EXPR_FIRST));
    expr_bp(p, 0);
}

fn expr_bp(p: &mut Parser, min_bp: u8) -> Option<CompletedMarker> {
    if !p.at_ts(EXPR_FIRST) {
        p.error("Expected <expr>");
        return None;
    }

    let mut lhs = lhs(p);
    loop {
        p.eat_trivia();
        let Some(current) = p.current() else {
            break;
        };
        let m;
        let r_bp = match infix_op_binding_power(current) {
            Some((l_bp, r_bp)) => {
                if l_bp < min_bp {
                    break;
                }
                m = lhs.precede(p);
                p.bump_any();
                r_bp
            }
            None => match current {
                T![~] if p.nth_at(1, T![=]) => {
                    todo!("~= error")
                }
                _ => break,
            },
        };
        p.eat_trivia();
        expr_bp(p, r_bp);
        lhs = m.complete(p, BINARY_EXPR);
    }
    Some(lhs)
}

fn lhs(p: &mut Parser) -> CompletedMarker {
    assert!(p.at_ts(EXPR_FIRST));

    // Precedence (binding power) of operators in Lico is strongest for prefix-op, followed by postfix-op, and finally infix-op.
    // So, we don't care about precedence to parse prefix-op and postfix-op; we can just parse them in the order we encounter them.

    // SAFETY: `at_ts(EXPR_FIRST)` is true, so `p.current()` is not None.
    match unsafe { p.current().unwrap_unchecked() } {
        T![+] | T![-] | T![not] | T![~] => {
            let m = p.start();
            p.bump_any();
            p.eat_trivia();
            expr_bp(p, 255); // prefix-op has the highest precedence
            m.complete(p, PREFIX_EXPR)
        }
        T![!] => {
            let m = p.start();
            p.error_with(|p| {
                p.bump(T![!]);
                "Should use `not` for logical negation or `~` for bitwise negation"
            });
            p.eat_trivia();
            expr_bp(p, 255); // same as normal prefix-op
            m.complete(p, PREFIX_EXPR)
        }
        _ => {
            let mut lhs = atom_expr(p);
            p.eat_trivia();
            while let Some(current) = p.current() {
                // postfix-op
                lhs = match current {
                    T![.] => dot_expr(p, lhs),
                    T!['['] => index_expr(p, lhs),
                    T!['('] => call_expr(p, lhs),
                    T![->] => method_call_expr(p, lhs),
                    _ => break,
                };
            }
            lhs
        }
    }
}

/// |        Precedence        | Associativity |     Operators     |
/// | -----------------------  | ------------- | ----------------- |
/// | 13: Unary Postfix        |    postfix    | .x, [], (), ->x() |
/// | 12: Unary Prefix         |    prefix     | +, -, not         |
/// | 11: Multiplicative       |   left infix  | *, /, %           |
/// | 10: Additive             |   left infix  | +, -              |
/// |  9: String concatenation |  right infix  | ..                |
/// |  8: Shift                |   left infix  | <<, >>            |
/// |  7: Bitwise-AND          |   left infix  | &                 |
/// |  6: Bitwise-XOR          |   left infix  | ^                 |
/// |  5: Bitwise-OR           |   left infix  | |                 |
/// |  4: Relational           |   left infix  | <, <=, >, >=      |
/// |  3: Equality             |   left infix  | ==, !=            |
/// |  2: Logical-AND          |   left infix  | and               |
/// |  1: Logical-OR           |   left infix  | or                |
/// |  0: Assignment           |   right infix | =                 |
const fn infix_op_binding_power(kind: SyntaxKind) -> Option<(u8, u8)> {
    const fn left(precedence: u8) -> (u8, u8) {
        (2 * precedence + 1, 2 * precedence + 2)
    }
    const fn right(precedence: u8) -> (u8, u8) {
        (2 * precedence + 2, 2 * precedence + 1)
    }
    #[rustfmt::skip]
    let bp = match kind {
        T![*] | T![/] | T![%]           => left(11),
        T![+] | T![-]                   => left(10),
        T![..]                          => right(9),
        T![<<] | T![>>]                 => left(8),
        T![&]                           => left(7),
        T![^]                           => left(6),
        T![|]                           => left(5),
        T![<] | T![<=] | T![>] | T![>=] => left(4),
        T![==] | T![!=]                 => left(3),
        T![and]                         => left(2),
        T![or]                          => left(1),
        T![=]                           => right(0),
        _ => return None,
    };
    Some(bp)
}

const ATOM_EXPR_FIRST: TokenSet = atom::LITERA_FIRST.unions(&[
    T![do],   // do_expr
    T![if],   // if_expr
    T!['('],  // paren_expr
    T!['{'],  // atom::table_const
    T!['['],  // atom::array_const
    T![func], // atom::func_const
    IDENT,    // atom::local_var
]);

fn atom_expr(p: &mut Parser) -> CompletedMarker {
    assert!(p.at_ts(ATOM_EXPR_FIRST));

    // SAFETY: `p.at_ts(..)` is true, so `p.current()` is not None.
    match unsafe { p.current().unwrap_unchecked() } {
        T![do] => do_expr(p),
        T![if] => if_expr(p),
        T!['('] => paren_expr(p),
        T!['{'] => atom::table_const(p),
        T!['['] => atom::array_const(p),
        T![func] => atom::func_const(p),
        IDENT => atom::local_var(p),
        c if atom::LITERA_FIRST.contains(c) => atom::literal(p),
        _ => unreachable!(),
    }
}

fn dot_expr(p: &mut Parser, lhs: CompletedMarker) -> CompletedMarker {
    let m = lhs.precede(p);
    p.bump(T![.]);
    p.eat_trivia();
    if p.at(IDENT) {
        name(p);
    } else {
        p.error("Expected <name> after `.`")
    }
    m.complete(p, FIELD_EXPR)
}

fn index_expr(p: &mut Parser, lhs: CompletedMarker) -> CompletedMarker {
    let m = lhs.precede(p);
    p.bump(T!['[']);
    p.eat_trivia();
    expr(p);
    p.eat_trivia();
    if !p.at(T![']']) {
        p.error("Expected `]` to close index expression");
    }
    m.complete(p, INDEX_EXPR)
}

fn call_expr(p: &mut Parser, lhs: CompletedMarker) -> CompletedMarker {
    assert!(p.at(T!['(']));
    let m = lhs.precede(p);
    arg_list(p);
    m.complete(p, CALL_EXPR)
}

fn method_call_expr(p: &mut Parser, lhs: CompletedMarker) -> CompletedMarker {
    let m = lhs.precede(p);
    p.bump(T![->]);
    p.eat_trivia();
    if p.at(IDENT) {
        name(p);
    } else {
        p.error("Expected <name> after `->`")
    }
    p.eat_trivia();
    if p.at(T!['(']) {
        arg_list(p);
    } else {
        todo!("Do recovery")
    }
    m.complete(p, METHOD_CALL_EXPR)
}

fn do_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(T![do]);
    program(p);
    if !p.eat(T![end]) {
        p.error("Missing 'end' keyword");
    }
    m.complete(p, DO_EXPR)
}

fn if_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(T![if]);
    p.eat_trivia();
    if p.at_ts(EXPR_FIRST) {
        expr(p);
    } else {
        p.error("Expected <expr>");
    }
    if !p.eat(T![then]) {
        todo!("error recovery")
    }
    if_expr_branch_program(p);
    match p.current() {
        Some(T![elif]) => {
            while p.at(T![elif]) {
                elif_branch(p);
            }
            if p.at(T![else]) {
                else_branch(p);
            }
        }
        Some(T![else]) => {
            else_branch(p);
        }
        _ => {}
    }
    if !p.eat(T![end]) {
        p.error("Missing 'end' keyword");
    }
    m.complete(p, IF_EXPR)
}

fn paren_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(T!['(']);
    p.eat_trivia();
    if p.at_ts(EXPR_FIRST) {
        expr(p);
        p.eat_trivia();
    } else {
        p.error("Expected <expr>, or use `nil` for unit literal");
    }
    if !p.eat(T![')']) {
        p.error("Missing closing ')'");
    }
    m.complete(p, PAREN_EXPR)
}

fn elif_branch(p: &mut Parser) {
    let m = p.start();
    p.bump(T![elif]);
    p.eat_trivia();
    if p.at_ts(EXPR_FIRST) {
        expr(p);
    } else {
        p.error("Expected <expr>");
    }
    p.eat_trivia();
    if !p.eat(T![then]) {
        p.error("Missing 'then' keyword");
    }
    if_expr_branch_program(p);
    m.complete(p, ELIF_BRANCH);
}

fn else_branch(p: &mut Parser) {
    let m = p.start();
    p.bump(T![else]);
    if_expr_branch_program(p);
    m.complete(p, ELSE_BRANCH);
}

fn if_expr_branch_program(p: &mut Parser) {
    let m = p.start();
    util::loop_stmt_until_st(p, TokenSet::new(&[T![end], T![elif], T![else]]));
    p.eat_trivia();
    m.complete(p, PROGRAM);
}
