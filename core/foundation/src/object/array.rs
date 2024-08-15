use super::*;
use core::{cell::Cell, fmt, ptr::NonNull};

pub struct Array {
    ptr: NonNull<Inner>,
}

unsafe impl PmsObject<Inner> for Array {
    fn ptr(&self) -> NonNull<Inner> {
        self.ptr
    }

    unsafe fn from_inner(ptr: NonNull<Inner>) -> Self {
        Array { ptr }
    }
}

pub(crate) struct Inner {
    data: Vec<Object>,
    version: u64,
    ref_count: Cell<usize>,
    color: Cell<Color>,
}

unsafe impl PmsInner for Inner {
    fn ref_count_ref(&self) -> &Cell<usize> {
        &self.ref_count
    }

    fn color_ref(&self) -> &Cell<Color> {
        &self.color
    }

    unsafe fn iter_children_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.data.iter_mut()
    }

    unsafe fn drain_children(&mut self) -> impl Iterator<Item = Object> {
        self.data.drain(..)
    }
}

impl Array {
    pub fn new() -> Self {
        Array::from(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.inner().data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn version(&self) -> u64 {
        self.inner().version
    }

    pub fn get(&self, index: usize) -> Option<&Object> {
        self.inner().data.get(index)
    }

    pub fn set<T: Into<Object>>(&mut self, index: usize, value: T) {
        self.inner_data_mut()[index] = value.into();
    }

    pub fn push<T: Into<Object>>(&mut self, value: T) {
        self.inner_data_mut().push(value.into());
    }

    pub fn pop(&mut self) -> Option<Object> {
        self.inner_data_mut().pop()
    }

    pub fn insert<T: Into<Object>>(&mut self, index: usize, element: T) {
        self.inner_data_mut().insert(index, element.into());
    }

    pub fn remove(&mut self, index: usize) -> Object {
        self.inner_data_mut().remove(index)
    }

    pub fn clear(&mut self) {
        self.inner_data_mut().clear();
    }

    pub fn contains(&self, value: &Object) -> bool {
        self.inner().data.contains(value)
    }

    /// # Safety
    /// TODO
    pub unsafe fn iter(&self) -> core::slice::Iter<'_, Object> {
        self.inner().data.iter()
    }

    fn inner_data_mut(&mut self) -> &mut Vec<Object> {
        let inner_mut = unsafe { self.inner_mut() };
        inner_mut.version += 1;
        &mut inner_mut.data
    }
}

impl Default for Array {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> From<[Object; N]> for Array {
    fn from(value: [Object; N]) -> Self {
        let boxed: Box<[Object]> = Box::new(value);
        Array::from(boxed.into_vec())
    }
}

impl From<Vec<Object>> for Array {
    fn from(value: Vec<Object>) -> Self {
        let ptr = Box::leak(Box::new(Inner {
            data: value,
            version: 0,
            ref_count: Cell::new(1),
            color: Cell::new(Color::Black),
        }));
        Array {
            ptr: NonNull::from(ptr),
        }
    }
}

impl Clone for Array {
    fn clone(&self) -> Self {
        self.inner().inc_ref_count();
        Self { ptr: self.ptr }
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        if self.ptr.eq(&other.ptr) {
            let has_nan = self
                .inner()
                .data
                .iter()
                .any(|x| matches!(x, Object::Float(x) if x.is_nan()));
            !has_nan
        } else {
            self.inner().data.eq(&other.inner().data)
        }
    }
}

impl fmt::Debug for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_list();
        for item in self.inner().data.iter() {
            dbg.entry(match item {
                Object::Int(x) => x,
                Object::Float(x) => x,
                Object::String(x) => x,
                Object::Bool(x) => x,
                Object::Nil => &"nil",
                Object::Function(_x) => todo!(),
                Object::Array(_x) => &"Array", // TODO: なんかいい感じにする
                Object::Table(_x) => &"Table",
                Object::RustFunction(x) => x,
            });
        }
        dbg.finish()
    }
}

impl Drop for Array {
    fn drop(&mut self) {
        PmsObject::custom_drop(self);
    }
}
