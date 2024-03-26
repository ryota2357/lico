mod code;
pub use code::Code;

mod unit;
pub use unit::*;

#[derive(Debug)]
pub struct Executable {
    _id: u32, // id というよりも info みたいなのが欲しい
    code: Vec<Code>,
}

impl Executable {
    pub fn new(id: u32, code: Vec<Code>) -> Self {
        if code.len() > u32::MAX as usize {
            panic!("code size is too large");
        }
        Executable { code, _id: id }
    }

    pub fn fetch(&self, addr: Address) -> &Code {
        &self.code.get(addr.as_usize()).expect("invalid address")
    }
}
