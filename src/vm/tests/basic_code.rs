use vm::code::Code::*;
use vm::runtime::{Object, Runtime};

#[test]
#[rustfmt::skip]
fn load() {
    let mut runtime = Runtime::new(vec![]);
    vm::execute(&[
        LoadInt(37), LoadFloat(42.0), LoadBool(true), LoadString("a b".to_string()), LoadStringAsRef("c"), LoadNil,
        Exit,
    ], &mut runtime).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Nil);
    assert_eq!(runtime.stack.pop().ensure_object(), Object::String("c".to_string()));
    assert_eq!(runtime.stack.pop().ensure_object(), Object::String("a b".to_string()));
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Bool(true));
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Float(42.0));
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(37));
}

#[test]
fn load_local() {
    let mut runtime = Runtime::new(vec![]);
    runtime.variable_table.insert("a", Object::Int(1));
    vm::execute(&[LoadLocal("a"), Exit], &mut runtime).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(1));
}

#[test]
fn load_rust_function() {
    let mut runtime = Runtime::new(vec![]);
    let res = vm::execute(
        &[
            LoadRustFunction(|obj| {
                assert_eq!(obj.len(), 1);
                assert_eq!(obj[0], Object::Int(1));
                Ok(Object::Int(2))
            }),
            LoadInt(1),
            Call(1),
            Return,
        ],
        &mut runtime,
    );
    assert_eq!(res.unwrap(), Object::Int(2));
}

#[test]
#[should_panic(expected = "[INTERNAL] Stack is empty.")]
fn unload() {
    let mut runtime = Runtime::new(vec![]);
    runtime.stack.push(Object::Int(0).into());
    vm::execute(&[UnloadTop, Exit], &mut runtime).unwrap();
    runtime.stack.pop(); // panic
}

#[test]
fn set_local() {
    // a = 10
    let mut runtime = Runtime::new(vec![]);
    runtime.variable_table.insert("a", Object::Int(1));
    vm::execute(&[LoadInt(10), SetLocal("a"), Exit], &mut runtime).unwrap();
    assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(10)));
}

#[test]
fn make_local() {
    // var a = 1
    let mut runtime = Runtime::new(vec![]);
    vm::execute(&[LoadInt(1), MakeLocal("a"), Exit], &mut runtime).unwrap();
    assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(1)));
}

#[test]
fn make_array() {
    use vm::runtime::StackValue;

    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[LoadInt(1), LoadInt(2), LoadInt(3), MakeArray(3), Exit],
        &mut runtime,
    )
    .unwrap();
    assert_eq!(
        runtime.stack.pop(),
        StackValue::RawArray(vec![Object::Int(1), Object::Int(2), Object::Int(3)])
    );
}

#[test]
fn make_named() {
    let mut runtime = Runtime::new(vec![]);
    vm::execute(&[LoadNil, MakeNamed("NILL"), Exit], &mut runtime).unwrap();
    assert_eq!(
        runtime.stack.pop().ensure_named(),
        ("NILL".to_string(), Object::Nil)
    );
}

#[test]
fn make_expr_named() {
    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[
            LoadInt(1),
            LoadString("Key".to_string()),
            MakeExprNamed,
            Exit,
        ],
        &mut runtime,
    )
    .unwrap();
    assert_eq!(
        runtime.stack.pop().ensure_named(),
        ("Key".to_string(), Object::Int(1))
    );
}

#[test]
fn make_table() {
    use std::{cell::RefCell, rc::Rc};
    use vm::runtime::TableObject;

    let mut runtime = Runtime::new(vec![]);
    for (key, value) in [
        ("Key1", Object::Int(1)),
        ("Key2", Object::Bool(true)),
        ("Key3", Object::String("a".to_string())),
    ] {
        runtime.stack.push((key.to_string(), value).into());
    }
    vm::execute(&[MakeTable(2), Exit], &mut runtime).unwrap();

    assert_eq!(
        runtime.stack.pop().ensure_object(),
        Object::Table(Rc::new(RefCell::new(TableObject::new(
            vec![
                ("Key2".to_string(), Object::Bool(true)),
                ("Key3".to_string(), Object::String("a".to_string())),
            ]
            .into_iter()
            .collect()
        ))))
    );
    assert_eq!(
        runtime.stack.pop().ensure_named(),
        ("Key1".to_string(), Object::Int(1))
    );
}

#[test]
fn drop_local() {
    let mut runtime = Runtime::new(vec![]);
    runtime.variable_table.insert("a", Object::Int(1));
    runtime.variable_table.insert("a", Object::Int(2));
    runtime.variable_table.insert("a", Object::Int(3));
    assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(3)));
    vm::execute(&[DropLocal(2), Exit], &mut runtime).unwrap();
    assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(1)));
}

#[test]
fn jump() {
    let mut runtime = Runtime::new(vec![]);
    runtime.variable_table.insert("a", Object::Int(1));
    vm::execute(&[Jump(2), DropLocal(1), Exit], &mut runtime).unwrap();
    assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(1)));

    let mut runtime = Runtime::new(vec![]);
    runtime.variable_table.insert("a", Object::Int(1));
    vm::execute(&[Jump(3), DropLocal(1), Exit, Jump(-2)], &mut runtime).unwrap();
    assert_eq!(runtime.variable_table.get("a"), None);
}

#[test]
#[rustfmt::skip]
fn jump_if_true() {
    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[LoadBool(true), JumpIfTrue(3), Nop, Exit, LoadInt(1), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(1));

    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[LoadBool(false), JumpIfTrue(3), LoadInt(2), Exit, Nop, Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(2));

    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[Jump(3), LoadInt(3), Exit, LoadBool(true), JumpIfTrue(-3), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(3));

    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[Jump(2), Exit, LoadBool(false), JumpIfTrue(-2), LoadInt(4), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(4));
}

#[test]
#[rustfmt::skip]
fn jump_if_false() {
    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[LoadBool(false), JumpIfFalse(3), Nop, Exit, LoadInt(1), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(1));

    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[LoadBool(true), JumpIfFalse(3), LoadInt(2), Exit, Nop, Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(2));

    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[Jump(3), LoadInt(3), Exit, LoadBool(false), JumpIfFalse(-3), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(3));

    let mut runtime = Runtime::new(vec![]);
    vm::execute(
        &[Jump(2), Exit, LoadBool(true), JumpIfFalse(-2), LoadInt(4), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(4));
}

#[test]
fn custom_method() {
    use vm::runtime::{FunctionObject, TableObject};

    let table_obj = {
        let mut table = TableObject::new(
            [("key".to_string(), Object::String("value".to_string()))]
                .into_iter()
                .collect(),
        );
        table.add_method(
            "testMethod",
            FunctionObject {
                id: (0, 0),
                env: vec![],
                args: vec!["self", "new_value"],
                code: vec![
                    LoadLocal("new_value"),
                    LoadLocal("self"),
                    LoadStringAsRef("key"),
                    SetItem,
                    LoadNil,
                    Return,
                ],
            },
        );
        table
    };

    let mut runtime = Runtime::new(vec![]);
    runtime
        .variable_table
        .insert("table", Object::new_table(table_obj));
    vm::execute(
        &[
            LoadLocal("table"),
            LoadFloat(1.23),
            CallMethod("testMethod", 1),
            Exit,
        ],
        &mut runtime,
    )
    .unwrap();

    if let Object::Table(table) = runtime.variable_table.get("table").unwrap() {
        assert_eq!(table.borrow().get("key"), Some(&Object::Float(1.23)));
    } else {
        unreachable!()
    }
}
