use super::*;

pub(crate) fn run_method(
    name: &str,
    receiver: Table,
    args: impl ExactSizeIterator<Item = Object>,
) -> RunMethodResult {
    let args = args.into_iter();
    match name {
        // common methods
        "to_string" => method::to_string(receiver, args),

        // table methods
        "len" => method::len(receiver, args),
        _ => RunMethodResult::NotFound {
            receiver_type: TypeFlag::TABLE,
        },
    }
}

mod method {
    use super::*;
    use RunMethodResult::*;

    util_macros::gen_method_macro!(Table);

    // to_string() -> string
    method!(to_string, 0, |this, args| {
        // TODO: improve
        let string = UString::from(format!("{:?}", this).as_str());
        Ok(Object::String(string))
    });

    // len() -> int
    method!(len, 0, |this, args| Ok(Object::Int(this.len() as i64)));
}
