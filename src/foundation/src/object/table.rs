#![allow(dead_code)]

use super::{pms::*, Function, Object};
use core::{
    // alloc::{Allocator, Layout},
    cell::Cell,
    marker::PhantomData,
    // mem::forget,
    // ptr,
    ptr::NonNull,
};
use std::borrow::Cow;

use collections::*;

#[derive(Debug)]
pub struct Table<T: TObject = Object> {
    ptr: NonNull<Inner<T>>,
    phantom: PhantomData<Inner<T>>,
}
impl<T: TObject> HasPmsInner<Inner<T>> for Table<T> {
    fn ptr(&self) -> NonNull<Inner<T>> {
        self.ptr
    }
    unsafe fn iter_inner_children_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        todo!()
    }
}

struct Inner<T: TObject = Object> {
    data: SwitchMap<Cow<'static, str>, T>,
    methods: LinerMap<Cow<'static, str>, Method>,
    _ref_count: Cell<usize>,
    _color: Cell<Color>,
}
impl<T: TObject> PmsInner for Inner<T> {
    fn ref_count_ref(&self) -> &Cell<usize> {
        &self._ref_count
    }
    fn color_ref(&self) -> &Cell<Color> {
        &self._color
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Method {
    Builtin(fn(Table, &[Object]) -> Result<Object, String>),
    Custom(Function),
    CustomNoSelf(Function),
}

impl<T: TObject> Table<T> {
    pub fn new() -> Self {
        Table::from([])
    }
}

impl<T: TObject> Default for Table<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TObject, const N: usize> From<[(Cow<'static, str>, T); N]> for Table<T> {
    fn from(value: [(Cow<'static, str>, T); N]) -> Self {
        let ptr = Box::leak(Box::new(Inner {
            data: SwitchMap::from(value),
            methods: LinerMap::new(),
            _ref_count: Cell::new(1),
            _color: Cell::new(Color::Black),
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
        todo!()
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
        pub fn iter(&self) -> impl Iterator<Item = &(K, V)> {
            self.data.iter()
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

    pub struct SwitchMap<K: Hash + Ord, V>(SwitchMapVariant<K, V>);
    const LINEAR_MAP_SIZE_LIMIT: usize = 16;
    enum SwitchMapVariant<K: Hash + Ord + Eq, V> {
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
