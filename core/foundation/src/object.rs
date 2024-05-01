mod ustring;
pub use ustring::UString;

mod array;
pub type Array = array::Array<Object>;

mod table;
pub use table::TableMethod;
pub type Table = table::Table<Object>;

mod function;
pub use function::Function;

mod rust_function;
pub use rust_function::RustFunction;

mod private;

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

const fn _static_assert_size() {
    use std::mem::size_of;
    const {
        assert!(size_of::<Object>() == 16);
        assert!(size_of::<UString>() == 8);
        assert!(size_of::<Array>() == 8);
        assert!(size_of::<Table>() == 8);
        assert!(size_of::<Function>() == 8);
        assert!(size_of::<RustFunction>() == 8);
    }
}

macro_rules! into_object_variant {
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
into_object_variant! {
    i64 :-> Int,
    f64 :-> Float,
    bool :-> Bool,
    UString :-> String,
    Array :-> Array,
    Table :-> Table,
}
impl From<&str> for Object {
    fn from(value: &str) -> Self {
        Object::String(UString::from(value))
    }
}
