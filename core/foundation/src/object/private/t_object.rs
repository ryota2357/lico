use super::Object;
use core::fmt::Debug;

pub(crate) trait TObject: Clone + Debug + PartialEq {
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
