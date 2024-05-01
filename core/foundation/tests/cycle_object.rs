use foundation::object::*;

use mockalloc::Mockalloc;

#[global_allocator]
static ALLOCATOR: Mockalloc<std::alloc::System> = Mockalloc(std::alloc::System);

#[mockalloc::test]
fn no_cycle() {
    let mut array = Array::new();
    array.push(Object::Int(10));
    array.push(Object::String("Hello".into()));
    array.push(Object::Table(Table::from([(
        "key".into(),
        Object::Bool(true),
    )])));

    let mut table = Table::new();
    table.insert("a".into(), Object::Float(1.23));
    table.insert(
        "b".into(),
        Object::String("Lo~~~~~~~~~~~~~~~~ng Text".into()),
    );
    table.insert("c".into(), Object::Array(Array::from(vec![Object::Nil])));
}

#[mockalloc::test]
fn self_cycle() {
    let mut array = Array::new();
    array.push(Object::Int(10));
    array.push(Object::Array(array.clone()));

    let mut table = Table::new();
    table.insert("str".into(), Object::Float(1.23));
    table.insert("self".into(), Object::Table(table.clone()));
}

#[mockalloc::test]
fn array_cycle() {
    {
        let mut array1 = Array::new();
        let mut array2 = Array::new();
        array1.push(Object::Array(array2.clone()));
        array2.push(Object::Array(array1.clone()));
    }
    {
        let mut array1 = Array::new();
        let mut array2 = Array::new();
        let mut array3 = Array::new();
        array1.push(Object::Array(array2.clone()));
        array2.push(Object::Array(array3.clone()));
        array3.push(Object::Array(array1.clone()));
    }
}

#[mockalloc::test]
fn table_cycle() {
    {
        let mut table1 = Table::new();
        let mut table2 = Table::new();
        table1.insert("table2".into(), Object::Table(table2.clone()));
        table2.insert("table1".into(), Object::Table(table1.clone()));
    }
    {
        let mut table1 = Table::new();
        let mut table2 = Table::new();
        let mut table3 = Table::new();
        table1.insert("table2".into(), Object::Table(table2.clone()));
        table2.insert("table3".into(), Object::Table(table3.clone()));
        table3.insert("table1".into(), Object::Table(table1.clone()));
    }
}

#[mockalloc::test]
fn mixed_cycle() {
    {
        let mut array = Array::new();
        let mut table = Table::new();
        array.push(Object::Table(table.clone()));
        table.insert("array".into(), Object::Array(array.clone()));
    }
    {
        let mut array1 = Array::new();
        let mut array2 = Array::new();
        let mut table1 = Table::new();
        let mut table2 = Table::new();
        array1.push(Object::Table(table1.clone()));
        table1.insert("array".into(), Object::Array(array2.clone()));
        array2.push(Object::Table(table2.clone()));
        table2.insert("array".into(), Object::Array(array1.clone()));
    }
}

#[mockalloc::test]
fn access_check() {
    {
        let mut array = Array::new();
        {
            let mut table = Table::new();
            table.insert("value".into(), Object::Int(3));
            array.push(Object::Table(table.clone()));
            drop(table);
        }
        assert_eq!(
            match array.get(0).unwrap() {
                Object::Table(table) => table.get("value").unwrap(),
                _ => unreachable!(),
            },
            &Object::Int(3)
        );
    }
    {
        let mut table = Table::new();
        {
            let mut array = Array::new();
            array.push(Object::Int(7));
            table.insert("array".into(), Object::Array(array.clone()));
            drop(array);
        }
        assert_eq!(
            match table.get("array").unwrap() {
                Object::Array(array) => array.get(0).unwrap(),
                _ => unreachable!(),
            },
            &Object::Int(7)
        );
    }
}

#[mockalloc::test]
fn cycle_eq() {
    {
        let mut a = Array::from([1.0.into()]);
        let mut b = Array::from([1.0.into()]);
        a.push(a.clone());
        b.push(a.clone());
        assert_eq!(a, b);
        a.push(f64::NAN);
        b.push(f64::NAN);
        assert_ne!(a, b);
    }
    {
        let mut a = Table::from([("key".into(), 1.0.into())]);
        let mut b = Table::from([("key".into(), 1.0.into())]);
        a.insert("a".into(), Object::Table(a.clone()));
        b.insert("a".into(), Object::Table(a.clone()));
        assert_eq!(a, b);
        a.insert("nan".into(), Object::Float(f64::NAN));
        b.insert("nan".into(), Object::Float(f64::NAN));
        assert_ne!(a, b);
    }
}

#[mockalloc::test]
fn complex_case1() {
    /*
     * A     E
     * | \__ |
     * B    \|
     * | \   F->G
     * C--D
     */
    macro_rules! define_table {
        ($($name:ident),*) => {
            $(
                let mut $name = Table::new();
                $name.insert("name".into(), Object::String(stringify!($name).into()));
            )*
        };
    }
    macro_rules! insert_table {
        ($($self:ident : [$($name:ident),*]),* $(,)?) => {
            $(
                $( $self.insert(stringify!($name).into(), Object::Table($name.clone())); )*
            )*
        };
    }
    define_table!(a, b, c, d, e, f, g);
    insert_table! {
        a: [b, f],
        b: [c, d],
        c: [b, d],
        d: [c, b],
        e: [f],
        f: [g],
    }

    // drop except a and e
    drop(b);
    drop(c);
    drop(d);
    drop(f);
    drop(g);

    // then drop a
    drop(a);

    // check f and g are alive
    assert_eq!(
        match e.get("f").unwrap() {
            Object::Table(f) => {
                let name = f.get("name").unwrap();
                let g = match f.get("g").unwrap() {
                    Object::Table(g) => g,
                    _ => unreachable!(),
                };
                (name, g.get("name").unwrap())
            }
            _ => unreachable!(),
        },
        (&Object::String("f".into()), &Object::String("g".into()))
    );
}

#[mockalloc::test]
fn complex_case2() {
    let mut a = Array::new();
    let mut b = Array::new();
    let mut c = Array::new();
    a.push(b.clone());
    a.push(c.clone());
    b.push(a.clone());
    b.push(c.clone());
    c.push(a.clone());
    c.push(b.clone());

    let mut root = Array::new();
    root.push(a);
    root.push(b);
    root.push(c);

    drop(root);
}
