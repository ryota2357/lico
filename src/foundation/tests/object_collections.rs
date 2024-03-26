use foundation::object::collections::*;

#[test]
fn linear_map_construct() {
    const _: LinearMap<i32, i32> = LinearMap::new();
    let map = LinearMap::<&str, &str>::new();
    assert_eq!(map.len(), 0);
    assert_eq!(map.is_empty(), true);
    assert_eq!(map.get("one"), None);
    assert_eq!(map, LinearMap::default());
}

#[test]
fn linear_map_insert() {
    let mut map = LinearMap::from([("zero", 0), ("two", 2), ("three", 3)]);
    map.insert("one", 1);
    assert_eq!(map.len(), 4);
    assert_eq!(map.is_empty(), false);
    assert_eq!(map.get("zero"), Some(&0));
    assert_eq!(map.get("one"), Some(&1));
    assert_eq!(map.get("four"), None);
}

#[test]
fn linear_map_remove() {
    let mut map = LinearMap::from([("zero", 0), ("one", 1), ("two", 2), ("three", 3)]);
    assert_eq!(map.remove("two"), Some(2));
    assert_eq!(map.remove("four"), None);
    assert_eq!(map.len(), 3);
    assert_eq!(map.is_empty(), false);
    assert_eq!(
        map,
        LinearMap::from([("zero", 0), ("one", 1), ("three", 3)])
    );
}

#[test]
fn linear_map_clear() {
    let mut map = LinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    assert_eq!(map.is_empty(), false);
    map.clear();
    assert_eq!(map.len(), 0);
    assert_eq!(map.is_empty(), true);
    assert_eq!(map, LinearMap::default());
}

#[test]
fn linear_map_contains_key() {
    let map = LinearMap::from([("aaa", 0), ("aad", 1), ("aac", 2)]);
    assert_eq!(map.contains_key("aac"), true);
    assert_eq!(map.contains_key("aab"), false);
}

#[test]
fn linear_map_iter() {
    let map = LinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut iter = map.iter();
    assert_eq!(iter.next(), Some(&("one", 1)));
    assert_eq!(iter.next(), Some(&("two", 2)));
    assert_eq!(iter.next(), Some(&("zero", 0)));
    assert_eq!(iter.next(), None);
}

#[test]
fn linear_map_iter_mut() {
    let mut map = LinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut iter = map.iter_mut();
    assert_eq!(iter.next(), Some(&mut ("one", 1)));
    assert_eq!(iter.next(), Some(&mut ("two", 2)));
    assert_eq!(iter.next(), Some(&mut ("zero", 0)));
    assert_eq!(iter.next(), None);
}

#[test]
fn linear_map_drain() {
    let mut map = LinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut iter = map.drain();
    assert_eq!(iter.next(), Some(("one", 1)));
    assert_eq!(iter.next(), Some(("two", 2)));
    assert_eq!(iter.next(), Some(("zero", 0)));
    assert_eq!(iter.next(), None);
}

#[test]
fn linear_map_into_iter() {
    let map = LinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut iter = map.into_iter();
    assert_eq!(iter.next(), Some(("one", 1)));
    assert_eq!(iter.next(), Some(("two", 2)));
    assert_eq!(iter.next(), Some(("zero", 0)));
    assert_eq!(iter.next(), None);
}

#[test]
fn linear_map_no_sync_with_clone() {
    let map1 = LinearMap::from([("zero", 0), ("one", 1), ("two", 2)]);
    let mut map2 = map1.clone();
    map2.insert("three", 3);
    assert_eq!(map1.len(), 3);
    assert_eq!(map1.get("three"), None);
    assert_ne!(map1, map2);
}

#[test]
fn linear_map_equal_for_nan() {
    let map1 = LinearMap::from([("nan", f64::NAN)]);
    let map2 = LinearMap::from([("nan", f64::NAN)]);
    assert_ne!(map1, map2);
}

