pub mod code;
pub mod runtime;

use code::{BuiltinInstr, Code, Code::*};
use runtime::{Object, Runtime, StackValue, TableObject};
use std::{collections::HashMap, rc::Rc};

pub fn execute<'src, W: std::io::Write>(
    code: &[Code<'src>],
    runtime: &mut Runtime<'src, W>,
) -> Result<Object<'src>, String> {
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
                runtime.stack.push(Object::String(x.clone()).into());
                pc += 1;
            }
            LoadStringAsRef(x) => {
                runtime.stack.push(Object::String(x.to_string()).into());
                pc += 1;
            }
            LoadNil => {
                runtime.stack.push(Object::Nil.into());
                pc += 1;
            }
            LoadLocal(name) => {
                let object = match runtime.variable_table.get(name) {
                    Some(x) => x,
                    None => Err(format!("{} is not defined.", name))?,
                };
                runtime.stack.push(object.into());
                pc += 1;
            }
            LoadRustFunction(x) => {
                runtime.stack.push(Object::RustFunction(*x).into());
                pc += 1;
            }
            UnloadTop => {
                runtime.stack.pop();
                pc += 1;
            }
            SetLocal(name) => {
                let object = runtime.stack.pop().ensure_object();
                runtime.variable_table.edit(name, object)?;
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
                array.reverse();
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
                    x => Err(format!("Expected String, but got {:?}", x))?,
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
                    x => Err(format!("Expected Bool, but got {:?}", x))?,
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
                    x => Err(format!("Expected Bool, but got {:?}", x))?,
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
                let args = {
                    let mut args = Vec::with_capacity((args_len + 1) as usize);
                    for _ in 0..*args_len {
                        args.push(runtime.stack.pop().ensure_object());
                    }
                    let self_obj = runtime.stack.pop().ensure_object();
                    args.push(self_obj);
                    args.reverse();
                    args
                };
                let self_obj = args.first().unwrap().clone();
                match self_obj {
                    Object::Array(_) => {
                        let res = runtime::run_array_method(name, &args)?;
                        runtime.stack.push(res.into());
                    }
                    Object::Table(table) => {
                        let method = table.borrow().get_method(name);
                        let res = match method {
                            Some(func) => execute_func(&func, args, runtime)?,
                            None => runtime::run_table_default_method(name, &args)?,
                        };
                        runtime.stack.push(res.into());
                    }
                    x => Err(format!("Expected Table Object, but got {:?}", x))?,
                }
                pc += 1;
            }
            Call(args_len) => {
                let args = create_args_vec(*args_len, runtime);
                let ret = match runtime.stack.pop() {
                    StackValue::RawFunction(func) => execute_func(&func, args, runtime)?,
                    StackValue::RawTable(table) => {
                        if let Some(func) = table.get_method("__call") {
                            execute_func(&func, args, runtime)?
                        } else {
                            Err("__call is not defined.".to_string())?
                        }
                    }
                    StackValue::Object(Object::Function(func)) => {
                        execute_func(&func, args, runtime)?
                    }
                    StackValue::Object(Object::Table(table)) => {
                        if let Some(func) = table.borrow().get_method("__call") {
                            execute_func(&func, args, runtime)?
                        } else {
                            Err("__call is not defined.".to_string())?
                        }
                    }
                    StackValue::Object(Object::RustFunction(func)) => func(&args)?,
                    x => Err(format!("Expected Callable Object, but got {:?}", x))?,
                };
                runtime.stack.push(ret.into());
                pc += 1;
            }
            SetItem => {
                let get_int_index = |accesser| match accesser {
                    StackValue::Object(Object::Int(x)) => Ok(x as usize),
                    x => Err(format!("Expected Int, but got {:?}", x)),
                };
                let get_string_index = |accesser| match accesser {
                    StackValue::Object(Object::String(x)) => Ok(x),
                    x => Err(format!("Expected String, but got {:?}", x)),
                };

                let accesser = runtime.stack.pop();
                let target = runtime.stack.pop();
                let value = runtime.stack.pop().ensure_object();
                match target {
                    StackValue::RawArray(mut array) => {
                        let index = get_int_index(accesser)?;
                        array[index] = value;
                        runtime.stack.push(array.into());
                    }
                    StackValue::RawTable(mut table) => {
                        let index = get_string_index(accesser)?;
                        table.insert(index, value);
                        runtime.stack.push(table.into());
                    }
                    StackValue::Object(Object::Array(array)) => {
                        let index = get_int_index(accesser)?;
                        array.borrow_mut()[index] = value;
                        runtime.stack.push(Object::Array(array).into());
                    }
                    StackValue::Object(Object::Table(table)) => {
                        let index = get_string_index(accesser)?;
                        table.borrow_mut().insert(index, value);
                        runtime.stack.push(Object::Table(table).into());
                    }
                    x => Err(format!("Expected Array or Table, but got {:?}", x))?,
                }
                pc += 1;
            }
            GetItem => {
                let get_int_index = |accesser| match accesser {
                    StackValue::Object(Object::Int(x)) => Ok(x as usize),
                    x => Err(format!("Expected Int, but got {:?}", x)),
                };
                let get_string_index = |accesser| match accesser {
                    StackValue::Object(Object::String(x)) => Ok(x),
                    x => Err(format!("Expected String, but got {:?}", x)),
                };

                let accesser = runtime.stack.pop();
                let target = runtime.stack.pop();
                match target {
                    StackValue::RawArray(array) => {
                        let index = get_int_index(accesser)?;
                        let item = match array.get(index) {
                            Some(x) => x.clone(),
                            None => Object::Nil,
                        };
                        runtime.stack.push(item.into());
                    }
                    StackValue::RawTable(table) => {
                        let index = get_string_index(accesser)?;
                        let item = match table.get(&index) {
                            Some(x) => x.clone(),
                            None => Object::Nil,
                        };
                        runtime.stack.push(item.into());
                    }
                    StackValue::Object(Object::Array(array)) => {
                        let index = get_int_index(accesser)?;
                        let item = match array.borrow().get(index) {
                            Some(x) => x.clone(),
                            None => Object::Nil,
                        };
                        runtime.stack.push(item.into());
                    }
                    StackValue::Object(Object::Table(table)) => {
                        let index = get_string_index(accesser)?;
                        let item = match table.borrow().get(&index) {
                            Some(x) => x.clone(),
                            None => Object::Nil,
                        };
                        runtime.stack.push(item.into());
                    }
                    x => Err(format!("Expected Array or Table, but got {:?}", x))?,
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
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            Sub => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Int(lhs - rhs).into());
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs as f64 - rhs).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Float(lhs - rhs as f64).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs - rhs).into());
                    }
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            Mul => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Int(lhs * rhs).into());
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs as f64 * rhs).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Float(lhs * rhs as f64).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs * rhs).into());
                    }
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            Div => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Int(lhs / rhs).into());
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs as f64 / rhs).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Float(lhs / rhs as f64).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs / rhs).into());
                    }
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            Mod => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Int(lhs % rhs).into());
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs as f64 % rhs).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Float(lhs % rhs as f64).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Float(lhs % rhs).into());
                    }
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            Pow => todo!("Pow"),
            Unm => {
                let obj = runtime.stack.pop().ensure_object();
                match obj {
                    Object::Int(x) => runtime.stack.push(Object::Int(-x).into()),
                    Object::Float(x) => runtime.stack.push(Object::Float(-x).into()),
                    x => Err(format!("Expected Int or Float, but got {:?}", x))?,
                }
                pc += 1;
            }
            Eq => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                runtime.stack.push(Object::Bool(lhs == rhs).into());
                pc += 1;
            }
            NotEq => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                runtime.stack.push(Object::Bool(lhs != rhs).into());
                pc += 1;
            }
            Less => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs < rhs).into());
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Bool((lhs as f64) < rhs).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs < (rhs as f64)).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs < rhs).into());
                    }
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            LessEq => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs <= rhs).into());
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Bool((lhs as f64) <= rhs).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs <= (rhs as f64)).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs <= rhs).into());
                    }
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            Greater => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs > rhs).into());
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Bool((lhs as f64) > rhs).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs > (rhs as f64)).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs > rhs).into());
                    }
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            GreaterEq => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs >= rhs).into());
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Bool((lhs as f64) >= rhs).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs >= (rhs as f64)).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        runtime.stack.push(Object::Bool(lhs >= rhs).into());
                    }
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            Builtin(instr, args_len) => {
                let args = create_args_vec(*args_len, runtime);
                match instr {
                    BuiltinInstr::Write => {
                        for arg in args {
                            write!(runtime.writer, "{}", arg).unwrap();
                        }
                    }
                    BuiltinInstr::Flush => {
                        assert!(*args_len == 0, "Builtin::Flush takes no arguments.");
                        runtime.writer.flush().unwrap();
                    }
                }
                pc += 1;
            }
            BeginFuncCreation => {
                let id = (pc, 0u8);
                pc += 1;
                let args = {
                    let mut args = Vec::new();
                    while let AddArgument(name) = code[pc] {
                        args.push(name);
                        pc += 1;
                    }
                    args
                };
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
            AddCapture(_) => panic!("[INTERNAL] AddCapture is not allowed here."),
            AddArgument(_) => panic!("[INTERNAL] AddArgument is not allowed here."),
            EndFuncCreation => panic!("[INTERNAL] EndFuncCreation is not allowed here."),
            Nop => {
                pc += 1;
            }
            Return => {
                return Ok(runtime.stack.pop().ensure_object());
            }
            Exit => {
                return Ok(Object::Nil);
            }
        }
    }
}

fn execute_func<'a, W: std::io::Write>(
    func: &runtime::FunctionObject<'a>,
    args: Vec<runtime::Object<'a>>,
    runtime: &mut Runtime<'a, W>,
) -> Result<Object<'a>, String> {
    if func.args.len() != args.len() {
        return Err(format!(
            "Expected {} arguments, but got {} arguments.",
            func.args.len(),
            args.len()
        ));
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
    let ret = execute(&func.code, runtime)?;
    runtime.variable_table.pop_scope();
    Ok(ret)
}

fn create_args_vec<'a, W: std::io::Write>(
    args_len: u8,
    runtime: &mut Runtime<'a, W>,
) -> Vec<runtime::Object<'a>> {
    let mut args = Vec::with_capacity(args_len as usize);
    for _ in 0..args_len {
        args.push(runtime.stack.pop().ensure_object());
    }
    args.reverse();
    args
}
