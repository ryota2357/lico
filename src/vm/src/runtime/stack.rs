use super::*;

#[derive(Default, Debug, PartialEq)]
pub struct Stack<'a> {
    vec: Vec<StackValue<'a>>,
}

impl<'a> Stack<'a> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    #[inline]
    pub fn push(&mut self, value: StackValue<'a>) {
        self.vec.push(value);
    }

    #[inline]
    pub fn pop(&mut self) -> StackValue<'a> {
        self.vec.pop().expect("[INTERNAL] Stack is empty.")
    }

    pub fn dump(&self, indent: usize) {
        println!("{}[Stack]", " ".repeat(indent));
        for (index, value) in self.vec.iter().rev().enumerate() {
            println!("{}{index:>02}: {value:?}", " ".repeat(indent + 2));
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StackValue<'src> {
    RawFunction(FunctionObject<'src>),
    RawArray(Vec<Object<'src>>),
    RawTable(TableObject<'src>),
    Object(Object<'src>),
    Named(String, Object<'src>),
}

impl<'a> StackValue<'a> {
    pub fn ensure_object(self) -> Object<'a> {
        match self {
            StackValue::RawFunction(func) => Object::new_function(func),
            StackValue::RawArray(array) => Object::new_array(array),
            StackValue::RawTable(table) => Object::new_table(table),
            StackValue::Object(obj) => obj,
            x => panic!("Expected Object, but got {:?}", x),
        }
    }

    pub fn ensure_named(self) -> (String, Object<'a>) {
        match self {
            StackValue::Named(name, obj) => (name, obj),
            x => panic!("Expected Named, but got {:?}", x),
        }
    }
}

macro_rules! impl_from {
    ($type:ty => $variant:ident) => {
        impl<'a> From<$type> for StackValue<'a> {
            fn from(value: $type) -> Self {
                Self::$variant(value)
            }
        }
    };
}
impl_from!(FunctionObject<'a> => RawFunction);
impl_from!(Vec<Object<'a>> => RawArray);
impl_from!(TableObject<'a> => RawTable);
impl_from!(Object<'a> => Object);
impl<'a> From<(String, Object<'a>)> for StackValue<'a> {
    fn from(value: (String, Object<'a>)) -> Self {
        Self::Named(value.0, value.1)
    }
}
