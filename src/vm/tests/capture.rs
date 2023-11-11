use vm::code::Code::*;
use vm::runtime::{Object, Runtime};

#[test]
fn case1() {
    let mut runtime = Runtime::new();

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
    );

    assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(11)));
    match runtime.variable_table.get("f").unwrap() {
        Object::Function(func) => {
            assert_eq!(func.id, (2, 0));
        }
        _ => panic!(
            "Expected Function, but got {:?}",
            runtime.variable_table.get("f")
        ),
    };
}

#[test]
fn case2() {
    let mut runtime = Runtime::new();
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
        // var f = func(x) return x end
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
    assert_eq!(res, Object::Int(111));
}
