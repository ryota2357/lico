use vm::code::Code::*;
use vm::runtime::{Object, Runtime};

#[test]
fn case1() {
    let mut runtime = Runtime::new(vec![]);

    #[rustfmt::skip]
    vm::execute(
        // var a = 1
        // vaf f = func() a = a + 10 end
        // f()
        &[
            LoadInt(1), MakeLocal("a"),

            BeginFuncCreation,
              AddCapture("a"),
              LoadLocal("a"), LoadInt(10), Add, SetLocal("a"),
              LoadNil, Return,
            EndFuncCreation,
            MakeLocal("f"),

            LoadLocal("f"), Call(0), UnloadTop,
            Exit,
        ],
        &mut runtime,
    ).unwrap();

    assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(11)));
    assert_eq!(
        runtime
            .variable_table
            .get("f")
            .unwrap()
            .ensure_function()
            .unwrap()
            .id,
        (2, 0)
    )
}

#[test]
fn case2() {
    let mut runtime = Runtime::new(vec![]);

    // var table = { key = "value" }
    runtime.variable_table.insert(
        "table",
        Object::new_table(vm::runtime::TableObject::new(
            [("key".to_string(), Object::String("value".to_string()))]
                .into_iter()
                .collect(),
        )),
    );

    #[rustfmt::skip]
    vm::execute(&[
        // func f(new_value)
        //     table.key = new_value
        // end
        // f(1.23)
        BeginFuncCreation,
          AddArgument("new_value"),
          AddCapture("table"),
          LoadLocal("new_value"),
          LoadLocal("table"), LoadStringAsRef("key"), SetItem,
          LoadNil, Return,
        EndFuncCreation,
        MakeLocal("f"),
        LoadLocal("f"), LoadFloat(1.23), Call(1),
        Exit,
    ], &mut runtime).unwrap();

    // assert: table.key == 1.23
    let table = if let Object::Table(table) = runtime.variable_table.get("table").unwrap() {
        table
    } else {
        unreachable!()
    };
    assert_eq!(table.borrow().get("key"), Some(&Object::Float(1.23)));
}

#[test]
fn case3() {
    let mut runtime = Runtime::new(vec![]);

    // var f = func(x) return x end
    runtime.variable_table.insert(
        "f",
        Object::new_function(vm::runtime::FunctionObject {
            id: (0, 0),
            env: vec![],
            args: vec!["x"],
            code: vec![LoadLocal("x"), Return],
        }),
    );

    #[rustfmt::skip]
    let res = vm::execute(
        // var ch = func()
        //    f = func(x) return x + 100 end
        //    return 10
        // end
        // return f(ch()) + f(1)
        &[
            BeginFuncCreation,
              AddCapture("f"),
              BeginFuncCreation,
                AddArgument("x"),
                LoadLocal("x"), LoadInt(100), Add,
                Return,
              EndFuncCreation,
              SetLocal("f"),
              LoadInt(10), Return,
            EndFuncCreation,
            MakeLocal("ch"),

            LoadLocal("f"),
              LoadLocal("ch"), Call(0),
            Call(1),
            LoadLocal("f"), LoadInt(1), Call(1),
            Add,
            Return,
        ],
        &mut runtime,
    );

    assert_eq!(res, Ok(Object::Int(111)));
}

#[test]
fn case4() {
    let mut runtime = Runtime::new(vec![]);

    // var a = 7
    runtime.variable_table.insert("a", Object::Int(7));

    #[rustfmt::skip]
    let res = vm::execute(&[
        // var c = func(b)
        //   a + b
        // end
        // return c(13)
        BeginFuncCreation,
          AddArgument("b"),
          AddCapture("a"),
          LoadLocal("a"), LoadLocal("b"), Add,
          Return,
        EndFuncCreation,
        MakeLocal("c"),
        LoadLocal("c"), LoadInt(13), Call(1),
        Return,
    ], &mut runtime);

    assert_eq!(res, Ok(Object::Int(20)));
}
