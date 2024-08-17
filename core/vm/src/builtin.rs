use bitflags::bitflags;
use core::fmt;
use foundation::object::{self, *};

use crate::EXCEPTION_LOG;

mod util_macros;

pub(crate) mod array;
pub(crate) mod bool;
pub(crate) mod float;
pub(crate) mod function;
pub(crate) mod int;
pub(crate) mod nil;
pub(crate) mod rust_function;
pub(crate) mod string;
pub(crate) mod table;

pub(crate) enum RunMethodResult {
    Ok(Object),
    NotFound {
        receiver_type: TypeFlag,
    },
    InvalidArgCount {
        expected: u8, // not including the receiver
        got: u8,      // not including the receiver
    },
    InvalidArgType {
        index: u8, // not including the receiver
        expected: TypeFlag,
        got: TypeFlag,
    },
    ExceptionOccurred,
}

bitflags! {
    pub(crate) struct TypeFlag: u8 {
        const INT      = 1 << 0;
        const FLOAT    = 1 << 1;
        const BOOL     = 1 << 2;
        const NIL      = 1 << 3;
        const STRING   = 1 << 4;
        const ARRAY    = 1 << 5;
        const TABLE    = 1 << 6;
        const FUNCTION = 1 << 7;
    }
}

impl fmt::Display for TypeFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        bitflags::parser::to_writer_strict(self, f)
    }
}

impl From<&Object> for TypeFlag {
    fn from(obj: &Object) -> Self {
        match obj {
            Object::Int(_) => TypeFlag::INT,
            Object::Float(_) => TypeFlag::FLOAT,
            Object::Bool(_) => TypeFlag::BOOL,
            Object::Nil => TypeFlag::NIL,
            Object::String(_) => TypeFlag::STRING,
            Object::Array(_) => TypeFlag::ARRAY,
            Object::Table(_) => TypeFlag::TABLE,
            Object::Function(_) | Object::RustFunction(_) => TypeFlag::FUNCTION,
        }
    }
}
