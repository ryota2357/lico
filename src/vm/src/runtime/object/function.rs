use super::*;

#[derive(Clone, Debug)]
pub struct FunctionObject<'a> {
    pub id: (usize, u8),
    pub env: Vec<(&'a str, Option<Rc<RefCell<Object<'a>>>>)>,
    pub args: Vec<&'a str>,
    pub code: Vec<Code<'a>>,
}

impl PartialEq for FunctionObject<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
