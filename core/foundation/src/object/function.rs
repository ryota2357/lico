use super::Object;
use crate::il::Executable;
use core::ptr::addr_eq;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct Function(Box<Inner>);

#[derive(Clone, Debug)]
struct Inner {
    exe: Executable,
    start_index: usize,
    env: Box<[Rc<RefCell<Object>>]>,
    call_count: u32,
}

impl Function {
    pub fn new<I>(exe: Executable, start: usize, env: I) -> Self
    where
        I: IntoIterator<Item = Rc<RefCell<Object>>>,
    {
        Function(Box::new(Inner {
            exe,
            env: env.into_iter().collect(),
            start_index: start,
            call_count: 0,
        }))
    }

    pub fn start_index(&self) -> usize {
        self.0.start_index
    }

    pub fn executable(&self) -> &Executable {
        &self.0.exe
    }

    pub fn environment(&self) -> &[Rc<RefCell<Object>>] {
        &self.0.env
    }

    pub fn call_count(&self) -> u32 {
        self.0.call_count
    }

    pub fn inc_call_count(&mut self) {
        self.0.call_count += 1;
    }
}

// TODO: write the reason why we only compare the pointer instead of the content (Inner)
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        let self_ptr = &*self.0 as *const Inner;
        let other_ptr = &*other.0 as *const Inner;
        addr_eq(self_ptr, other_ptr)
    }
}
