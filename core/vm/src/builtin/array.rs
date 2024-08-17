use super::*;

pub(crate) fn run_method(
    name: &str,
    receiver: Array,
    args: impl ExactSizeIterator<Item = Object>,
) -> RunMethodResult {
    let args = args.into_iter();
    match name {
        // common methods
        "to_string" => method::to_string(receiver, args),

        // array methods
        "len" => method::len(receiver, args),
        "pop" => method::pop(receiver, args),
        "push" => method::push(receiver, args),
        _ => RunMethodResult::NotFound {
            receiver_type: TypeFlag::ARRAY,
        },
    }
}

mod method {
    use super::*;
    use RunMethodResult::*;

    util_macros::gen_method_macro!(Array);

    method!(len, 0, |this, args| Ok(Object::Int(this.len() as i64)));

    method!(to_string, 0, |this, args| {
        let string = UString::from(format!("{:?}", this).as_str());
        Ok(Object::String(string))
    });

    method!(pop, 0, |this, args| {
        match this.pop() {
            Some(value) => Ok(value),
            None => Ok(Object::Nil),
        }
    });

    method!(push, 1, |this, args| {
        for arg in args {
            this.push(arg);
        }
        Ok(Object::Nil)
    });
}
