use super::Object;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct Function(Optimize);

#[derive(Clone, Debug)]
enum Optimize {
    #[allow(non_camel_case_types)]
    __dummy,
    Body {
        created_addr: u32,
        inner: Rc<Inner>,
    },
}

#[derive(Debug)]
struct Inner {
    env: Vec<Rc<RefCell<Object>>>,
    // args: Vec<ArgumentKind>,
    // code: Vec<crate::code::Code>,
    source_id: u32,
}

impl Function {
    fn inner(&self) -> &Inner {
        match &self.0 {
            Optimize::Body { inner, .. } => inner,
            Optimize::__dummy => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    pub fn new(env: Vec<Rc<RefCell<Object>>>, created_addr: u32, source_id: u32) -> Self {
        Function(Optimize::Body {
            created_addr,
            inner: Rc::new(Inner {
                env,
                // args: Vec::new(),
                // code: Vec::new(),
                source_id,
            }),
        })
    }

    pub fn created_addr(&self) -> u32 {
        match &self.0 {
            Optimize::Body { created_addr, .. } => *created_addr,
            Optimize::__dummy => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    pub fn env(&self) -> &[Rc<RefCell<Object>>] {
        &self.inner().env
    }

    pub fn source_id(&self) -> u32 {
        self.inner().source_id
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.created_addr() == other.created_addr() && self.env() == other.env()
    }
}

impl Eq for Function {}
