use super::*;

pub(super) fn skip_while_st(p: &mut Parser, st: TokenSet) {
    while let Some(current) = p.current() {
        if st.contains(current) {
            break;
        }
        p.bump_any();
    }
}

pub(super) fn loop_stmt_until_st(p: &mut Parser, st: TokenSet) {
    loop {
        p.eat_trivia();
        if p.at_ts(st) || p.at_eof() {
            break;
        }
        if p.at_ts(statement::STMT_FIRST) {
            statement::stmt(p);
        } else {
            let m = p.start();
            p.error_with(|p| {
                let current = p.current().unwrap();
                p.bump_any();
                format!("Unexpected token: {:?}", current)
            });
            m.complete(p, ERROR);
        }
    }
}
