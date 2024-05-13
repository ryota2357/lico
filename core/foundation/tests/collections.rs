use foundation::collections::*;

#[test]
fn arena_construct() {
    let mut arena = Arena::<String>::new();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
    arena.alloc("foo".to_string());
    assert_eq!(arena.len(), 1);
    assert!(!arena.is_empty());
}

#[test]
fn arena_alloc_get_iter() {
    #[derive(PartialEq)]
    struct T(u32);
    let mut arena = Arena::new();
    let idx1 = arena.alloc(T(42));
    assert_eq!(format!("{:?}", idx1), "Index::<T>(1)");
    let idx2 = arena.alloc(T(17));
    assert_eq!(format!("{:?}", idx2), "Index::<T>(2)");
    let idx3 = arena.alloc(T(17));
    assert_eq!(format!("{:?}", idx3), "Index::<T>(3)");
    assert_eq!(arena.get(idx1).0, 42);
    assert_eq!(arena.get(idx2).0, 17);
    assert_eq!(arena.get(idx3).0, 17);
    arena.get_mut(idx2).0 = 18;
    let mut iter = arena.iter();
    assert!(iter.next() == Some((idx1, &T(42))));
    assert!(iter.next() == Some((idx2, &T(18))));
    assert!(iter.next() == Some((idx3, &T(17))));
    assert!(iter.next().is_none());
}

#[test]
fn arena_alloc_many() {
    let mut arena = Arena::new();
    let slice = arena.alloc_many([10, 20, 30]);
    assert_eq!(format!("{:?}", slice), "Slice::<i32>(1..4)");
    assert_eq!(arena.get_slice(slice), &[10, 20, 30]);
    arena.get_slice_mut(slice)[1] = 21;
    let mut iter = arena.iter();
    assert_eq!(iter.next(), Some((unsafe { arena::Index::new(1) }, &10)));
    assert_eq!(iter.next(), Some((unsafe { arena::Index::new(2) }, &21)));
    assert_eq!(iter.next(), Some((unsafe { arena::Index::new(3) }, &30)));
    assert_eq!(iter.next(), None);
}

#[test]
fn sorted_linear_map_construct() {
    const _: SortedLinearMap<i32, i32> = SortedLinearMap::new();
    let map = SortedLinearMap::<&str, &str>::new();
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
    assert_eq!(map.get("one"), None);
    assert_eq!(map, SortedLinearMap::default());
}

#[test]
fn sorted_linear_map_insert() {
    let mut map = SortedLinearMap::from([("zero", 0), ("two", 2), ("three", 3)]);
    map.insert("one", 1);
    assert_eq!(map.len(), 4);
    assert!(!map.is_empty());
    assert_eq!(map.get("zero"), Some(&0));
    assert_eq!(map.get("one"), Some(&1));
    assert_eq!(map.get("four"), None);
}

#[test]
fn sorted_linear_map_remove() {
    let mut map = SortedLinearMap::from([("zero", 0), ("one", 1), ("two", 2), ("three", 3)]);
    assert_eq!(map.remove("two"), Some(2));
    assert_eq!(map.remove("four"), None);
    assert_eq!(map.len(), 3);
    assert!(!map.is_empty());
    assert_eq!(
        map,
        SortedLinearMap::from([("zero", 0), ("one", 1), ("three", 3)])
    );
}

#[test]
fn sorted_linear_map_clear() {
    let mut map = SortedLinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    assert!(!map.is_empty());
    map.clear();
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
    assert_eq!(map, SortedLinearMap::default());
}

#[test]
fn sorted_linear_map_contains_key() {
    let map = SortedLinearMap::from([("aaa", 0), ("aad", 1), ("aac", 2)]);
    assert!(map.contains_key("aac"));
    assert!(!map.contains_key("aab"));
}

#[test]
fn sorted_linear_map_iter() {
    let map = SortedLinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut iter = map.iter();
    assert_eq!(iter.next(), Some(&("one", 1)));
    assert_eq!(iter.next(), Some(&("two", 2)));
    assert_eq!(iter.next(), Some(&("zero", 0)));
    assert_eq!(iter.next(), None);
}

#[test]
fn sorted_linear_map_iter_mut() {
    let mut map = SortedLinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut iter = map.iter_mut();
    assert_eq!(iter.next(), Some(&mut ("one", 1)));
    assert_eq!(iter.next(), Some(&mut ("two", 2)));
    assert_eq!(iter.next(), Some(&mut ("zero", 0)));
    assert_eq!(iter.next(), None);
}

#[test]
fn sorted_linear_map_drain() {
    let mut map = SortedLinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut iter = map.drain();
    assert_eq!(iter.next(), Some(("one", 1)));
    assert_eq!(iter.next(), Some(("two", 2)));
    assert_eq!(iter.next(), Some(("zero", 0)));
    assert_eq!(iter.next(), None);
}

#[test]
fn sorted_linear_map_into_iter() {
    let map = SortedLinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut iter = map.into_iter();
    assert_eq!(iter.next(), Some(("one", 1)));
    assert_eq!(iter.next(), Some(("two", 2)));
    assert_eq!(iter.next(), Some(("zero", 0)));
    assert_eq!(iter.next(), None);
}

#[test]
fn sorted_linear_map_no_sync_with_clone() {
    let map1 = SortedLinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut map2 = map1.clone();
    map2.insert("three", 3);
    assert_eq!(map1.len(), 3);
    assert_eq!(map1.get("three"), None);
    assert_ne!(map1, map2);
}

