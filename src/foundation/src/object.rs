mod array;
pub use array::Array;

mod function;
pub use function::Function;

mod table;
pub use table::{Table, TableMethod};

mod uni_string;
pub use uni_string::UniString;

mod pms;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Int(i64),
    Float(f64),
    String(UniString),
    Bool(bool),
    Nil,
    Function(Function),
    Array(Array),
    Table(Table),
    RustFunction(fn(&[Object]) -> Result<Object, String>),
}

mod private {
    use super::*;
    use core::fmt::Debug;

    pub trait TObject: Clone + Debug + PartialEq {
        fn into_object(self) -> Object;
        fn as_object(&self) -> &Object;
        fn as_object_mut(&mut self) -> &mut Object;
    }

    impl TObject for Object {
        fn into_object(self) -> Object {
            self
        }
        fn as_object(&self) -> &Object {
            self
        }
        fn as_object_mut(&mut self) -> &mut Object {
            self
        }
    }
}
