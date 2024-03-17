#![allow(dead_code)]

use super::{pms::*, private::*, Function, Object};
use core::{borrow::Borrow, cell::Cell, hash::Hash, marker::PhantomData, ptr::NonNull};
use std::borrow::Cow;

use collections::*;

#[derive(Debug)]
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

    unsafe fn into_children_iter(self) -> impl Iterator<Item = Object> {
        self.data.into_iter().map(|(_, v)| v.into_object())
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

impl<T: TObject> Drop for Table<T> {
    fn drop(&mut self) {
        Table::custom_drop(self);
    }
}

mod collections {
    use core::{borrow::Borrow, hash::Hash, mem};
    type HashMap<K, T> = hashbrown::HashMap<K, T, ahash::RandomState>;

    pub struct LinerMap<K: Ord, V> {
        data: Vec<(K, V)>,
    }

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

        pub fn iter(&self) -> core::slice::Iter<(K, V)> {
            self.data.iter()
        }

        pub fn iter_mut(&mut self) -> core::slice::IterMut<(K, V)> {
            self.data.iter_mut()
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
        type IntoIter = std::vec::IntoIter<(K, V)>;
        fn into_iter(self) -> Self::IntoIter {
            self.data.into_iter()
        }
    }

    pub struct SwitchMap<K: Hash + Ord, V>(SwitchMapVariant<K, V>);
    const LINEAR_MAP_SIZE_LIMIT: usize = 16;
    enum SwitchMapVariant<K: Hash + Ord, V> {
        Linear(LinerMap<K, V>),
        Hashed(HashMap<K, V>),
    }

    impl<K: Hash + Ord, V> SwitchMap<K, V> {
        pub const fn new() -> Self {
            Self(SwitchMapVariant::Linear(LinerMap::new()))
        }

        pub fn insert(&mut self, key: K, value: V) -> Option<V> {
            match &mut self.0 {
                SwitchMapVariant::Linear(map) => {
                    if map.len() < LINEAR_MAP_SIZE_LIMIT {
                        map.insert(key, value)
                    } else {
                        let mut hashmap = HashMap::default();
                        for (k, v) in map.data.drain(..) {
                            hashmap.insert(k, v);
                        }
                        let res = hashmap.insert(key, value);
                        self.0 = SwitchMapVariant::Hashed(hashmap);
                        res
                    }
                }
                SwitchMapVariant::Hashed(map) => map.insert(key, value),
            }
        }

        pub fn get<Q>(&self, key: &Q) -> Option<&V>
        where
            K: Borrow<Q>,
            Q: Hash + Ord + ?Sized,
        {
            match &self.0 {
                SwitchMapVariant::Linear(map) => map.get(key),
                SwitchMapVariant::Hashed(map) => map.get(key),
            }
        }

        pub fn iter(&self) -> SwitchMapIter<K, V> {
            match &self.0 {
                SwitchMapVariant::Linear(map) => SwitchMapIter::Linear(map.iter()),
                SwitchMapVariant::Hashed(map) => SwitchMapIter::Hashed(map.iter()),
            }
        }

        pub fn iter_mut(&mut self) -> SwitchMapIterMut<K, V> {
            match &mut self.0 {
                SwitchMapVariant::Linear(map) => SwitchMapIterMut::Linear(map.iter_mut()),
                SwitchMapVariant::Hashed(map) => SwitchMapIterMut::Hashed(map.iter_mut()),
            }
        }
    }

    impl<K: Hash + Ord, V, const N: usize> From<[(K, V); N]> for SwitchMap<K, V> {
        fn from(value: [(K, V); N]) -> Self {
            if N <= LINEAR_MAP_SIZE_LIMIT {
                let map = LinerMap::from(value);
                SwitchMap(SwitchMapVariant::Linear(map))
            } else {
                let mut map = HashMap::default();
                for (k, v) in value {
                    map.insert(k, v);
                }
                Self(SwitchMapVariant::Hashed(map))
            }
        }
    }
    impl<K: Hash + Ord, V> From<Vec<(K, V)>> for SwitchMap<K, V> {
        fn from(value: Vec<(K, V)>) -> Self {
            if value.len() <= LINEAR_MAP_SIZE_LIMIT {
                let map = LinerMap::from(value);
                SwitchMap(SwitchMapVariant::Linear(map))
            } else {
                let mut map = HashMap::default();
                for (k, v) in value {
                    map.insert(k, v);
                }
                Self(SwitchMapVariant::Hashed(map))
            }
        }
    }

    impl<K: Hash + Ord, V> IntoIterator for SwitchMap<K, V> {
        type Item = (K, V);
        type IntoIter = SwitchMapIntoIter<K, V>;
        fn into_iter(self) -> Self::IntoIter {
            match self.0 {
                SwitchMapVariant::Linear(map) => SwitchMapIntoIter::Linear(map.into_iter()),
                SwitchMapVariant::Hashed(map) => SwitchMapIntoIter::Hashed(map.into_iter()),
            }
        }
    }

    macro_rules! fallback_to_each_variant {
        ($self:ident, $name:ident($($arg:ident),*) $(vec: $(.$vec_chain:ident($vec_expr:expr))*)? $(map: $(.$map_chain:ident($map_expr:expr))*)?) => {
            match $self {
                Self::Linear(iter) => iter.$name($($arg),*) $($(.$vec_chain($vec_expr))*)? ,
                Self::Hashed(iter) => iter.$name($($arg),*) $($(.$map_chain($map_expr))*)?,
            }
        };
    }

    #[derive(Debug)]
    pub enum SwitchMapIter<'a, K: Hash + Ord, V> {
        Linear(core::slice::Iter<'a, (K, V)>),
        Hashed(hashbrown::hash_map::Iter<'a, K, V>),
    }
    impl<'a, K: Hash + Ord, V> Iterator for SwitchMapIter<'a, K, V> {
        type Item = (&'a K, &'a V);
        fn next(&mut self) -> Option<Self::Item> {
            fallback_to_each_variant!(self, next() vec: .map(|(k, v)| (k, v)))
        }
        fn count(self) -> usize {
            fallback_to_each_variant!(self, count())
        }
        fn nth(&mut self, n: usize) -> Option<Self::Item> {
            fallback_to_each_variant!(self, nth(n) vec: .map(|(k, v)| (k, v)))
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            fallback_to_each_variant!(self, size_hint())
        }
    }
    impl<K: Hash + Ord, V> ExactSizeIterator for SwitchMapIter<'_, K, V> {
        fn len(&self) -> usize {
            fallback_to_each_variant!(self, len())
        }
    }
    impl<K: Hash + Ord, V> std::iter::FusedIterator for SwitchMapIter<'_, K, V> {}

    #[derive(Debug)]
    pub enum SwitchMapIterMut<'a, K: Hash + Ord, V> {
        Linear(core::slice::IterMut<'a, (K, V)>),
        Hashed(hashbrown::hash_map::IterMut<'a, K, V>),
    }
    impl<'a, K: Hash + Ord, V> Iterator for SwitchMapIterMut<'a, K, V> {
        type Item = (&'a K, &'a mut V);
        fn next(&mut self) -> Option<Self::Item> {
            fallback_to_each_variant!(self, next() vec: .map(|(k, v)| (&*k, v)))
        }
        fn count(self) -> usize {
            fallback_to_each_variant!(self, count())
        }
        fn last(self) -> Option<Self::Item> {
            fallback_to_each_variant!(self, last() vec: .map(|(k, v)| (&*k, v)))
        }
        fn nth(&mut self, n: usize) -> Option<Self::Item> {
            fallback_to_each_variant!(self, nth(n) vec: .map(|(k, v)| (&*k, v)))
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            fallback_to_each_variant!(self, size_hint())
        }
    }
    impl<K: Hash + Ord, V> ExactSizeIterator for SwitchMapIterMut<'_, K, V> {
        fn len(&self) -> usize {
            fallback_to_each_variant!(self, len())
        }
    }
    impl<K: Hash + Ord, V> std::iter::FusedIterator for SwitchMapIterMut<'_, K, V> {}

    #[derive(Debug)]
    pub enum SwitchMapIntoIter<K: Hash + Ord, V> {
        Linear(std::vec::IntoIter<(K, V)>),
        Hashed(hashbrown::hash_map::IntoIter<K, V>),
    }
    impl<K: Hash + Ord, V> Iterator for SwitchMapIntoIter<K, V> {
        type Item = (K, V);
        fn next(&mut self) -> Option<Self::Item> {
            fallback_to_each_variant!(self, next() vec: .map(|(k, v)| (k, v)))
        }
        fn count(self) -> usize {
            fallback_to_each_variant!(self, count())
        }
        fn last(self) -> Option<Self::Item> {
            fallback_to_each_variant!(self, last() vec: .map(|(k, v)| (k, v)))
        }
        fn nth(&mut self, n: usize) -> Option<Self::Item> {
            fallback_to_each_variant!(self, nth(n) vec: .map(|(k, v)| (k, v)))
        }
        fn size_hint(&self) -> (usize, Option<usize>) {
            fallback_to_each_variant!(self, size_hint())
        }
    }
    impl<K: Hash + Ord, V> ExactSizeIterator for SwitchMapIntoIter<K, V> {
        fn len(&self) -> usize {
            fallback_to_each_variant!(self, len())
        }
    }
    impl<K: Hash + Ord, V> std::iter::FusedIterator for SwitchMapIntoIter<K, V> {}
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
