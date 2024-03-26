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
