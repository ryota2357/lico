#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalId(usize);

impl LocalId {
    pub const fn new(id: usize) -> Self {
        LocalId(id)
    }

    pub const fn as_usize(&self) -> usize {
        self.0
    }
}
