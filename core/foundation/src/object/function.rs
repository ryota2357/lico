use super::*;
use crate::il::Executable;
use core::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Function(Rc<Inner>);

#[derive(Clone, Debug)]
struct Inner {
    exe: Executable,
    env: Box<[Rc<RefCell<Object>>]>,
    param_len: u8,
    start_index: usize,
    call_count: Cell<u32>,
}

impl Function {
    pub fn new<I>(exe: Executable, param_len: u8, start_index: usize, env: I) -> Self
    where
        I: IntoIterator<Item = Rc<RefCell<Object>>>,
    {
        Function(Rc::new(Inner {
            exe,
            env: env.into_iter().collect(),
            param_len,
            start_index,
            call_count: Cell::new(0),
        }))
    }

    pub fn executable(&self) -> &Executable {
        &self.0.exe
    }

    pub fn environment(&self) -> &[Rc<RefCell<Object>>] {
        &self.0.env
    }

    pub fn param_len(&self) -> u8 {
        self.0.param_len
    }

    pub fn start_index(&self) -> usize {
        self.0.start_index
    }

    pub fn call_count(&self) -> u32 {
        self.0.call_count.get()
    }

    pub fn inc_call_count(&mut self) {
        let call_count = self.0.call_count.get().saturating_add(1);
        self.0.call_count.set(call_count);
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Function {}
