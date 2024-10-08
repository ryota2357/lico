use foundation::object::*;

/// &str -> UString -> Object
fn s(str: &str) -> Object {
    UString::from(str).into()
}

/// i64 -> Object
fn i(i: i64) -> Object {
    i.into()
}

/// f64 -> Object
fn f(f: f64) -> Object {
    f.into()
}

/// bool -> Object
fn b(b: bool) -> Object {
    b.into()
}

#[test]
fn u_string_construct_empty() {
    let empty = UString::new();
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert!(empty.is_ascii());
    assert_eq!(empty.as_str(), "");
    assert_eq!(empty.get(0), None);
    assert_eq!(empty.sub_string(0, 0), None);
    assert_eq!(empty, UString::default());
    assert_eq!(empty, UString::from(""));
    assert_eq!("", empty);
}

#[test]
fn u_string_construct_with_ascii() {
    let ascii = UString::from("ryota2357");
    assert_eq!(ascii.len(), 9);
    assert!(!ascii.is_empty());
    assert!(ascii.is_ascii());
    assert_eq!(ascii.as_str(), "ryota2357");
    assert_eq!("ryota2357", ascii);
}

#[test]
fn u_string_construct_with_unicode() {
    let unicode = UString::from("こんにちは");
    assert_eq!(unicode.len(), 5);
    assert!(!unicode.is_empty());
    assert!(!unicode.is_ascii());
    assert_eq!(unicode.as_str(), "こんにちは");
    assert_eq!("こんにちは", unicode);
}

#[test]
fn u_stirng_get() {
    let ascii = UString::from("abc");
    assert_eq!(ascii.get(0), Some('a'));
    assert_eq!(ascii.get(1), Some('b'));
    assert_eq!(ascii.get(2), Some('c'));
    assert_eq!(ascii.get(3), None);

    let u_char = UString::from("");
    assert_eq!(u_char.get(0), Some(''));
    assert_eq!(u_char.get(1), None);

    //                             0 1 2 345678901
    let non_ascii = UString::from("やあ、ryota2357");
    assert_eq!(non_ascii.get(0), Some('や'));
    assert_eq!(non_ascii.get(2), Some('、'));
    assert_eq!(non_ascii.get(6), Some('t'));
    assert_eq!(non_ascii.get(11), Some('7'));
    assert_eq!(non_ascii.get(12), None);
}

#[test]
fn u_string_sub_string() {
    let ascii = UString::from("abc");
    assert_eq!(ascii.sub_string(0, 3), Some(UString::from("abc")));
    assert_eq!(ascii.sub_string(1, 2), Some(UString::from("b")));
    assert_eq!(ascii.sub_string(1, 3), Some(UString::from("bc")));
    assert_eq!(ascii.sub_string(3, 3), Some(UString::from("")));
    assert_eq!(ascii.sub_string(2, 4), None);

    let u_char = UString::from("👍");
    assert_eq!(u_char.sub_string(0, 1), Some(UString::from("👍")));
    assert_eq!(u_char.sub_string(0, 0), Some(UString::from("")));
    assert_eq!(u_char.sub_string(1, 2), None);

    let non_ascii = UString::from("あaいbうcえdおe");
    assert_eq!(
        non_ascii.sub_string(0, 10),
        Some(UString::from("あaいbうcえdおe"))
    );
    assert_eq!(non_ascii.sub_string(1, 1), Some(UString::from("")));
    assert_eq!(non_ascii.sub_string(1, 2), Some(UString::from("a")));
    assert_eq!(non_ascii.sub_string(1, 5), Some(UString::from("aいbう")));
    assert_eq!(non_ascii.sub_string(5, 11), None);
}

#[test]
fn u_string_add_with_empty() {
    let empty = UString::new();

    let empty2 = empty.clone() + "";
    assert_eq!(empty2.len(), 0);
    assert!(empty2.is_empty());
    assert!(empty2.is_ascii());
    assert_eq!(empty2, "");

    let ascii = empty.clone() + "abc";
    assert_eq!(ascii.len(), 3);
    assert!(!ascii.is_empty());
    assert!(ascii.is_ascii());
    assert_eq!(ascii, "abc");

    let none_ascii = empty + "你好世界";
    assert_eq!(none_ascii.len(), 4);
    assert!(!none_ascii.is_empty());
    assert!(!none_ascii.is_ascii());
    assert_eq!(none_ascii, "你好世界");
}

