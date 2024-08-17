use super::*;

pub(crate) fn run_method(
    name: &str,
    receiver: bool,
    args: impl ExactSizeIterator<Item = Object>,
) -> RunMethodResult {
    let args = args.into_iter();
    match name {
        // common methods
        "to_string" => method::to_string(receiver, args),
        _ => RunMethodResult::NotFound {
            receiver_type: TypeFlag::BOOL,
        },
    }
}

mod method {
    use super::*;
    use RunMethodResult::*;

    util_macros::gen_method_macro!(bool);

    method!(to_string, 0, |this, args| {
        let string = UString::from(if this { "true" } else { "false" });
        Ok(Object::String(string))
    });
}
