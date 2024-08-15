mod pms_gc;
use pms_gc::*;

mod ustring;
pub use ustring::UString;

mod array;
pub use array::Array;

mod table;
pub use table::{Table, TableMethod};

mod function;
pub use function::Function;

mod rust_function;
pub use rust_function::RustFunction;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Int(i64),
    Float(f64),
    Bool(bool),
    Nil,
    String(UString),
    Array(Array),
    Table(Table),
    Function(Function),
    RustFunction(RustFunction),
}

fn _size_check() {
    const {
        assert!(size_of::<Object>() == 16);
        assert!(size_of::<Option<Object>>() == 16);
    }
}

macro_rules! impl_into_object_variant {
    ($(impl From<$type:ty> for Object::$variant:ident);* $(;)?) => {
        $(impl From<$type> for Object {
            fn from(value: $type) -> Self {
                Object::$variant(value)
            }
        })*
    };
}
impl_into_object_variant! {
    impl From<i64>     for Object::Int;
    impl From<f64>     for Object::Float;
    impl From<bool>    for Object::Bool;
    impl From<UString> for Object::String;
    impl From<Array>   for Object::Array;
    impl From<Table>   for Object::Table;
}
