use super::{pms::*, private::TObject, Object};
use core::{cell::Cell, marker::PhantomData, ptr::NonNull};

#[derive(Debug)]
pub struct Array<T: TObject = Object> {
    ptr: NonNull<Inner<T>>,
    phantom: PhantomData<Inner<T>>,
}
impl<T: TObject> PmsObject<Inner<T>> for Array<T> {
    fn ptr(&self) -> NonNull<Inner<T>> {
        self.ptr
    }
    fn ptr_mut(&mut self) -> &mut NonNull<Inner<T>> {
        &mut self.ptr
    }
}

pub struct Inner<T: TObject> {
    data: Vec<T>,
    version: u64,
    ref_count: Cell<usize>,
    color: Cell<Color>,
}
impl<T: TObject> PmsInner for Inner<T> {
    fn ref_count_ref(&self) -> &Cell<usize> {
        &self.ref_count
    }
    fn color_ref(&self) -> &Cell<Color> {
        &self.color
    }

    unsafe fn iter_children_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.data.iter_mut().map(|x| x.as_object_mut())
    }
    unsafe fn into_children_iter(self) -> impl Iterator<Item = Object> {
        self.data.into_iter().map(|x| x.into_object())
    }
}

impl<T: TObject> Array<T> {
    fn inner_mut(&mut self) -> &mut Inner<T> {
        unsafe {
            let inner = PmsObject::inner_mut(self);
            inner.version += 1;
            inner
        }
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
    pub fn set(&mut self, index: usize, value: Object) {
        self.inner_mut().data[index] = value;
    }

    pub fn push(&mut self, value: Object) {
        self.inner_mut().data.push(value);
    }
    pub fn pop(&mut self) -> Option<Object> {
        self.inner_mut().data.pop()
    }
    pub fn insert(&mut self, index: usize, element: Object) {
        self.inner_mut().data.insert(index, element);
    }
    pub fn remove(&mut self, index: usize) -> Object {
        self.inner_mut().data.remove(index)
    }
    pub fn clear(&mut self) {
        todo!("Pms cared clear() call");
        // self.inner_mut().data.clear();
    }

    pub fn contains(&self, value: &Object) -> bool {
        self.inner().data.contains(value)
    }
}

impl Default for Array {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TObject> From<Vec<T>> for Array<T> {
    fn from(array: Vec<T>) -> Self {
        let ptr = Box::leak(Box::new(Inner {
            data: array,
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
        // We can't use pointer for `eq` since the `Object` is `PartialEq` and not `Eq`.
        // Even if pointers are same, `false` must be returned when they contain (point) data for which the reflection rule does not hold.
        self.inner().data.eq(&other.inner().data)
    }
}

unsafe impl<#[may_dangle] T: TObject> Drop for Array<T> {
    fn drop(&mut self) {
        Array::custom_drop(self);
    }
}