#[test]
fn linear_map_debug() {
    let map = LinearMap::from([("key", 1), ("foo", 3)]);
    assert_eq!(format!("{:?}", map), r#"{"foo": 3, "key": 1}"#);
}

#[test]
fn switch_map_construct() {
    const _: SwitchMap<i32, i32> = SwitchMap::new();
    let map = SwitchMap::<&str, &str>::new();
    assert_eq!(map.len(), 0);
    assert_eq!(map.is_empty(), true);
    assert_eq!(map.get("one"), None);
    assert_eq!(map, SwitchMap::default());
}

#[test]
fn switch_map_insert() {
    let mut map = SwitchMap::new();
    map.insert("foo".to_string(), -1);
    map.insert("bar".to_string(), -2);
    assert_eq!(map.len(), 2);
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert_eq!(map.len(), 102);
    assert_eq!(map.is_empty(), false);
    assert_eq!(map.get("bar"), Some(&-2));
    assert_eq!(map.get("fo"), None);
    assert_eq!(map.get("key99"), Some(&99));
}

#[test]
fn switch_map_remove() {
    let mut map = SwitchMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    assert_eq!(map.remove("foo"), Some(-1));
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert_eq!(map.remove("bar"), Some(-2));
    assert_eq!(map.remove("bar"), None);
    assert_eq!(map.len(), 100);
}

#[test]
fn switch_map_clear() {
    let mut map = SwitchMap::new();
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert_eq!(map.is_empty(), false);
    map.clear();
    assert_eq!(map.len(), 0);
    assert_eq!(map.is_empty(), true);
    assert_eq!(map, SwitchMap::default());
}

#[test]
fn switch_map_contains_key() {
    let mut map = SwitchMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    assert_eq!(map.contains_key("foo"), true);
    assert_eq!(map.contains_key("baz"), false);
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert_eq!(map.contains_key("foo"), true);
    assert_eq!(map.contains_key("baz"), false);
    assert_eq!(map.contains_key("key99"), true);
    assert_eq!(map.contains_key("key100"), false);
}

#[test]
fn switch_map_iter() {
    let mut map = SwitchMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    assert_eq!(map.len(), 102);
    let mut iter = map.iter();
    let mut nexts = std::collections::HashMap::new();
    for _ in 0..102 {
        let (key, value) = iter.next().unwrap();
        nexts.insert(key.clone(), value.clone());
    }
    assert_eq!(nexts.get("bar"), Some(&-2));
    assert_eq!(nexts.get("baz"), None);
    assert_eq!(nexts.get("key57"), Some(&57));
    assert_eq!(iter.next(), None);
}

#[test]
fn switch_map_iter_mut() {
    let mut map = SwitchMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
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
fn switch_map_drain() {
    let mut map = SwitchMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
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
fn switch_map_into_iter() {
    let mut map = SwitchMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
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
fn switch_map_no_sync_with_clone() {
    let map1 = SwitchMap::from([("foo".to_string(), -1), ("bar".to_string(), -2)]);
    let mut map2 = map1.clone();
    map2.insert("baz".to_string(), -3);
    assert_eq!(map1.len(), 2);
    assert_eq!(map1.get("baz"), None);
    assert_ne!(map1, map2);
}

#[test]
fn switch_map_equal_for_nan() {
    let map1 = SwitchMap::from([("nan".to_string(), f64::NAN)]);
    let map2 = SwitchMap::from([("nan".to_string(), f64::NAN)]);
    assert_ne!(map1, map2);
}

#[test]
fn switch_map_debug() {
    let mut map = SwitchMap::new();
    for i in 0..100 {
        map.insert(format!("key{}", i), i);
    }
    map.clear();
    map.insert("foo".to_string(), 3);
    map.insert("key".to_string(), 1);
    let debug = format!("{:?}", map);
    assert!(debug == r#"{"foo": 3, "key": 1}"# || debug == r#"{"key": 1, "foo": 3}"#);
}
