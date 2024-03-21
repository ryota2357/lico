use core::{borrow::Borrow, fmt::Debug, mem};

#[derive(Clone, PartialEq)]
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
