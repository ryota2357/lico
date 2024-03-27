#![allow(dead_code)]

use super::Object;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct Function(Rc<Inner>);

#[derive(Debug)]
pub struct Inner {
    env: Vec<Rc<RefCell<Object>>>,
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}
