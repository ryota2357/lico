#![allow(dead_code)]

use super::{pms::*, private::*, Function, Object};
use core::{borrow::Borrow, cell::Cell, fmt::Debug, hash::Hash, marker::PhantomData, ptr::NonNull};
use std::borrow::Cow;

use collections::*;

pub struct Table<T: TObject = Object> {
    ptr: NonNull<Inner<T>>,
    phantom: PhantomData<Inner<T>>,
}
impl<T: TObject> PmsObject<Inner<T>> for Table<T> {
    fn ptr(&self) -> NonNull<Inner<T>> {
        self.ptr
    }
    fn ptr_mut(&mut self) -> &mut NonNull<Inner<T>> {
        &mut self.ptr
    }
}

pub struct Inner<T: TObject> {
    data: SwitchMap<Cow<'static, str>, T>,
    methods: LinerMap<Cow<'static, str>, TableMethod>,
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

impl<T: TObject> Table<T> {
    fn inner_mut(&mut self) -> &mut Inner<T> {
        unsafe { PmsObject::inner_mut(self) }
    }
}

impl Table {
    pub fn new() -> Self {
        Table::from([])
    }

    pub fn insert(&mut self, key: Cow<'static, str>, value: Object) -> Option<Object> {
        self.inner_mut().data.insert(key, value)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&Object>
    where
        Cow<'static, str>: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        self.inner().data.get(key)
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TObject, const N: usize> From<[(Cow<'static, str>, T); N]> for Table<T> {
    fn from(value: [(Cow<'static, str>, T); N]) -> Self {
        let ptr = Box::leak(Box::new(Inner {
            data: SwitchMap::from(value),
            methods: LinerMap::new(),
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
    fn eq(&self, _other: &Self) -> bool {
        todo!()
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

mod collections {
    pub use liner_map::LinerMap;
    pub use switch_map::SwitchMap;

    pub mod liner_map {
        use core::{borrow::Borrow, fmt::Debug, mem};

        pub struct LinerMap<K: Ord, V> {
            data: Vec<(K, V)>,
        }

        pub type Iter<'a, K, V> = core::slice::Iter<'a, (K, V)>;
        pub type IterMut<'a, K, V> = core::slice::IterMut<'a, (K, V)>;
        pub type IntoIter<K, V> = std::vec::IntoIter<(K, V)>;
        pub type Drain<'a, K, V> = std::vec::Drain<'a, (K, V)>;

        impl<K: Ord, V> LinerMap<K, V> {
            pub const fn new() -> Self {
                Self { data: Vec::new() }
            }

            pub fn insert(&mut self, key: K, value: V) -> Option<V> {
                let index = self.data.binary_search_by_key(&&key, |(k, _)| k);
                match index {
                    Ok(index) => Some(mem::replace(
                        unsafe { &mut self.data.get_unchecked_mut(index).1 },
                        value,
                    )),
                    Err(index) => {
                        self.data.insert(index, (key, value));
                        None
                    }
                }
            }

            pub fn get<Q>(&self, key: &Q) -> Option<&V>
            where
                K: Borrow<Q>,
                Q: Ord + ?Sized,
            {
                let index = self.data.binary_search_by_key(&key, |(k, _)| k.borrow());
                match index {
                    Ok(index) => Some(&self.data[index].1),
                    Err(_) => None,
                }
            }

            pub fn len(&self) -> usize {
                self.data.len()
            }

            pub fn is_empty(&self) -> bool {
                self.data.is_empty()
            }

            pub fn clear(&mut self) {
                self.data.clear()
            }

            pub fn iter(&self) -> Iter<K, V> {
                self.data.iter()
            }

            pub fn iter_mut(&mut self) -> IterMut<K, V> {
                self.data.iter_mut()
            }

            pub fn drain(&mut self) -> Drain<K, V> {
                self.data.drain(..)
            }
        }

        impl<K: Ord, V, const N: usize> From<[(K, V); N]> for LinerMap<K, V> {
            fn from(value: [(K, V); N]) -> Self {
                let vec = value.into_iter().collect::<Vec<_>>();
                LinerMap::from(vec)
            }
        }

        impl<K: Ord, V> From<Vec<(K, V)>> for LinerMap<K, V> {
            fn from(value: Vec<(K, V)>) -> Self {
                let mut vec = value;
                vec.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
                vec.dedup_by(|(a, _), (b, _)| a == b);
                LinerMap { data: vec }
            }
        }

        impl<K: Ord, V> IntoIterator for LinerMap<K, V> {
            type Item = (K, V);
            type IntoIter = IntoIter<K, V>;
            fn into_iter(self) -> Self::IntoIter {
                self.data.into_iter()
            }
        }

        impl<K: Ord + Debug, V: Debug> Debug for LinerMap<K, V> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_map()
                    .entries(self.iter().map(|&(ref k, ref v)| (k, v)))
                    .finish()
            }
        }
    }

    pub mod switch_map {
        use super::{liner_map, liner_map::LinerMap};
        use core::{borrow::Borrow, fmt::Debug, hash::Hash};

        type HashMap<K, T> = hashbrown::HashMap<K, T, ahash::RandomState>;

        const LINEAR_MAP_SIZE_LIMIT: usize = 16;

        pub struct SwitchMap<K: Hash + Ord, V>(Variant<K, V>);
        enum Variant<K: Hash + Ord, V> {
            Linear(LinerMap<K, V>),
            Hashed(HashMap<K, V>),
        }

        impl<K: Hash + Ord, V> SwitchMap<K, V> {
            pub const fn new() -> Self {
                Self(Variant::Linear(LinerMap::new()))
            }

            pub fn insert(&mut self, key: K, value: V) -> Option<V> {
                match &mut self.0 {
                    Variant::Linear(map) => {
                        if map.len() < LINEAR_MAP_SIZE_LIMIT {
                            map.insert(key, value)
                        } else {
                            let mut hashmap = HashMap::default();
                            for (k, v) in map.drain() {
                                hashmap.insert(k, v);
                            }
                            let res = hashmap.insert(key, value);
                            self.0 = Variant::Hashed(hashmap);
                            res
                        }
                    }
                    Variant::Hashed(map) => map.insert(key, value),
                }
            }

            pub fn get<Q>(&self, key: &Q) -> Option<&V>
            where
                K: Borrow<Q>,
                Q: Hash + Ord + ?Sized,
            {
                match &self.0 {
                    Variant::Linear(map) => map.get(key),
                    Variant::Hashed(map) => map.get(key),
                }
            }

            pub fn clear(&mut self) {
                match &mut self.0 {
                    Variant::Linear(map) => map.clear(),
                    Variant::Hashed(map) => map.clear(),
                }
            }

            pub fn iter(&self) -> Iter<K, V> {
                match &self.0 {
                    Variant::Linear(map) => Iter::Linear(map.iter()),
                    Variant::Hashed(map) => Iter::Hashed(map.iter()),
                }
            }

            pub fn iter_mut(&mut self) -> IterMut<K, V> {
                match &mut self.0 {
                    Variant::Linear(map) => IterMut::Linear(map.iter_mut()),
                    Variant::Hashed(map) => IterMut::Hashed(map.iter_mut()),
                }
            }

            pub fn drain(&mut self) -> Drain<'_, K, V> {
                match &mut self.0 {
                    Variant::Linear(map) => Drain::Linear(map.drain()),
                    Variant::Hashed(map) => Drain::Hashed(map.drain()),
                }
            }
        }

        impl<K: Hash + Ord, V, const N: usize> From<[(K, V); N]> for SwitchMap<K, V> {
            fn from(value: [(K, V); N]) -> Self {
                if N <= LINEAR_MAP_SIZE_LIMIT {
                    let map = LinerMap::from(value);
                    SwitchMap(Variant::Linear(map))
                } else {
                    let mut map = HashMap::default();
                    for (k, v) in value {
                        map.insert(k, v);
                    }
                    Self(Variant::Hashed(map))
                }
            }
        }

        impl<K: Hash + Ord, V> From<Vec<(K, V)>> for SwitchMap<K, V> {
            fn from(value: Vec<(K, V)>) -> Self {
                if value.len() <= LINEAR_MAP_SIZE_LIMIT {
                    let map = LinerMap::from(value);
                    SwitchMap(Variant::Linear(map))
                } else {
                    let mut map = HashMap::default();
                    for (k, v) in value {
                        map.insert(k, v);
                    }
                    SwitchMap(Variant::Hashed(map))
                }
            }
        }

        impl<K: Hash + Ord, V> IntoIterator for SwitchMap<K, V> {
            type Item = (K, V);
            type IntoIter = IntoIter<K, V>;
            fn into_iter(self) -> Self::IntoIter {
                match self.0 {
                    Variant::Linear(map) => IntoIter::Linear(map.into_iter()),
                    Variant::Hashed(map) => IntoIter::Hashed(map.into_iter()),
                }
            }
        }

        impl<K: Hash + Ord + Debug, V: Debug> Debug for SwitchMap<K, V> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_map().entries(self.iter()).finish()
            }
        }

        macro_rules! impl_iterator {
            ( for $ty:ty { type Item = ($ty_k:ty, $ty_v:ty); map = $map_fn:expr } ) => {
                impl<'a, K: Hash + Ord, V> Iterator for $ty {
                    type Item = ($ty_k, $ty_v);
                    fn next(&mut self) -> Option<Self::Item> {
                        match self {
                            Self::Linear(iter) => iter.next().map($map_fn),
                            Self::Hashed(iter) => iter.next(),
                        }
                    }
                    fn count(self) -> usize {
                        match self {
                            Self::Linear(iter) => iter.count(),
                            Self::Hashed(iter) => iter.count(),
                        }
                    }
                    fn nth(&mut self, n: usize) -> Option<Self::Item> {
                        match self {
                            Self::Linear(iter) => iter.nth(n).map($map_fn),
                            Self::Hashed(iter) => iter.nth(n),
                        }
                    }
                    fn size_hint(&self) -> (usize, Option<usize>) {
                        match self {
                            Self::Linear(iter) => iter.size_hint(),
                            Self::Hashed(iter) => iter.size_hint(),
                        }
                    }
                }
                impl<'a, K: Hash + Ord, V> ExactSizeIterator for $ty {
                    fn len(&self) -> usize {
                        match self {
                            Self::Linear(iter) => iter.len(),
                            Self::Hashed(iter) => iter.len(),
                        }
                    }
                }
                impl<'a, K: Hash + Ord, V> std::iter::FusedIterator for $ty {}
            };
        }

        #[derive(Debug)]
        pub enum Iter<'a, K: Hash + Ord, V> {
            Linear(liner_map::Iter<'a, K, V>),
            Hashed(hashbrown::hash_map::Iter<'a, K, V>),
        }
        impl_iterator!(for Iter<'a, K, V> {
            type Item = (&'a K, &'a V);
            map = |(k, v)| (k, v)
        });

        #[derive(Debug)]
        pub enum IterMut<'a, K: Hash + Ord, V> {
            Linear(liner_map::IterMut<'a, K, V>),
            Hashed(hashbrown::hash_map::IterMut<'a, K, V>),
        }
        impl_iterator!(for IterMut<'a, K, V> {
            type Item = (&'a K, &'a mut V);
            map = |(k, v)| (&*k, v)
        });

        #[derive(Debug)]
        pub enum IntoIter<K: Hash + Ord, V> {
            Linear(liner_map::IntoIter<K, V>),
            Hashed(hashbrown::hash_map::IntoIter<K, V>),
        }
        impl_iterator!(for IntoIter<K, V> {
            type Item = (K, V);
            map = |(k, v)| (k, v)
        });

        #[derive(Debug)]
        pub enum Drain<'a, K: Hash + Ord, V> {
            Linear(liner_map::Drain<'a, K, V>),
            Hashed(hashbrown::hash_map::Drain<'a, K, V>),
        }
        impl_iterator!(for Drain<'a, K, V> {
            type Item = (K, V);
            map = |(k, v)| (k, v)
        });
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn variant_size() {
//         macro_rules! check {
//             ($($variant:ident : $type:ty { $init:expr } = $size:literal),* $(,)?) => {
//                 $(
//                     let x = Variant::$variant($init);
//                     match x {
//                         Variant::$variant(x) => { let _: $type = x; },
//                         _ => unreachable!(),
//                     }
//                     let size = ::std::mem::size_of::<$type>();
//                     assert_eq!(size, $size);
//                 )*
//             };
//         }
//         check! {
//             Vec: Vec<Object> { Vec::new() } = 24,
//             Map: Map<Cow<'static, str>, Object> { Map::default() } = 64
//         }
//         assert_eq!(std::mem::size_of::<Variant>(), 64);
//     }
// }
