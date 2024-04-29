pub use foundation::syntax::token::*;

#[macro_export]
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
        pretty_assertions::assert_eq!(None, tokens.get(i));
    };
}
