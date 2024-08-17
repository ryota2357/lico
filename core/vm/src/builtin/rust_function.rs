use super::*;

pub(crate) fn run_method(
    name: &str,
    receiver: RustFunction,
    args: impl ExactSizeIterator<Item = Object>,
) -> RunMethodResult {
    let args = args.into_iter();
    match name {
        // common methods
        "to_string" => method::to_string(receiver, args),
        _ => RunMethodResult::NotFound {
            receiver_type: TypeFlag::FUNCTION,
        },
    }
}

mod method {
    use super::*;
    use RunMethodResult::*;

    util_macros::gen_method_macro!(RustFunction);

    // to_string() -> string
    method!(to_string, 0, |this, args| {
        // TODO: improve
        let string = UString::from(format!("{:?}", this).as_str());
        Ok(Object::String(string))
    });
}
