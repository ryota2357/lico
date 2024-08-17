use super::*;

pub(crate) fn run_method(
    name: &str,
    args: impl ExactSizeIterator<Item = Object>,
) -> RunMethodResult {
    let args = args.into_iter();
    match name {
        // common methods
        "to_string" => method::to_string(args),
        _ => RunMethodResult::NotFound {
            receiver_type: TypeFlag::NIL,
        },
    }
}

mod method {
    use super::*;
    use RunMethodResult::*;

    macro_rules! method {
        ($name:ident, $expected:expr, |$args:ident| $impl:expr) => {
            pub(crate) fn $name(args: impl ExactSizeIterator<Item = Object>) -> RunMethodResult {
                if args.len() != $expected {
                    return arg_error($expected, args.len());
                }
                $impl
            }
        };
    }
    #[cold]
    fn arg_error(expected: u8, got: usize) -> RunMethodResult {
        debug_assert!(got <= u8::MAX as usize);
        let got = got as u8;
        InvalidArgCount { expected, got }
    }

    method!(to_string, 0, |args| {
        let string = UString::from("nil");
        Ok(Object::String(string))
    });
}
