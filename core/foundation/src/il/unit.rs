#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalId(usize);

impl LocalId {
    pub const fn new(id: usize) -> Self {
        LocalId(id)
    }

    pub const fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalAddress(u32);

impl LocalAddress {
    pub const fn new(start: u32) -> Self {
        LocalAddress(start)
    }

    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn update_by_offset(&mut self, offset: i32) {
        if offset.is_positive() {
            let offset = offset as u32;
            if cfg!(debug_assertions) {
                self.0 = self.0.checked_add(offset).unwrap_or_else(|| {
                    panic!("Overflow in LocalAddress: {} + {}", self.0, offset);
                });
            } else {
                self.0 = self.0.wrapping_add(offset);
            }
        } else {
            let offset = offset.unsigned_abs();
            if cfg!(debug_assertions) {
                self.0 = self.0.checked_sub(offset).unwrap_or_else(|| {
                    panic!("Overflow in LocalAddress: {} - {}", self.0, offset);
                });
            } else {
                self.0 = self.0.wrapping_sub(offset);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address(usize);

impl Address {
    pub const fn new(id: u32, local: LocalAddress) -> Self {
        Address((id as usize) << 31 | local.as_usize())
    }
}
