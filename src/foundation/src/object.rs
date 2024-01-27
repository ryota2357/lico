mod integer;
use integer::*;

mod uni_string;
use uni_string::*;

mod function;
use function::*;

mod array;
use array::*;

mod table;
use table::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Int(Integer),
    Float(f64),
    String(UniString),
    Bool(bool),
    Nil,
    Function(Function),
    Array(Array),
    Table(Table),
    RustFunction(fn(&[Object]) -> Result<Object, String>),
}
