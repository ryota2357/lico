use super::*;
use crate::collections::*;
use core::{borrow::Borrow, cell::Cell, fmt, hash::Hash, ptr::NonNull};
use std::borrow::Cow;

pub struct Table {
    ptr: NonNull<Inner>,
}

unsafe impl PmsObject<Inner> for Table {
    fn ptr(&self) -> NonNull<Inner> {
        self.ptr
    }

    unsafe fn from_inner(ptr: NonNull<Inner>) -> Self {
        Table { ptr }
    }
}

pub struct Inner {
    map: LazyHashMap<Cow<'static, str>, Object>,
    methods: SortedLinearMap<Cow<'static, str>, TableMethod>,
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
        self.map.iter_mut().map(|(_, v)| v)
    }

    unsafe fn drain_children(&mut self) -> impl Iterator<Item = Object> {
        self.map.drain().map(|(_, v)| v)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TableMethod {
    Native(RustFunction),
    Custom(Function),
}

impl Table {
    pub fn new() -> Self {
        let map = LazyHashMap::new();
        Table::with_map(map)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let map = LazyHashMap::with_capacity(capacity);
        Table::with_map(map)
    }

    pub fn len(&self) -> usize {
        self.inner().map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner().map.is_empty()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Object>
    where
        Cow<'static, str>: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        self.inner().map.get(key)
    }

    pub fn insert<T: Into<Object>>(&mut self, key: Cow<'static, str>, value: T) -> Option<Object> {
        unsafe { self.inner_mut().map.insert(key, value.into()) }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<Object>
    where
        Cow<'static, str>: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        unsafe { self.inner_mut().map.remove(key) }
    }

    pub fn clear(&mut self) {
        unsafe {
            self.inner_mut().map.clear();
        }
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Cow<'static, str>: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        self.inner().map.contains_key(key)
    }

    /// # Safety
    /// TODO
    pub unsafe fn iter(&self) -> lazy_hash_map::Iter<Cow<'static, str>, Object> {
        self.inner().map.iter()
    }

    pub fn get_method<Q>(&self, key: &Q) -> Option<&TableMethod>
    where
        Cow<'static, str>: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        self.inner().methods.get(key)
    }

    pub fn set_method<T: Into<TableMethod>>(&mut self, key: Cow<'static, str>, value: T) {
        unsafe { self.inner_mut().methods.insert(key, value.into()) };
    }

    fn with_map(map: LazyHashMap<Cow<'static, str>, Object>) -> Self {
        let ptr = Box::leak(Box::new(Inner {
            map,
            methods: SortedLinearMap::new(),
            ref_count: Cell::new(1),
            color: Cell::new(Color::Black),
        }));
        Table {
            ptr: NonNull::from(ptr),
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> From<[(Cow<'static, str>, Object); N]> for Table {
    fn from(value: [(Cow<'static, str>, Object); N]) -> Self {
        let data = LazyHashMap::from(value);
        Table::with_map(data)
    }
}

impl PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        if self.ptr.eq(&other.ptr) {
            let has_nan = self
                .inner()
                .map
                .iter()
                .any(|(_, x)| matches!(x, Object::Float(x) if x.is_nan()));
            !has_nan
        } else {
            let inner = self.inner();
            let other = other.inner();
            inner.map.eq(&other.map) && inner.methods.eq(&other.methods)
        }
    }
}

impl Clone for Table {
    fn clone(&self) -> Self {
        self.inner().inc_ref_count();
        Self { ptr: self.ptr }
    }
}

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_map();
        for (key, value) in self.inner().map.iter() {
            dbg.key(key).value(match value {
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

impl Drop for Table {
    fn drop(&mut self) {
        PmsObject::custom_drop(self);
    }
}
