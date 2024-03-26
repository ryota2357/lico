#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalId(usize);

impl LocalId {
    pub fn new(id: usize) -> Self {
        LocalId(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address(u32);

impl Address {
    pub fn new(start: u32) -> Self {
        Address(start)
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn update_by_offset(&mut self, offset: i32) {
        if offset.is_positive() {
            self.0 += offset as u32;
        } else {
            self.0 -= offset.unsigned_abs();
        }
    }
}
