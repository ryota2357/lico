use std::rc::Rc;
use vm::{
    code::{ArgumentKind, Code::*, LocalId},
    runtime::{Object, Runtime},
};

#[test]
fn case1() {
    // var a = 1
    // vaf f = func() a = a + 10 end
    // f()

    let mut runtime = Runtime::new();

    #[rustfmt::skip]
    vm::execute(
        &[
            // a: 0
            // f: 1
            LoadInt(1), MakeLocal,

            BeginFuncCreation,
              AddCapture(LocalId(0)),
              LoadLocal(LocalId(0)), LoadInt(10), Add, SetLocal(LocalId(0)),
              LoadNil, Return,
            EndFuncCreation,
            MakeLocal,

            LoadLocal(LocalId(1)), Call(0), UnloadTop,
            Exit,
        ],
        &mut runtime,
    ).unwrap();

    assert_eq!(runtime.variable_table.get(LocalId(0)), Object::Int(11));
    assert_eq!(
        runtime
            .variable_table
            .get(LocalId(1))
            .ensure_function()
            .unwrap()
            .id,
        (2, 0)
    )
}

#[test]
fn case2() {
    // var table = { key = "value" }
    // func f(new_value)
    //     table.key = new_value
    // end
    // f(1.23)

    let mut runtime = Runtime::new();

    // table: 0
    // f: 1
    //   table: 0
    //   new_value: 1
    runtime
        .variable_table
        .push(Object::new_table(vm::runtime::TableObject::new(
            [("key".into(), Object::new_string("value".to_string()))]
                .into_iter()
                .collect(),
        )));
    #[rustfmt::skip]
    vm::execute(&[
        BeginFuncCreation,
          AddCapture(LocalId(0)),
          AddArgument(ArgumentKind::Copy),
          LoadLocal(LocalId(1)), LoadLocal(LocalId(0)), LoadString(Rc::new("key".to_string())), SetItem,
          LoadNil, Return,
        EndFuncCreation,
        MakeLocal,
        LoadLocal(LocalId(1)), LoadFloat(1.23), Call(1),
        Exit,
    ], &mut runtime).unwrap();

    // assert: table.key == 1.23
    let table = if let Object::Table(table) = runtime.variable_table.get(LocalId(0)) {
        table
    } else {
        unreachable!()
    };
    assert_eq!(table.borrow().get("key"), Some(&Object::Float(1.23)));
}

#[test]
fn case3() {
    // var f = func(x) return x end
    // var ch = func()
    //    f = func(x) return x + 100 end
    //    return 10
    // end
    // return f(ch()) + f(1)

    let mut runtime = Runtime::new();

    // f: 0
    //   x: 0
    // ch: 1
    //   f: 0
    //     x: 0
    runtime
        .variable_table
        .push(Object::new_function(vm::runtime::FunctionObject {
            id: (0, 0),
            env: vec![],
            args: vec![ArgumentKind::Copy],
            code: vec![LoadLocal(LocalId(0)), Return],
        }));
    #[rustfmt::skip]
    let res = vm::execute(
        &[
            BeginFuncCreation,
              AddCapture(LocalId(0)),
              BeginFuncCreation,
                AddArgument(ArgumentKind::Copy),
                LoadLocal(LocalId(0)), LoadInt(100), Add,
                Return,
              EndFuncCreation,
              SetLocal(LocalId(0)),
              LoadInt(10), Return,
            EndFuncCreation,
            MakeLocal,

            LoadLocal(LocalId(0)),
              LoadLocal(LocalId(1)), Call(0),
            Call(1),
            LoadLocal(LocalId(0)), LoadInt(1), Call(1),
            Add,
            Return,
        ],
        &mut runtime,
    );

    assert_eq!(res, Ok(Object::Int(111)));
}

#[test]
fn case4() {
    // var a = 7
    // var c = func(b)
    //   return a + b
    // end
    // return c(13)

    let mut runtime = Runtime::new();

    // a: 0
    // c: 1
    //   a: 0
    //   b: 1
    runtime.variable_table.push(Object::Int(7));
    #[rustfmt::skip]
    let res = vm::execute(&[
        BeginFuncCreation,
          AddCapture(LocalId(0)),
          AddArgument(ArgumentKind::Copy),
          LoadLocal(LocalId(0)), LoadLocal(LocalId(1)), Add,
          Return,
        EndFuncCreation,
        MakeLocal,
        LoadLocal(LocalId(1)), LoadInt(13), Call(1),
        Return,
    ], &mut runtime);

    assert_eq!(res, Ok(Object::Int(20)));
}
