mod icode;
pub use icode::ICode;

mod unit;
pub use unit::*;

#[derive(Debug)]
pub struct Executable {
    start_addr: Address,
    code: Vec<ICode>,
}

impl Executable {
    pub fn new(start_addr: Address, code: Vec<ICode>) -> Self {
        if code.len() > u32::MAX as usize {
            panic!("code size is too large");
        }
        Executable { start_addr, code }
    }

    pub fn start_addr(&self) -> Address {
        self.start_addr
    }

    pub fn fetch(&self, addr: LocalAddress) -> &ICode {
        self.code.get(addr.as_usize()).expect("invalid address")
    }
}
