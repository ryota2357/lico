#![allow(dead_code)]

use super::Object;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct Function {
    id: (u32, u32),
    inner: Rc<Inner>,
}

#[derive(Debug)]
struct Inner {
    env: Vec<Rc<RefCell<Object>>>,
    // args: Vec<ArgumentKind>,
    // code: Vec<crate::code::Code>,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Function {}
