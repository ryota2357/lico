use super::{private::*, Object};
use core::{cell::Cell, fmt::Debug, marker::PhantomData, ptr::NonNull};

#[allow(private_bounds)]
pub struct Array<T: TObject = Object> {
    ptr: NonNull<Inner<T>>,
    phantom: PhantomData<Inner<T>>,
}
unsafe impl<T: TObject> PmsObject<Inner<T>> for Array<T> {
    fn ptr(&self) -> NonNull<Inner<T>> {
        self.ptr
    }

    unsafe fn from_inner(ptr: NonNull<Inner<T>>) -> Self {
        Array {
            ptr,
            phantom: PhantomData,
        }
    }
}

pub struct Inner<T: TObject> {
    data: Vec<T>,
    version: u64,
    ref_count: Cell<usize>,
    color: Cell<Color>,
}
unsafe impl<T: TObject> PmsInner for Inner<T> {
    fn ref_count_ref(&self) -> &Cell<usize> {
        &self.ref_count
    }

    fn color_ref(&self) -> &Cell<Color> {
        &self.color
    }

    unsafe fn iter_children_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.data.iter_mut().map(|x| x.as_object_mut())
    }

    unsafe fn drain_children(&mut self) -> impl Iterator<Item = Object> {
        self.data.drain(..).map(|x| x.into_object())
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
            phantom: PhantomData,
        }
    }
}

impl<T: TObject> Clone for Array<T> {
    fn clone(&self) -> Self {
        self.inner().inc_ref_count();
        Self {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<T: TObject> PartialEq for Array<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner().data.eq(&other.inner().data)
    }
}

impl<T: TObject> Debug for Array<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_list();
        for item in self.inner().data.iter() {
            dbg.entry(match item.as_object() {
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

unsafe impl<#[may_dangle] T: TObject> Drop for Array<T> {
    fn drop(&mut self) {
        Array::custom_drop(self);
    }
}
