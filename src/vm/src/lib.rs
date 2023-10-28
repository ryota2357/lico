mod code;
mod runtime;

use code::{BuiltinInstr, Code, Code::*};
use runtime::{Object, Runtime, StackValue, TableObject};
use std::{collections::HashMap, rc::Rc};

pub fn execute<'src>(code: &[Code<'src>], runtime: &mut Runtime<'src>) -> Object<'src> {
    let mut pc = 0;

    loop {
        // println!("code: {:?}", code[pc]);
        // runtime.dump();
        // println!();

        match &code[pc] {
            LoadInt(x) => {
                runtime.stack.push(Object::Int(*x).into());
                pc += 1;
            }
            LoadFloat(x) => {
                runtime.stack.push(Object::Float(*x).into());
                pc += 1;
            }
            LoadBool(x) => {
                runtime.stack.push(Object::Bool(*x).into());
                pc += 1;
            }
            LoadString(x) => {
                runtime.stack.push(Object::String(x.to_string()).into());
                pc += 1;
            }
            LoadNil => {
                runtime.stack.push(Object::Nil.into());
                pc += 1;
            }
            LoadLocal(name) => {
                let object = runtime.variable_table.get(name).unwrap();
                runtime.stack.push(object.into());
                pc += 1;
            }
            Unload(count) => {
                for _ in 0..*count {
                    runtime.stack.pop();
                }
                pc += 1;
            }
            SetLocal(name) => {
                let object = runtime.stack.pop().ensure_object();
                runtime.variable_table.edit(name, object).unwrap();
                pc += 1;
            }
            MakeLocal(name) => {
                let object = runtime.stack.pop().ensure_object();
                runtime.variable_table.insert(name, object);
                pc += 1;
            }
            MakeArray(count) => {
                let mut array = Vec::with_capacity(*count as usize);
                for _ in 0..*count {
                    array.push(runtime.stack.pop().ensure_object());
                }
                runtime.stack.push(array.into());
                pc += 1;
            }
            MakeNamed(name) => {
                let value = runtime.stack.pop().ensure_object();
                runtime.stack.push((name.to_string(), value).into());
                pc += 1;
            }
            MakeExprNamed => {
                let name = match runtime.stack.pop().ensure_object() {
                    Object::String(x) => x,
                    x => panic!("Expected String, but got {:?}", x),
                };
                let object = runtime.stack.pop().ensure_object();
                runtime.stack.push((name, object).into());
                pc += 1;
            }
            MakeTable(count) => {
                let mut table = HashMap::with_capacity(*count as usize);
                for _ in 0..*count {
                    let (name, value) = runtime.stack.pop().ensure_named();
                    table.insert(name, value);
                }
                runtime.stack.push(TableObject::new(table).into());
                pc += 1;
            }
            DropLocal(count) => {
                runtime.variable_table.erase(*count);
                pc += 1;
            }
            Jump(offset) => {
                if *offset < 0 {
                    pc -= offset.unsigned_abs();
                } else {
                    pc += *offset as usize;
                }
            }
            JumpIfTrue(offset) => {
                let boolean = match runtime.stack.pop().ensure_object() {
                    Object::Bool(x) => x,
                    x => panic!("Expected Bool, but got {:?}", x),
                };
                if boolean {
                    if *offset < 0 {
                        pc -= offset.unsigned_abs();
                    } else {
                        pc += *offset as usize;
                    }
                } else {
                    pc += 1;
                }
            }
            JumpIfFalse(offset) => {
                let boolean = match runtime.stack.pop().ensure_object() {
                    Object::Bool(x) => x,
                    x => panic!("Expected Bool, but got {:?}", x),
                };
                if !boolean {
                    if *offset < 0 {
                        pc -= offset.unsigned_abs();
                    } else {
                        pc += *offset as usize;
                    }
                } else {
                    pc += 1;
                }
            }
            CustomMethod(name, args_len) => {
                let args = create_args_vec(*args_len, runtime);
                match runtime.stack.pop() {
                    StackValue::RawTable(table) => {
                        if let Some(func) = table.get_method(name) {
                            execute_func(func, args, runtime);
                        } else {
                            panic!("{} is not defined.", name);
                        }
                    }
                    StackValue::Object(Object::Table(table)) => {
                        let table = table.borrow();
                        if let Some(func) = table.get_method(name) {
                            execute_func(func, args, runtime);
                        } else {
                            panic!("{} is not defined.", name);
                        }
                    }
                    x => panic!("Expected Callable Object, but got {:?}", x),
                }
                pc += 1;
            }
            Call(args_len) => {
                let args = create_args_vec(*args_len, runtime);
                match runtime.stack.pop() {
                    StackValue::RawFunction(func) => {
                        execute_func(&func, args, runtime);
                    }
                    StackValue::RawTable(table) => {
                        if let Some(func) = table.get_method("__call") {
                            execute_func(func, args, runtime);
                        } else {
                            panic!("__call is not defined.");
                        }
                    }
                    StackValue::Object(Object::Function(func)) => {
                        let func = std::borrow::Borrow::borrow(&func);
                        execute_func(func, args, runtime);
                    }
                    StackValue::Object(Object::Table(table)) => {
                        let table = table.borrow();
                        if let Some(func) = table.get_method("__call") {
                            execute_func(func, args, runtime);
                        } else {
                            panic!("__call is not defined.");
                        }
                    }
                    x => panic!("Expected Callable Object, but got {:?}", x),
                }
                pc += 1;
            }
            SetItem => {
                let get_int_index = |accesser| match accesser {
                    StackValue::Object(Object::Int(x)) => x as usize,
                    x => panic!("Expected Int, but got {:?}", x),
                };
                let get_string_index = |accesser| match accesser {
                    StackValue::Object(Object::String(x)) => x,
                    x => panic!("Expected String, but got {:?}", x),
                };

                let accesser = runtime.stack.pop();
                let target = runtime.stack.pop();
                let value = runtime.stack.pop().ensure_object();
                match target {
                    StackValue::RawArray(mut array) => {
                        let index = get_int_index(accesser);
                        array[index] = value;
                        runtime.stack.push(array.into());
                    }
                    StackValue::RawTable(mut table) => {
                        let index = get_string_index(accesser);
                        table.insert(index, value);
                        runtime.stack.push(table.into());
                    }
                    StackValue::Object(Object::Array(array)) => {
                        let index = get_int_index(accesser);
                        array.borrow_mut()[index] = value;
                        runtime.stack.push(Object::Array(array).into());
                    }
                    StackValue::Object(Object::Table(table)) => {
                        let index = get_string_index(accesser);
                        table.borrow_mut().insert(index, value);
                        runtime.stack.push(Object::Table(table).into());
                    }
                    x => panic!("Expected Array or Table, but got {:?}", x),
                }
                pc += 1;
            }
            GetItem => {
                let get_int_index = |accesser| match accesser {
                    StackValue::Object(Object::Int(x)) => x as usize,
                    x => panic!("Expected Int, but got {:?}", x),
                };
                let get_string_index = |accesser| match accesser {
                    StackValue::Object(Object::String(x)) => x,
                    x => panic!("Expected String, but got {:?}", x),
                };

                let accesser = runtime.stack.pop();
                let target = runtime.stack.pop();
                match target {
                    StackValue::RawArray(array) => {
                        let index = get_int_index(accesser);
                        let item = match array.get(index) {
                            Some(x) => x.clone(),
                            None => Object::Nil,
                        };
                        runtime.stack.push(item.into());
                    }
                    StackValue::RawTable(table) => {
                        let index = get_string_index(accesser);
                        let item = match table.get(&index) {
                            Some(x) => x.clone(),
                            None => Object::Nil,
                        };
                        runtime.stack.push(item.into());
                    }
                    StackValue::Object(Object::Array(array)) => {
                        let index = get_int_index(accesser);
                        let item = match array.borrow().get(index) {
                            Some(x) => x.clone(),
                            None => Object::Nil,
                        };
                        runtime.stack.push(item.into());
                    }
                    StackValue::Object(Object::Table(table)) => {
                        let index = get_string_index(accesser);
                        let item = match table.borrow().get(&index) {
                            Some(x) => x.clone(),
                            None => Object::Nil,
                        };
                        runtime.stack.push(item.into());
                    }
                    x => panic!("Expected Array or Table, but got {:?}", x),
                }
                pc += 1;
            }
            Add => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Int(lhs + rhs).into());
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs as f64 + rhs).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Float(lhs + rhs as f64).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs + rhs).into());
                    }
                    (lhs, rhs) => {
                        panic!("Expected Int or Float, but got {:?} and {:?}", lhs, rhs)
                    }
                }
                pc += 1;
            }
            Builtin(instr, args_len) => {
                let args = create_args_vec(*args_len, runtime);
                match instr {
                    BuiltinInstr::Print => {
                        if args.len() != 1 {
                            panic!("Expected 1 argument, but got {} arguments.", args.len());
                        }
                        println!("{}", args[0]);
                    }
                }
                pc += 1;
            }
            BeginFuncCreation => {
                let id = (pc, 0u8);
                pc += 1;
                let env = {
                    let mut env = Vec::new();
                    while let AddCapture(name) = code[pc] {
                        match runtime.variable_table.get_ref(name) {
                            Some(obj) => env.push((name, Some(obj))),
                            None => env.push((name, None)),
                        }
                        pc += 1;
                    }
                    env
                };
                let args = {
                    let mut args = Vec::new();
                    while let AddArgument(name) = code[pc] {
                        args.push(name);
                        pc += 1;
                    }
                    args
                };
                let code = {
                    let mut func_code = Vec::<Code>::new();
                    let mut inner_count = 0;
                    loop {
                        if let BeginFuncCreation = code[pc] {
                            inner_count += 1;
                        } else if let EndFuncCreation = code[pc] {
                            inner_count -= 1;
                        }
                        if inner_count < 0 {
                            break;
                        }
                        func_code.push(code[pc].clone());
                        pc += 1;
                    }
                    func_code
                };
                runtime.stack.push(
                    Object::new_function(runtime::FunctionObject {
                        id,
                        env,
                        args,
                        code,
                    })
                    .into(),
                );
                pc += 1;
            }
            AddCapture(_) => panic!("AddCapture is not allowed here."),
            AddArgument(_) => panic!("AddArgument is not allowed here."),
            EndFuncCreation => panic!("EndFuncCreation is not allowed here."),
            Nop => {
                pc += 1;
            }
            Return => {
                return runtime.stack.pop().ensure_object();
            }

            #[cfg(test)]
            Exit => {
                return Object::Nil;
            }
        }
    }
}

