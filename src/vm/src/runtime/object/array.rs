use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayObject<'a>(Vec<Object<'a>>);

impl<'a> ArrayObject<'a> {
    pub fn new(array: Vec<Object<'a>>) -> Self {
        Self(array)
    }
}

impl<'a> Deref for ArrayObject<'a> {
    type Target = Vec<Object<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for ArrayObject<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
