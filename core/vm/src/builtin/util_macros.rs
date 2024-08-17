#[macro_export]
macro_rules! __gen_method_macro {
    ($receiver_ty:ty) => {
        macro_rules! method {
            ($name:ident, $param_len:literal, |$this:ident, $args:ident| $impl:expr) => {
                #[allow(unused_mut)]
                pub(crate) fn $name(
                    mut $this: $receiver_ty,
                    mut $args: impl ExactSizeIterator<Item = Object>,
                ) -> $crate::builtin::RunMethodResult {
                    if $args.len() != $param_len {
                        return __arg_error($param_len, $args.len());
                    }
                    $impl
                }
            };
        }
        #[cold]
        fn __arg_error(expected: u8, got: usize) -> $crate::builtin::RunMethodResult {
            debug_assert!(expected <= u8::MAX);
            let got = got as u8;
            $crate::builtin::RunMethodResult::InvalidArgCount { expected, got }
        }
    };
}
pub(crate) use __gen_method_macro as gen_method_macro;
