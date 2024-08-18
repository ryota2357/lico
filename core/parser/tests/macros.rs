#[macro_export]
macro_rules! test {
    ($name:ident, $path:literal) => {
        #[test]
        fn $name() {
            let source = std::include_str!($path);
            let (green, err) = parser::parse(source, lexer::tokenize(source));
            assert!(err.is_empty(), "{:?}", err);
            insta::with_settings!({
                prepend_module_to_snapshot => false,
                omit_expression => true,
                description => stringify!($name),
            }, {
                insta::assert_snapshot!(format!("{:#?}", foundation::syntax::SyntaxNode::new_root(green)));
            });
        }
    };
}
