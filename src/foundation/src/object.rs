mod array;
pub use array::Array;

mod function;
pub use function::Function;

mod table;
pub use table::{Table, TableMethod};

mod uni_string;
pub use uni_string::UniString;

pub mod collections;

mod private;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Int(i64),
    Float(f64),
    Bool(bool),
    Nil,
    String(UniString),
    Array(Array),
    Table(Table),
    Function(Function),
    RustFunction(fn(&[Object]) -> Result<Object, String>),
}

macro_rules! into_object {
    ($($type:ty :-> $variant:ident),* $(,)?) => {
        $(
            impl From<$type> for Object {
                fn from(value: $type) -> Self {
                    Object::$variant(value)
                }
            }
        )*
    };
}
into_object! {
    i64 :-> Int,
    f64 :-> Float,
    bool :-> Bool,
    UniString :-> String,
    Array :-> Array,
    Table :-> Table,
}