fn execute_func<'a>(
    func: &runtime::FunctionObject<'a>,
    args: Vec<runtime::Object<'a>>,
    runtime: &mut Runtime<'a>,
) {
    if func.args.len() != args.len() {
        panic!(
            "Expected {} arguments, but got {} arguments.",
            func.args.len(),
            args.len()
        );
    }
    runtime.variable_table.push_scope();
    for (name, value) in &func.env {
        match value {
            Some(value) => runtime.variable_table.insert_ref(name, Rc::clone(value)),
            None => runtime.variable_table.insert(name, Object::Nil),
        };
    }
    for (name, value) in func.args.iter().zip(args.iter()) {
        runtime.variable_table.insert(name, value.clone());
    }
    let ret = execute(&func.code, runtime);
    runtime.stack.push(ret.into());
    runtime.variable_table.pop_scope();
}

fn create_args_vec<'a>(args_len: u8, runtime: &mut Runtime<'a>) -> Vec<runtime::Object<'a>> {
    let mut args = Vec::with_capacity(args_len as usize);
    for _ in 0..args_len {
        args.push(runtime.stack.pop().ensure_object());
    }
    args.reverse();
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case1() {
        // var a = 1
        let mut runtime = Runtime::new();
        execute(&[LoadInt(1), MakeLocal("a"), Exit], &mut runtime);
        assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(1)));
    }

    #[test]
    fn case2() {
        // a = 10
        let mut runtime = Runtime::new();
        runtime.variable_table.insert("a", Object::Int(1));
        execute(&[LoadInt(10), SetLocal("a"), Exit], &mut runtime);
        assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(10)));
    }

    #[test]
    #[rustfmt::skip]
    fn case3() {
        // var a = 1
        // vaf f = func() a = a + 10 end
        // f()
        let mut runtime = Runtime::new();
        execute(
            &[
                LoadInt(1), MakeLocal("a"),

                BeginFuncCreation,
                  AddCapture("a"),
                  LoadLocal("a"), LoadInt(10), Add, SetLocal("a"),
                  LoadNil, Return,
                EndFuncCreation,
                MakeLocal("f"),

                LoadLocal("f"), Call(0), Unload(1),
                Exit,
            ],
            &mut runtime,
        );
        assert_eq!(runtime.variable_table.get("a"), Some(Object::Int(11)));
        match runtime.variable_table.get("f").unwrap() {
            Object::Function(func) => {
                let func: &runtime::FunctionObject = std::borrow::Borrow::borrow(&func);
                assert_eq!(func.id, (2, 0));
            }
            _ => panic!("Expected Function, but got {:?}", runtime.variable_table.get("f")),
        };
    }

    #[test]
    #[rustfmt::skip]
    fn case4() {
        // var f = func(x)
        //     return x
        // end
        // var ch = func()
        //    f = func(x)
        //        return x + 100
        //    end
        //    return 10
        // end
        // return f(ch()) + f(1)
        let mut runtime = Runtime::new();
        runtime.variable_table.insert(
            "f",
            Object::new_function(runtime::FunctionObject {
                id: (0, 0),
                env: vec![],
                args: vec!["x"],
                code: vec![LoadLocal("x"), Return],
            }),
        );
        let res = execute(
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
}