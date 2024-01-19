use std::rc::Rc;
use vm::{
    code::{ArgumentKind, Code::*, LocalId},
    runtime::{Object, Runtime},
};

#[test]
#[rustfmt::skip]
fn load() {
    let mut runtime = Runtime::new();
    vm::execute(&[
        LoadInt(37), LoadFloat(42.0), LoadBool(true), LoadString(Rc::new("a b".to_string())), LoadString(Rc::new("c".to_string())), LoadNil,
        Exit,
    ], &mut runtime).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Nil);
    assert_eq!(runtime.stack.pop().ensure_object(), Object::new_string("c".to_string()));
    assert_eq!(runtime.stack.pop().ensure_object(), Object::new_string("a b".to_string()));
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Bool(true));
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Float(42.0));
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(37));
}

#[test]
fn load_local() {
    let mut runtime = Runtime::new();
    runtime.variable_table.push(Object::Int(1));
    vm::execute(&[LoadLocal(LocalId(0)), Exit], &mut runtime).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(1));
}

#[test]
fn load_rust_function() {
    let mut runtime = Runtime::new();
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
#[should_panic(expected = "[BUG] Stack must have at least one value at pop.")]
fn unload() {
    let mut runtime = Runtime::new();
    runtime.stack.push(Object::Int(0).into());
    vm::execute(&[UnloadTop, Exit], &mut runtime).unwrap();
    runtime.stack.pop(); // panic
}

#[test]
fn set_local() {
    let mut runtime = Runtime::new();
    runtime.variable_table.push(Object::Int(1));
    runtime.variable_table.push(Object::Bool(false));
    #[rustfmt::skip]
    vm::execute(
        &[
            LoadInt(10), SetLocal(LocalId(0)),
            LoadBool(true), SetLocal(LocalId(1)),
            Exit,
        ],
        &mut runtime,
    )
    .unwrap();
    assert_eq!(runtime.variable_table.get(LocalId(0)), Object::Int(10));
}

#[test]
fn make_local() {
    let mut runtime = Runtime::new();
    vm::execute(&[LoadInt(1), MakeLocal, Exit], &mut runtime).unwrap();
    vm::execute(&[LoadInt(2), MakeLocal, Exit], &mut runtime).unwrap();
    assert_eq!(runtime.variable_table.get(LocalId(0)), Object::Int(1));
    assert_eq!(runtime.variable_table.get(LocalId(1)), Object::Int(2));
}

#[test]
fn make_array() {
    use vm::runtime::StackValue;

    let mut runtime = Runtime::new();
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
    let mut runtime = Runtime::new();
    vm::execute(
        &[
            LoadNil,
            LoadString(Rc::new("NILL".to_string())),
            MakeNamed,
            Exit,
        ],
        &mut runtime,
    )
    .unwrap();
    assert_eq!(
        runtime.stack.pop().ensure_named(),
        (Rc::new("NILL".to_string()), Object::Nil)
    );
}

#[test]
fn make_table() {
    use std::{cell::RefCell, rc::Rc};
    use vm::runtime::TableObject;

    let mut runtime = Runtime::new();
    for (key, value) in [
        ("Key1".to_string(), Object::Int(1)),
        ("Key2".to_string(), Object::Bool(true)),
        ("Key3".to_string(), Object::new_string("a".to_string())),
    ] {
        runtime.stack.push((Rc::new(key), value).into());
    }
    vm::execute(&[MakeTable(2), Exit], &mut runtime).unwrap();

    assert_eq!(
        runtime.stack.pop().ensure_object(),
        Object::Table(Rc::new(RefCell::new(TableObject::new(
            vec![
                ("Key2".into(), Object::Bool(true)),
                ("Key3".into(), Object::new_string("a".to_string())),
            ]
            .into_iter()
            .collect()
        ))))
    );
    assert_eq!(
        runtime.stack.pop().ensure_named(),
        (Rc::new("Key1".to_string()), Object::Int(1))
    );
}

#[test]
#[should_panic(expected = "[BUG] LocalId out of range. Expected 0..1, but got 1.")]
fn drop_local() {
    let mut runtime = Runtime::new();
    runtime.variable_table.push(Object::Int(1));
    runtime.variable_table.push(Object::Int(2));
    runtime.variable_table.push(Object::Int(3));
    assert_eq!(runtime.variable_table.get(LocalId(2)), Object::Int(3));
    vm::execute(&[DropLocal(2), Exit], &mut runtime).unwrap();
    assert_eq!(runtime.variable_table.get(LocalId(0)), Object::Int(1));
    runtime.variable_table.get(LocalId(1)); // panic
}

#[test]
fn jump_forward() {
    let mut runtime = Runtime::new();
    runtime.variable_table.push(Object::Int(1));
    vm::execute(&[Jump(2), DropLocal(1), Exit], &mut runtime).unwrap();
    assert_eq!(runtime.variable_table.get(LocalId(0)), Object::Int(1));
}

#[test]
#[should_panic(expected = "[BUG] LocalId out of range. Expected 0..0, but got 0.")]
fn jump_backward() {
    let mut runtime = Runtime::new();
    runtime.variable_table.push(Object::Int(1));
    vm::execute(&[Jump(3), DropLocal(1), Exit, Jump(-2)], &mut runtime).unwrap();
    runtime.variable_table.get(LocalId(0)); // panic
}

#[test]
#[rustfmt::skip]
fn jump_if_true() {
    let mut runtime = Runtime::new();
    vm::execute(
        &[LoadBool(true), JumpIfTrue(3), Nop, Exit, LoadInt(1), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(1));

    let mut runtime = Runtime::new();
    vm::execute(
        &[LoadBool(false), JumpIfTrue(3), LoadInt(2), Exit, Nop, Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(2));

    let mut runtime = Runtime::new();
    vm::execute(
        &[Jump(3), LoadInt(3), Exit, LoadBool(true), JumpIfTrue(-3), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(3));

    let mut runtime = Runtime::new();
    vm::execute(
        &[Jump(2), Exit, LoadBool(false), JumpIfTrue(-2), LoadInt(4), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(4));
}

#[test]
#[rustfmt::skip]
fn jump_if_false() {
    let mut runtime = Runtime::new();
    vm::execute(
        &[LoadBool(false), JumpIfFalse(3), Nop, Exit, LoadInt(1), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(1));

    let mut runtime = Runtime::new();
    vm::execute(
        &[LoadBool(true), JumpIfFalse(3), LoadInt(2), Exit, Nop, Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(2));

    let mut runtime = Runtime::new();
    vm::execute(
        &[Jump(3), LoadInt(3), Exit, LoadBool(false), JumpIfFalse(-3), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(3));

    let mut runtime = Runtime::new();
    vm::execute(
        &[Jump(2), Exit, LoadBool(true), JumpIfFalse(-2), LoadInt(4), Exit],
        &mut runtime,
    ).unwrap();
    assert_eq!(runtime.stack.pop().ensure_object(), Object::Int(4));
}

#[test]
fn custom_method() {
    use vm::runtime::{FunctionObject, TableObject};

    // var table = {
    //     key = "value"
    //     func testMethod = func(self, new_value)
    //         self.key = new_value
    //     end
    // }
    let table_obj = {
        let mut table = TableObject::new(
            [("key".into(), Object::new_string("value".to_string()))]
                .into_iter()
                .collect(),
        );
        table.add_method(
            "testMethod",
            FunctionObject {
                id: (0, 0),
                env: vec![],
                args: vec![ArgumentKind::Auto, ArgumentKind::Copy], // self, new_value
                code: vec![
                    LoadLocal(LocalId(1)),
                    LoadLocal(LocalId(0)),
                    LoadString(Rc::new("key".to_string())),
                    SetItem,
                    LoadNil,
                    Return,
                ],
            },
        );
        table
    };

    let mut runtime = Runtime::new();
    runtime.variable_table.push(Object::new_table(table_obj));
    vm::execute(
        &[
            LoadLocal(LocalId(0)),
            LoadFloat(1.23),
            CallMethod("testMethod".into(), 1),
            Exit,
        ],
        &mut runtime,
    )
    .unwrap();

    if let Object::Table(table) = runtime.variable_table.get(LocalId(0)) {
        assert_eq!(table.borrow().get("key"), Some(&Object::Float(1.23)));
    } else {
        unreachable!()
    }
}