#[test]
fn u_string_add_with_ascii() {
    let ascii = UString::from("abc");

    let empty = "" + ascii.clone();
    assert_eq!(empty.len(), 3);
    assert!(!empty.is_empty());
    assert!(empty.is_ascii());
    assert_eq!(empty, "abc");

    let ascii2 = ascii.clone() + "def";
    assert_eq!(ascii2.len(), 6);
    assert!(!ascii2.is_empty());
    assert!(ascii2.is_ascii());
    assert_eq!(ascii2, "abcdef");

    let none_ascii = ascii + " 👀";
    assert_eq!(none_ascii.len(), 5);
    assert!(!none_ascii.is_empty());
    assert!(!none_ascii.is_ascii());
    assert_eq!(none_ascii, "abc 👀");
}

#[test]
fn u_string_add_with_non_ascii() {
    let none_ascii = UString::from("Hello 世界");

    let empty = none_ascii.clone() + "";
    assert_eq!(empty.len(), 8);
    assert!(!empty.is_empty());
    assert!(!empty.is_ascii());
    assert_eq!(empty, "Hello 世界");

    let ascii = none_ascii.clone() + "!!";
    assert_eq!(ascii.len(), 10);
    assert!(!ascii.is_empty());
    assert!(!ascii.is_ascii());
    assert_eq!(ascii, "Hello 世界!!");

    let none_ascii2 = none_ascii + "。🫚🫚";
    assert_eq!(none_ascii2.len(), 11);
    assert!(!none_ascii2.is_empty());
    assert!(!none_ascii2.is_ascii());
    assert_eq!(none_ascii2, "Hello 世界。🫚🫚");
}

#[test]
fn u_string_clone_no_sync() {
    let mut s1 = UString::from("abc");
    let s2 = s1.clone();
    s1 += "def";
    assert_eq!(s2, "abc");
    assert_eq!(s1, "abcdef");
}

#[test]
fn array_construct_empty() {
    let array = Array::new();
    assert_eq!(array.version(), 0);
    assert_eq!(array.len(), 0);
    assert!(array.is_empty());
    assert_eq!(array.get(0), None);
    assert_eq!(array, Array::default());
}

#[test]
fn array_set_sync() {
    let mut array1 = Array::from([i(1), s("tow")]);
    let v = array1.version();
    array1.set(0, UString::from("one"));
    assert_ne!(array1.version(), v);
    assert_eq!(array1.get(0), Some(&s("one")));
    let mut array2 = array1.clone();
    let v = array1.version();
    array2.set(1, 2);
    assert_ne!(array1.version(), v);
    assert_eq!(array1.get(1), Some(&i(2)));
}

#[test]
fn array_push_sync() {
    let mut array1 = Array::new();
    let v = array1.version();
    array1.push(100);
    assert_ne!(array1.version(), v);
    assert_eq!(array1.get(0), Some(&i(100)));
    let mut array2 = array1.clone();
    let v = array1.version();
    array2.push(UString::from("hello"));
    assert_ne!(array1.version(), v);
    assert_eq!(array1.get(1), Some(&s("hello")));
}

#[test]
fn array_pop_sync() {
    let mut array1 = Array::from([b(true), f(1.23)]);
    let v = array1.version();
    assert_eq!(array1.pop(), Some(f(1.23)));
    assert_ne!(array1.version(), v);
    let mut array2 = array1.clone();
    let v = array1.version();
    assert_eq!(array2.pop(), Some(b(true)));
    assert_ne!(array1.version(), v);
    assert_eq!(array1.pop(), None);
}

#[test]
fn array_insert_sync() {
    let mut array1 = Array::from([i(1), s("tow")]);
    let v = array1.version();
    array1.insert(1, UString::from("one"));
    assert_ne!(array1.version(), v);
    assert_eq!(array1, Array::from([i(1), s("one"), s("tow")]));
    let mut array2 = array1.clone();
    let v = array1.version();
    array2.insert(0, 0);
    assert_ne!(array1.version(), v);
    assert_eq!(array1, Array::from([i(0), i(1), s("one"), s("tow")]));
}

#[test]
fn array_remove_sync() {
    let mut array1 = Array::from([i(1), s("tow")]);
    let v = array1.version();
    assert_eq!(array1.remove(0), i(1));
    assert_ne!(array1.version(), v);
    assert_eq!(array1, Array::from([s("tow")]));
    let mut array2 = array1.clone();
    let v = array1.version();
    assert_eq!(array2.remove(0), s("tow"));
    assert_ne!(array1.version(), v);
    assert_eq!(array1, Array::new());
}

