use foundation::syntax::*;

#[test]
fn size_check() {
    use core::mem::size_of;
    assert_eq!(size_of::<SyntaxKind>(), 1);
    assert_eq!(size_of::<token::Token>(), 8);
}
