use super::*;

#[derive(Default, Debug, PartialEq)]
pub struct Stack {
    vec: Vec<StackValue>,
}

impl Stack {
    #[inline]
    pub const fn new() -> Self {
        Self { vec: Vec::new() }
    }

    #[inline]
    pub fn push(&mut self, value: StackValue) {
        self.vec.push(value);
    }

    pub fn pop(&mut self) -> StackValue {
        self.vec
            .pop()
            .expect("[BUG] Stack must have at least one value at pop.")
    }

    pub fn dump(&self, indent: usize) {
        println!("{}[Stack]", " ".repeat(indent));
        for (index, value) in self.vec.iter().rev().enumerate() {
            println!("{}{index:>02}: {value:?}", " ".repeat(indent + 2));
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StackValue {
    RawArray(Vec<Object>),
    Object(Object),
    Named(Rc<String>, Object),
}

impl StackValue {
    pub fn ensure_object(self) -> Object {
        match self {
            StackValue::RawArray(array) => Object::new_array(ArrayObject::new(array)),
            StackValue::Object(obj) => obj,
            x => panic!("[BUG] Expected Object, but got {:?}", x),
        }
    }

    pub fn ensure_named(self) -> (Rc<String>, Object) {
        match self {
            StackValue::Named(name, obj) => (name, obj),
            x => panic!("[BUG] Expected Named, but got {:?}", x),
        }
    }
}

macro_rules! impl_from {
    ($type:ty => $variant:ident) => {
        impl From<$type> for StackValue {
            fn from(value: $type) -> Self {
                Self::$variant(value)
            }
        }
    };
}
impl_from!(Vec<Object> => RawArray);
impl_from!(Object => Object);
impl From<(Rc<String>, Object)> for StackValue {
    fn from(value: (Rc<String>, Object)) -> Self {
        Self::Named(value.0, value.1)
    }
}
impl From<(StringObject, Object)> for StackValue {
    fn from(value: (StringObject, Object)) -> Self {
        Self::Named(Rc::clone(value.0.inner()), value.1)
    }
}
