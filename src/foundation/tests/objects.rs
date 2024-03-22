use foundation::object::*;

#[test]
fn array_construct_empty() {
    let array = Array::new();
    assert_eq!(array.version(), 0);
    assert_eq!(array.len(), 0);
    assert_eq!(array.is_empty(), true);
    assert_eq!(array.get(0), None);
    assert_eq!(array, Array::default());
}

#[test]
fn array_set_sync() {
    let mut array1 = Array::from([1.into(), "tow".into()]);
    let v = array1.version();
    array1.set(0, "one");
    assert_ne!(array1.version(), v);
    assert_eq!(array1.get(0), Some(&"one".into()));
    let mut array2 = array1.clone();
    let v = array1.version();
    array2.set(1, 2);
    assert_ne!(array1.version(), v);
    assert_eq!(array1.get(1), Some(&2.into()));
}

#[test]
fn array_push_sync() {
    let mut array1 = Array::new();
    let v = array1.version();
    array1.push(100);
    assert_ne!(array1.version(), v);
    assert_eq!(array1.get(0), Some(&100.into()));
    let mut array2 = array1.clone();
    let v = array1.version();
    array2.push("hello");
    assert_ne!(array1.version(), v);
    assert_eq!(array1.get(1), Some(&"hello".into()));
}

#[test]
fn array_pop_sync() {
    let mut array1 = Array::from([true.into(), 3.14.into()]);
    let v = array1.version();
    assert_eq!(array1.pop(), Some(3.14.into()));
    assert_ne!(array1.version(), v);
    let mut array2 = array1.clone();
    let v = array1.version();
    assert_eq!(array2.pop(), Some(true.into()));
    assert_ne!(array1.version(), v);
    assert_eq!(array1.pop(), None);
}

#[test]
fn array_insert_sync() {
    let mut array1 = Array::from([1.into(), "tow".into()]);
    let v = array1.version();
    array1.insert(1, "one");
    assert_ne!(array1.version(), v);
    assert_eq!(array1, Array::from([1.into(), "one".into(), "tow".into()]));
    let mut array2 = array1.clone();
    let v = array1.version();
    array2.insert(0, 0);
    assert_ne!(array1.version(), v);
    assert_eq!(
        array1,
        Array::from([0.into(), 1.into(), "one".into(), "tow".into()])
    );
}

#[test]
fn array_remove_sync() {
    let mut array1 = Array::from([1.into(), "tow".into()]);
    let v = array1.version();
    assert_eq!(array1.remove(0), 1.into());
    assert_ne!(array1.version(), v);
    assert_eq!(array1, Array::from(["tow".into()]));
    let mut array2 = array1.clone();
    let v = array1.version();
    assert_eq!(array2.remove(0), "tow".into());
    assert_ne!(array1.version(), v);
    assert_eq!(array1, Array::new());
}

#[test]
fn array_clear_sync() {
    let mut array1 = Array::from([1.into(), "tow".into()]);
    assert_eq!(array1.is_empty(), false);
    let array2 = array1.clone();
    let v = array1.version();
    array1.clear();
    assert_ne!(array1.version(), v);
    assert_eq!(array1, Array::new());
    assert_eq!(array2, Array::new());
    assert_eq!(array1.is_empty(), true);
}

#[test]
fn array_contains() {
    let array = Array::from([1.into(), "tow".into()]);
    assert!(array.contains(&"tow".into()));
    let v = array.version();
    assert!(!array.contains(&3.into()));
    assert_eq!(array.version(), v);
}

#[test]
fn array_unsafe_iter() {
    let array = Array::from([1.into(), "tow".into()]);
    let mut iter = unsafe { array.iter() };
    assert_eq!(iter.next(), Some(&1.into()));
    assert_eq!(iter.next(), Some(&"tow".into()));
    assert_eq!(iter.next(), None);
}

#[test]
fn array_equal_for_nan() {
    let array1 = Array::from([f64::NAN.into()]);
    let array2 = array1.clone();
    assert_ne!(array1, array2);
}

#[test]
fn table_construct_empty() {
    let table = Table::new();
    assert_eq!(table.len(), 0);
    assert_eq!(table.is_empty(), true);
    assert_eq!(table.get("key"), None);
    assert_eq!(table, Table::default());
}

#[test]
fn table_insert_sync() {
    let mut table1 = Table::new();
    table1.insert("key".into(), 100);
    assert_eq!(table1.get("key"), Some(&100.into()));
    let mut table2 = table1.clone();
    table2.insert("key".into(), "hello");
    assert_eq!(table1.get("key"), Some(&"hello".into()));
    table1.insert("foo".into(), true);
    assert_eq!(
        table2,
        Table::from([("key".into(), "hello".into()), ("foo".into(), true.into())])
    );
}

#[test]
fn table_remove_sync() {
    let mut table1 = Table::from([("key".into(), 1.23.into()), ("foo".into(), true.into())]);
    assert_eq!(table1.remove("key"), Some(1.23.into()));
    assert_eq!(table1, Table::from([("foo".into(), true.into())]));
    let mut table2 = table1.clone();
    assert_eq!(table2.remove("foo"), Some(true.into()));
    assert_eq!(table1, Table::new());
}

#[test]
fn table_clear_sync() {
    let mut table1 = Table::from([("key".into(), 1.23.into()), ("foo".into(), true.into())]);
    assert_eq!(table1.is_empty(), false);
    let table2 = table1.clone();
    table1.clear();
    assert_eq!(table1, Table::new());
    assert_eq!(table2, Table::new());
    assert_eq!(table1.is_empty(), true);
}

#[test]
fn table_contains_key() {
    let table = Table::from([("foo".into(), true.into()), ("bar".into(), 3.14.into())]);
    assert!(table.contains_key("foo"));
    assert!(!table.contains_key("baz"));
}

#[test]
fn table_unsafe_iter() {
    let table = Table::from([("foo".into(), true.into()), ("bar".into(), 3.14.into())]);
    let mut iter = unsafe { table.iter() };
    let mut nexts = vec![iter.next().unwrap(), iter.next().unwrap()];
    nexts.sort_by_key(|(key, _)| *key);
    assert_eq!(nexts[0], (&"bar".into(), &3.14.into()));
    assert_eq!(nexts[1], (&"foo".into(), &true.into()));
    assert_eq!(iter.next(), None);
}

#[test]
fn table_equal_for_nan() {
    let table1 = Table::from([("key".into(), f64::NAN.into())]);
    let table2 = table1.clone();
    assert_ne!(table1, table2);
}
