use super::*;

#[derive(Clone, Debug)]
pub struct FunctionObject {
    pub id: (usize, u8),
    pub env: Vec<Rc<RefCell<Object>>>,
    pub args: Vec<ArgumentKind>,
    pub code: Vec<Code>,
}

impl PartialEq for FunctionObject {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
