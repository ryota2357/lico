use crate::macros::impl_from_variant;
use core::fmt;

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

impl_from_variant! {
    Object {
        Int: i64,
        Float: f64,
        Bool: bool,
        String: UString,
        Array: Array,
        Table: Table,
        Function: Function,
        RustFunction: RustFunction,
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Int(x) => write!(f, "{}", x),
            Object::Float(x) => write!(f, "{}", x),
            Object::Bool(x) => write!(f, "{}", x),
            Object::Nil => write!(f, "nil"),
            Object::String(x) => write!(f, "{}", x),
            // TODO: use for each Display, don't use debug
            Object::Array(x) => write!(f, "{:?}", x),
            Object::Table(x) => write!(f, "{:?}", x),
            Object::Function(x) => write!(f, "{:?}", x),
            Object::RustFunction(x) => write!(f, "{:?}", x),
        }
    }
}