#[test]
fn array_clear_sync() {
    let mut array1 = Array::from([i(1), s("tow")]);
    assert!(!array1.is_empty());
    let array2 = array1.clone();
    let v = array1.version();
    array1.clear();
    assert_ne!(array1.version(), v);
    assert_eq!(array1, Array::new());
    assert_eq!(array2, Array::new());
    assert!(array1.is_empty());
}

#[test]
fn array_contains() {
    let array = Array::from([i(1), s("tow")]);
    assert!(array.contains(&s("tow")));
    let v = array.version();
    assert!(!array.contains(&i(3)));
    assert_eq!(array.version(), v);
}

#[test]
fn array_unsafe_iter() {
    let array = Array::from([i(1), s("tow")]);
    let mut iter = unsafe { array.iter() };
    assert_eq!(iter.next(), Some(&i(1)));
    assert_eq!(iter.next(), Some(&s("tow")));
    assert_eq!(iter.next(), None);
}

#[test]
fn array_equal() {
    let array1 = Array::from([i(1), b(true), Array::new().into()]);
    let array2 = Array::from([i(1), b(true), Array::new().into()]);
    assert_eq!(array1, array2);
}

#[test]
fn array_equal_for_nan() {
    let array1 = Array::from([f(f64::NAN)]);
    let array2 = array1.clone();
    assert_ne!(array1, array2);
}

#[test]
fn table_construct_empty() {
    let table = Table::new();
    assert_eq!(table.len(), 0);
    assert!(table.is_empty());
    assert_eq!(table.get("key"), None);
    assert_eq!(table, Table::default());
}

#[test]
fn table_insert_sync() {
    let mut table1 = Table::new();
    table1.insert("key".into(), 100);
    assert_eq!(table1.get("key"), Some(&i(100)));
    let mut table2 = table1.clone();
    table2.insert("key".into(), UString::from("hello"));
    assert_eq!(table1.get("key"), Some(&s("hello")));
    table1.insert("foo".into(), true);
    assert_eq!(
        table2,
        Table::from([("key".into(), s("hello")), ("foo".into(), b(true))])
    );
}

#[test]
fn table_remove_sync() {
    let mut table1 = Table::from([("key".into(), f(1.23)), ("foo".into(), b(true))]);
    assert_eq!(table1.remove("key"), Some(f(1.23)));
    assert_eq!(table1, Table::from([("foo".into(), b(true))]));
    let mut table2 = table1.clone();
    assert_eq!(table2.remove("foo"), Some(b(true)));
    assert_eq!(table1, Table::new());
}

#[test]
fn table_clear_sync() {
    let mut table1 = Table::from([("key".into(), f(1.23)), ("foo".into(), b(true))]);
    assert!(!table1.is_empty());
    let table2 = table1.clone();
    table1.clear();
    assert_eq!(table1, Table::new());
    assert_eq!(table2, Table::new());
    assert!(table1.is_empty());
}

#[test]
fn table_contains_key() {
    let table = Table::from([("foo".into(), b(true)), ("bar".into(), f(1.23))]);
    assert!(table.contains_key("foo"));
    assert!(!table.contains_key("baz"));
}

#[test]
fn table_unsafe_iter() {
    let table = Table::from([("foo".into(), b(true)), ("bar".into(), f(1.23))]);
    let mut iter = unsafe { table.iter() };
    let mut nexts = [iter.next().unwrap(), iter.next().unwrap()];
    nexts.sort_by_key(|(key, _)| *key);
    assert_eq!(nexts[0], (&"bar".into(), &f(1.23)));
    assert_eq!(nexts[1], (&"foo".into(), &b(true)));
    assert_eq!(iter.next(), None);
}

#[test]
fn table_equal() {
    let table1 = Table::from([("1".into(), i(1)), ("tbl".into(), Table::new().into())]);
    let table2 = Table::from([("1".into(), i(1)), ("tbl".into(), Table::new().into())]);
    assert_eq!(table1, table2);
}

#[test]
fn table_equal_for_nan() {
    let table1 = Table::from([("key".into(), f64::NAN.into())]);
    let table2 = table1.clone();
    assert_ne!(table1, table2);
}
