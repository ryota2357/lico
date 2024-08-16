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
