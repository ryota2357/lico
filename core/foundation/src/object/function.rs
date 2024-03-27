use super::Object;
use crate::il::Executable;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct Function(Rc<Inner>);

#[derive(Debug)]
pub struct Inner {
    executable: Rc<Executable>,
    env: Vec<Rc<RefCell<Object>>>,
}

impl Function {
    pub fn new(executable: Rc<Executable>, env: Vec<Rc<RefCell<Object>>>) -> Self {
        Function(Rc::new(Inner { executable, env }))
    }

    pub fn executable(&self) -> &Executable {
        &self.0.executable
    }

    pub fn env(&self) -> &[Rc<RefCell<Object>>] {
        &self.0.env
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        let self_start_addr = self.0.executable.start_addr();
        let other_start_addr = other.0.executable.start_addr();
        self_start_addr == other_start_addr && self.0.env == other.0.env
    }
}