#[test]
fn sorted_linear_map_equal_for_nan() {
    let map1 = SortedLinearMap::from([("nan", f64::NAN)]);
    let map2 = SortedLinearMap::from([("nan", f64::NAN)]);
    assert_ne!(map1, map2);
}

#[test]
fn sorted_linear_map_debug() {
    let map = SortedLinearMap::from([("key", 1), ("foo", 3)]);
    assert_eq!(format!("{:?}", map), r#"{"foo": 3, "key": 1}"#);
}

#[test]
fn lazy_hash_map_construct() {
    const _: LazyHashMap<i32, i32> = LazyHashMap::new();
    let map = LazyHashMap::<&str, &str>::new();
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
    assert_eq!(map.get("one"), None);
    assert_eq!(map, LazyHashMap::default());
}

#[test]
fn lazy_hash_map_insert() {
    let mut map = LazyHashMap::new();
    map.insert("foo".to_string(), -1);
    map.insert("bar".to_string(), -2);
    assert_eq!(map.len(), 2);
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert_eq!(map.len(), 102);
    assert!(!map.is_empty());
    assert_eq!(map.get("bar"), Some(&-2));
    assert_eq!(map.get("fo"), None);
    assert_eq!(map.get("key99"), Some(&99));
}

#[test]
fn lazy_hash_map_remove() {
    let mut map = LazyHashMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    assert_eq!(map.remove("foo"), Some(-1));
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert_eq!(map.remove("bar"), Some(-2));
    assert_eq!(map.remove("bar"), None);
    assert_eq!(map.len(), 100);
}

#[test]
fn lazy_hash_map_clear() {
    let mut map = LazyHashMap::new();
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert!(!map.is_empty());
    map.clear();
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
    assert_eq!(map, LazyHashMap::default());
}

#[test]
fn lazy_hash_map_contains_key() {
    let mut map = LazyHashMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    assert!(map.contains_key("foo"));
    assert!(!map.contains_key("baz"));
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert!(map.contains_key("foo"));
    assert!(!map.contains_key("baz"));
    assert!(map.contains_key("key99"));
    assert!(!map.contains_key("key100"));
}

#[test]
fn lazy_hash_map_iter() {
    let mut map = LazyHashMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert_eq!(map.len(), 102);
    let mut iter = map.iter();
    let mut nexts = std::collections::HashMap::new();
    for _ in 0..102 {
        let (key, value) = iter.next().unwrap();
        nexts.insert(key.clone(), *value);
    }
    assert_eq!(nexts.get("bar"), Some(&-2));
    assert_eq!(nexts.get("baz"), None);
    assert_eq!(nexts.get("key57"), Some(&57));
    assert_eq!(iter.next(), None);
}

#[test]
fn lazy_hash_map_iter_mut() {
    let mut map = LazyHashMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    let mut iter = map.iter_mut();
    let mut nexts = std::collections::HashMap::new();
    for _ in 0..102 {
        let (key, value) = iter.next().unwrap();
        nexts.insert(key.clone(), *value);
    }
    assert_eq!(nexts.get("bar"), Some(&-2));
    assert_eq!(nexts.get("baz"), None);
    assert_eq!(nexts.get("key57"), Some(&57));
    assert_eq!(iter.next(), None);
}

#[test]
fn lazy_hash_map_drain() {
    let mut map = LazyHashMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    let mut iter = map.drain();
    let mut nexts = std::collections::HashMap::new();
    for _ in 0..102 {
        let (key, value) = iter.next().unwrap();
        nexts.insert(key.clone(), value);
    }
    assert_eq!(nexts.get("bar"), Some(&-2));
    assert_eq!(nexts.get("baz"), None);
    assert_eq!(nexts.get("key57"), Some(&57));
    assert_eq!(iter.next(), None);
}

#[test]
fn lazy_hash_map_into_iter() {
    let mut map = LazyHashMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    let mut iter = map.into_iter();
    let mut nexts = std::collections::HashMap::new();
    for _ in 0..102 {
        let (key, value) = iter.next().unwrap();
        nexts.insert(key, value);
    }
    assert_eq!(nexts.get("bar"), Some(&-2));
    assert_eq!(nexts.get("baz"), None);
    assert_eq!(nexts.get("key57"), Some(&57));
    assert_eq!(iter.next(), None);
}

#[test]
fn lazy_hash_map_no_sync_with_clone() {
    let map1 = LazyHashMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    let mut map2 = map1.clone();
    map2.insert("baz".to_string(), -3);
    assert_eq!(map1.len(), 2);
    assert_eq!(map1.get("baz"), None);
    assert_ne!(map1, map2);
}

#[test]
fn lazy_hash_map_equal_for_nan() {
    let map1 = LazyHashMap::from([("nan".to_string(), f64::NAN)]);
    let map2 = LazyHashMap::from([("nan".to_string(), f64::NAN)]);
    assert_ne!(map1, map2);
}

#[test]
fn lazy_hash_map_debug() {
    let mut map = LazyHashMap::new();
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    map.clear();
    map.insert("foo".to_string(), 3);
    map.insert("key".to_string(), 1);
    let debug = format!("{:?}", map);
    assert!(debug == r#"{"foo": 3, "key": 1}"# || debug == r#"{"key": 1, "foo": 3}"#);
}
