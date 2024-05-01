use super::{private::*, Function, Object};
use crate::collections::*;
use core::{borrow::Borrow, cell::Cell, fmt::Debug, hash::Hash, marker::PhantomData, ptr::NonNull};
use std::borrow::Cow;

#[allow(private_bounds)]
pub struct Table<T: TObject = Object> {
    ptr: NonNull<Inner<T>>,
    phantom: PhantomData<Inner<T>>,
}
unsafe impl<T: TObject> PmsObject<Inner<T>> for Table<T> {
    fn ptr(&self) -> NonNull<Inner<T>> {
        self.ptr
    }

    unsafe fn from_inner(ptr: NonNull<Inner<T>>) -> Self {
        Table {
            ptr,
            phantom: PhantomData,
        }
    }
}

pub struct Inner<T: TObject> {
    data: LazyHashMap<Cow<'static, str>, T>,
    methods: SortedLinearMap<Cow<'static, str>, TableMethod>,
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
        self.data.iter_mut().map(|(_, v)| v.as_object_mut())
    }

    unsafe fn drain_children(&mut self) -> impl Iterator<Item = Object> {
        self.data.drain().map(|(_, v)| v.into_object())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TableMethod {
    Builtin(fn(Table, &[Object]) -> Result<Object, String>),
    Custom(Function),
    CustomNoSelf(Function),
}

impl Table {
    pub fn new() -> Self {
        Table::from([])
    }

    pub fn len(&self) -> usize {
        self.inner().data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner().data.is_empty()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Object>
    where
        Cow<'static, str>: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        self.inner().data.get(key)
    }

    pub fn insert<T: Into<Object>>(&mut self, key: Cow<'static, str>, value: T) -> Option<Object> {
        unsafe { self.inner_mut().data.insert(key, value.into()) }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<Object>
    where
        Cow<'static, str>: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        unsafe { self.inner_mut().data.remove(key) }
    }

    pub fn clear(&mut self) {
        unsafe {
            self.inner_mut().data.clear();
        }
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Cow<'static, str>: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        self.inner().data.contains_key(key)
    }

    /// # Safety
    /// TODO
    pub unsafe fn iter(&self) -> lazy_hash_map::Iter<Cow<'static, str>, Object> {
        self.inner().data.iter()
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> From<[(Cow<'static, str>, Object); N]> for Table {
    fn from(value: [(Cow<'static, str>, Object); N]) -> Self {
        let ptr = Box::leak(Box::new(Inner {
            data: LazyHashMap::from(value),
            methods: SortedLinearMap::new(),
            ref_count: Cell::new(1),
            color: Cell::new(Color::Black),
        }));
        Table {
            ptr: NonNull::from(ptr),
            phantom: PhantomData,
        }
    }
}

impl<T: TObject> PartialEq for Table<T> {
    fn eq(&self, other: &Self) -> bool {
        let inner = self.inner();
        let other = other.inner();
        inner.data.eq(&other.data) && inner.methods.eq(&other.methods)
    }
}

impl<T: TObject> Clone for Table<T> {
    fn clone(&self) -> Self {
        self.inner().inc_ref_count();
        Self {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<T: TObject> Debug for Table<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_map();
        for (key, value) in self.inner().data.iter() {
            dbg.key(key).value(match value.as_object() {
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

impl<T: TObject> Drop for Table<T> {
    fn drop(&mut self) {
        Table::custom_drop(self);
    }
}
