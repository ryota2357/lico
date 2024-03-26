use super::Object;
use crate::il::{Address, Executable};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct Function(Optimize);

#[derive(Clone, Debug)]
enum Optimize {
    #[allow(non_camel_case_types)]
    __dummy,
    Body {
        created_addr: Address,
        inner: Rc<Inner>,
    },
}

#[derive(Debug)]
struct Inner {
    executable: Executable,
    env: Vec<Rc<RefCell<Object>>>,
    // args: Vec<ArgumentKind>,
}

impl Function {
    fn inner(&self) -> &Inner {
        match &self.0 {
            Optimize::Body { inner, .. } => inner,
            Optimize::__dummy => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    // fn inner_mut(&mut self) -> &mut Inner {
    //     match &mut self.0 {
    //         Optimize::Body { inner, .. } => inner,
    //         Optimize::__dummy => unsafe { core::hint::unreachable_unchecked() },
    //     }
    // }

    pub fn new(
        created_addr: Address,
        executable: Executable,
        env: Vec<Rc<RefCell<Object>>>,
    ) -> Self {
        Function(Optimize::Body {
            created_addr,
            inner: Rc::new(Inner { env, executable }),
        })
    }

    pub fn created_addr(&self) -> Address {
        match &self.0 {
            Optimize::Body { created_addr, .. } => *created_addr,
            Optimize::__dummy => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    pub fn env(&self) -> &[Rc<RefCell<Object>>] {
        &self.inner().env
    }

    pub fn executable(&self) -> &Executable {
        &self.inner().executable
    }

    // pub fn executable_mut(&mut self) -> &mut Executable {
    //     &mut self.inner_mut().executable
    // }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        // TODO: executable が同じかどうかも見る
        self.created_addr() == other.created_addr()
    }
}

impl Eq for Function {}
