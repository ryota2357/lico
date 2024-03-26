use super::{sorted_linear_map, sorted_linear_map::SortedLinearMap};
use core::{borrow::Borrow, fmt::Debug, hash::Hash};

type HashMap<K, T> = hashbrown::HashMap<K, T, ahash::RandomState>;

const LINEAR_MAP_SIZE_LIMIT: usize = 16;

#[derive(Clone)]
pub struct LazyHashMap<K: Hash + Ord, V>(Variant<K, V>);

#[derive(Clone, PartialEq)]
enum Variant<K: Hash + Ord, V> {
    Linear(SortedLinearMap<K, V>),
    Hashed(HashMap<K, V>),
}

impl<K: Hash + Ord, V> LazyHashMap<K, V> {
    pub const fn new() -> Self {
        Self(Variant::Linear(SortedLinearMap::new()))
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Variant::Linear(map) => map.len(),
            Variant::Hashed(map) => map.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.0 {
            Variant::Linear(map) => map.is_empty(),
            Variant::Hashed(map) => map.is_empty(),
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

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        match &mut self.0 {
            Variant::Linear(map) => map.remove(key),
            Variant::Hashed(map) => map.remove(key),
        }
    }

    pub fn clear(&mut self) {
        match &mut self.0 {
            Variant::Linear(map) => map.clear(),
            Variant::Hashed(map) => map.clear(),
        }
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Ord + ?Sized,
    {
        match &self.0 {
            Variant::Linear(map) => map.contains_key(key),
            Variant::Hashed(map) => map.contains_key(key),
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

impl<K: Hash + Ord, V> Default for LazyHashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Ord, V, const N: usize> From<[(K, V); N]> for LazyHashMap<K, V> {
    fn from(value: [(K, V); N]) -> Self {
        if N <= LINEAR_MAP_SIZE_LIMIT {
            let map = SortedLinearMap::from(value);
            LazyHashMap(Variant::Linear(map))
        } else {
            let mut map = HashMap::default();
            for (k, v) in value {
                map.insert(k, v);
            }
            Self(Variant::Hashed(map))
        }
    }
}

impl<K: Hash + Ord, V> From<Vec<(K, V)>> for LazyHashMap<K, V> {
    fn from(value: Vec<(K, V)>) -> Self {
        if value.len() <= LINEAR_MAP_SIZE_LIMIT {
            let map = SortedLinearMap::from(value);
            LazyHashMap(Variant::Linear(map))
        } else {
            let mut map = HashMap::default();
            for (k, v) in value {
                map.insert(k, v);
            }
            LazyHashMap(Variant::Hashed(map))
        }
    }
}

impl<K: Hash + Ord, V> IntoIterator for LazyHashMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Variant::Linear(map) => IntoIter::Linear(map.into_iter()),
            Variant::Hashed(map) => IntoIter::Hashed(map.into_iter()),
        }
    }
}

impl<K: Hash + Ord, V: PartialEq> PartialEq for LazyHashMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Variant::Linear(a), Variant::Linear(b)) => a.eq(b),
            (Variant::Hashed(a), Variant::Hashed(b)) => a.eq(b),
            (Variant::Linear(linear), Variant::Hashed(hashed))
            | (Variant::Hashed(hashed), Variant::Linear(linear)) => {
                if linear.len() != hashed.len() {
                    return false;
                }
                for (k, v) in hashed.iter() {
                    if linear.get(k) != Some(v) {
                        return false;
                    }
                }
                true
            }
        }
    }
}

impl<K: Hash + Ord, V: Eq> Eq for LazyHashMap<K, V> {}

impl<K: Hash + Ord + Debug, V: Debug> Debug for LazyHashMap<K, V> {
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

#[derive(Debug, Clone)]
pub enum Iter<'a, K: Hash + Ord, V> {
    Linear(sorted_linear_map::Iter<'a, K, V>),
    Hashed(hashbrown::hash_map::Iter<'a, K, V>),
}
impl_iterator!(for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    map = |(k, v)| (k, v)
});

#[derive(Debug)]
pub enum IterMut<'a, K: Hash + Ord, V> {
    Linear(sorted_linear_map::IterMut<'a, K, V>),
    Hashed(hashbrown::hash_map::IterMut<'a, K, V>),
}
impl_iterator!(for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    map = |(k, v)| (&*k, v)
});

#[derive(Debug)]
pub enum IntoIter<K: Hash + Ord, V> {
    Linear(sorted_linear_map::IntoIter<K, V>),
    Hashed(hashbrown::hash_map::IntoIter<K, V>),
}
impl_iterator!(for IntoIter<K, V> {
    type Item = (K, V);
    map = |(k, v)| (k, v)
});

#[derive(Debug)]
pub enum Drain<'a, K: Hash + Ord, V> {
    Linear(sorted_linear_map::Drain<'a, K, V>),
    Hashed(hashbrown::hash_map::Drain<'a, K, V>),
}
impl_iterator!(for Drain<'a, K, V> {
    type Item = (K, V);
    map = |(k, v)| (k, v)
});
