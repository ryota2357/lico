use super::*;
use smallvec::SmallVec;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub fn execute(code: &[Code], runtime: &mut Runtime) -> Result<Object, String> {
    use Code::*;

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
                let x = StringObject::new(Rc::clone(x));
                runtime.stack.push(Object::String(x).into());
                pc += 1;
            }
            LoadNil => {
                runtime.stack.push(Object::Nil.into());
                pc += 1;
            }
            LoadLocal(id) => {
                let object = runtime.variable_table.get(*id);
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
            SetLocal(id) => {
                let object = runtime.stack.pop().ensure_object();
                runtime.variable_table.edit(*id, object);
                pc += 1;
            }
            MakeLocal => {
                let object = runtime.stack.pop().ensure_object();
                runtime.variable_table.push(object);
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
            MakeNamed => {
                let name = runtime.stack.pop().ensure_object().ensure_string()?;
                let object = runtime.stack.pop().ensure_object();
                runtime.stack.push((name, object).into());
                pc += 1;
            }
            MakeTable(count) => {
                let mut hash_map = HashMap::with_capacity(*count as usize);
                for _ in 0..*count {
                    let (name, value) = runtime.stack.pop().ensure_named();
                    let name = name.to_string();
                    hash_map.insert(name.into(), value);
                }
                let table = TableObject::new(hash_map);
                runtime.stack.push(Object::new_table(table).into());
                pc += 1;
            }
            DropLocal(count) => {
                runtime.variable_table.drop(*count);
                pc += 1;
            }
            Jump(offset) => {
                if offset.is_positive() {
                    pc += *offset as usize;
                } else {
                    pc -= offset.unsigned_abs();
                }
            }
            JumpIfTrue(offset) => {
                let boolean = runtime.stack.pop().ensure_object().ensure_bool()?;
                if boolean {
                    if offset.is_positive() {
                        pc += *offset as usize;
                    } else {
                        pc -= offset.unsigned_abs();
                    }
                } else {
                    pc += 1;
                }
            }
            JumpIfFalse(offset) => {
                let boolean = runtime.stack.pop().ensure_object().ensure_bool()?;
                if !boolean {
                    if offset.is_positive() {
                        pc += *offset as usize;
                    } else {
                        pc -= offset.unsigned_abs();
                    }
                } else {
                    pc += 1;
                }
            }
            CallMethod(name, args_len) => {
                let res = match args_len {
                    0 => {
                        let self_obj = runtime.stack.pop().ensure_object();
                        code_impl::call_method(self_obj, name, &[], runtime)?
                    }
                    1 => {
                        let arg = runtime.stack.pop().ensure_object();
                        let self_obj = runtime.stack.pop().ensure_object();
                        code_impl::call_method(self_obj, name, &[arg], runtime)?
                    }
                    2 => {
                        let arg2 = runtime.stack.pop().ensure_object();
                        let arg1 = runtime.stack.pop().ensure_object();
                        let self_obj = runtime.stack.pop().ensure_object();
                        code_impl::call_method(self_obj, name, &[arg2, arg1], runtime)?
                    }
                    3 => {
                        let arg3 = runtime.stack.pop().ensure_object();
                        let arg2 = runtime.stack.pop().ensure_object();
                        let arg1 = runtime.stack.pop().ensure_object();
                        let self_obj = runtime.stack.pop().ensure_object();
                        code_impl::call_method(self_obj, name, &[arg3, arg2, arg1], runtime)?
                    }
                    _ => {
                        let mut args = Vec::with_capacity(*args_len as usize);
                        for _ in 0..*args_len {
                            args.push(runtime.stack.pop().ensure_object());
                        }
                        let self_obj = runtime.stack.pop().ensure_object();
                        code_impl::call_method(self_obj, name, &args, runtime)?
                    }
                };
                runtime.stack.push(res.into());
                pc += 1;
            }
            Call(args_len) => {
                let res = match args_len {
                    0 => {
                        let callee = runtime.stack.pop();
                        code_impl::call(callee, &[], runtime)?
                    }
                    1 => {
                        let arg = runtime.stack.pop().ensure_object();
                        let callee = runtime.stack.pop();
                        code_impl::call(callee, &[arg], runtime)?
                    }
                    2 => {
                        let arg2 = runtime.stack.pop().ensure_object();
                        let arg1 = runtime.stack.pop().ensure_object();
                        let callee = runtime.stack.pop();
                        code_impl::call(callee, &[arg2, arg1], runtime)?
                    }
                    3 => {
                        let arg3 = runtime.stack.pop().ensure_object();
                        let arg2 = runtime.stack.pop().ensure_object();
                        let arg1 = runtime.stack.pop().ensure_object();
                        let callee = runtime.stack.pop();
                        code_impl::call(callee, &[arg3, arg2, arg1], runtime)?
                    }
                    _ => {
                        let mut args = Vec::with_capacity(*args_len as usize);
                        for _ in 0..*args_len {
                            args.push(runtime.stack.pop().ensure_object());
                        }
                        let callee = runtime.stack.pop();
                        code_impl::call(callee, &args, runtime)?
                    }
                };
                runtime.stack.push(res.into());
                pc += 1;
            }
            SetItem => {
                let accesser = runtime.stack.pop().ensure_object();
                let target = runtime.stack.pop();
                let value = runtime.stack.pop().ensure_object();
                let res = code_impl::set_item(target, accesser, value)?;
                runtime.stack.push(res);
                pc += 1;
            }
            GetItem => {
                let accesser = runtime.stack.pop().ensure_object();
                let target = runtime.stack.pop();
                let item = code_impl::get_item(target, accesser)?;
                runtime.stack.push(item.into());
                pc += 1;
            }
            Add => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                let res = code_impl::add(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            Sub => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                let res = code_impl::sub(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            Mul => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                let res = code_impl::mul(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            Div => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                let res = code_impl::div(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            Mod => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                let res = code_impl::r#mod(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            Pow => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                match (lhs, rhs) {
                    (Object::Int(_lhs), Object::Int(_rhs)) => {
                        unimplemented!("Int.pow(Int) is not implemented.");
                    }
                    (Object::Int(lhs), Object::Float(rhs)) => {
                        let pow = (lhs as f64).powf(rhs);
                        runtime.stack.push(Object::Float(pow).into());
                    }
                    (Object::Float(lhs), Object::Int(rhs)) => {
                        let pow = if rhs > i32::MAX as i64 {
                            lhs.powf(rhs as f64)
                        } else {
                            lhs.powi(rhs as i32)
                        };
                        runtime.stack.push(Object::Float(pow).into());
                    }
                    (Object::Float(lhs), Object::Float(rhs)) => {
                        let pow = lhs.powf(rhs);
                        runtime.stack.push(Object::Float(pow).into());
                    }
                    (lhs, rhs) => Err(format!(
                        "Expected Int or Float, but got {:?} and {:?}",
                        lhs, rhs
                    ))?,
                }
                pc += 1;
            }
            Unm => {
                let obj = runtime.stack.pop().ensure_object();
                let res = code_impl::unm(obj)?;
                runtime.stack.push(res.into());
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
                let res = code_impl::less(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            LessEq => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                let res = code_impl::less_eq(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            Greater => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                let res = code_impl::greater(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            GreaterEq => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                let res = code_impl::greater_eq(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            Concat => {
                let rhs = runtime.stack.pop().ensure_object();
                let lhs = runtime.stack.pop().ensure_object();
                let res = code_impl::concat(lhs, rhs)?;
                runtime.stack.push(res.into());
                pc += 1;
            }
            Builtin(instr, args_len) => {
                let mut args = SmallVec::<[_; 2]>::with_capacity(*args_len as usize);
                for _ in 0..*args_len {
                    args.push(runtime.stack.pop().ensure_object());
                }
                match instr {
                    BuiltinInstr::Write => {
                        for arg in args.iter().rev() {
                            runtime.stdio.write(format!("{}", arg));
                        }
                    }
                    BuiltinInstr::Flush => {
                        assert!(*args_len == 0, "Builtin::Flush takes no arguments.");
                        runtime.stdio.flush();
                    }
                    BuiltinInstr::WriteError => {
                        for arg in args.iter().rev() {
                            runtime.stdio.write_err(format!("{}", arg));
                        }
                    }
                    BuiltinInstr::FlushError => {
                        assert!(*args_len == 0, "Builtin::FlushError takes no arguments.");
                        runtime.stdio.flush_err();
                    }
                    BuiltinInstr::ReadLine => {
                        assert!(*args_len == 0, "Builtin::ReadLine takes no arguments.");
                        let line = runtime.stdio.read_line();
                        runtime.stack.push(Object::new_string(line).into());
                    }
                    BuiltinInstr::ReadFile => {
                        assert!(*args_len == 1, "Builtin::ReadFile takes 1 argument.");
                        let path = args.into_iter().next().unwrap().ensure_string()?;
                        let content = std::fs::read(path.as_str()).map_err(|e| e.to_string())?;
                        let string = String::from_utf8(content).map_err(|e| e.to_string())?;
                        runtime.stack.push(Object::new_string(string).into());
                    }
                    BuiltinInstr::WriteFile => {
                        assert!(*args_len == 2, "Builtin::WriteFile takes 2 arguments.");
                        let mut args = args.into_iter();
                        let path = args.next().unwrap().ensure_string()?;
                        let content = args.next().unwrap().ensure_string()?;
                        std::fs::write(path.as_str(), content.as_str())
                            .map_err(|e| e.to_string())?;
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
                        let obj = runtime.variable_table.get_ref(name);
                        env.push(obj);
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
                    let mut func_code = Vec::new();
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
                    Object::new_function(FunctionObject {
                        id,
                        env,
                        args,
                        code,
                    })
                    .into(),
                );
                pc += 1;
            }
            AddCapture(_) => panic!("[BUG] AddCapture is not allowed here."),
            AddArgument(_) => panic!("[BUG] AddArgument is not allowed here."),
            EndFuncCreation => panic!("[BUG] EndFuncCreation is not allowed here."),
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

mod shared_proc {
    use super::*;

    pub fn execute_func(
        func: &FunctionObject,
        args: &[Object],
        runtime: &mut Runtime,
    ) -> Result<Object, String> {
        runtime.variable_table.push_scope();
        for value in func.env.iter() {
            runtime.variable_table.push_ref(Rc::clone(value));
        }
        let args_len = func.args.len();
        for (i, _attr) in func.args.iter().enumerate() {
            let value = args.get(args_len - i - 1).cloned().unwrap_or(Object::Nil);
            runtime.variable_table.push(value);
        }
        let ret = execute(&func.code, runtime)?;
        runtime.variable_table.pop_scope();
        Ok(ret)
    }

    pub fn exec_table_method(
        table: Rc<RefCell<TableObject>>,
        name: &str,
        args: &[Object],
        runtime: &mut Runtime,
    ) -> Result<Object, String> {
        let method = table.borrow().get_method(name);
        match method {
            Some(TableMethod::Builtin(func)) => func(table, args),
            Some(TableMethod::Custom(func)) => {
                let args = args
                    .iter()
                    .cloned()
                    .chain(std::iter::once(Object::Table(table)))
                    .collect::<SmallVec<[Object; 3]>>();
                execute_func(&func, &args, runtime)
            }
            Some(TableMethod::CustomNoSelf(func)) => execute_func(&func, args, runtime),
            None => run_table_default_method(table, name, args),
        }
    }
}

mod code_impl {
    use super::*;
    use std::borrow::Cow;

    pub fn call_method(
        self_obj: Object,
        name: &str,
        args: &[Object],
        runtime: &mut Runtime,
    ) -> Result<Object, String> {
        match self_obj {
            Object::Int(int) => run_int_method(int, name, args),
            Object::Float(float) => run_float_method(float, name, args),
            Object::String(string) => run_string_method(string, name, args),
            Object::Bool(boolean) => run_bool_method(boolean, name, args),
            Object::Nil => run_nil_method(name, args),
            Object::Array(array) => run_array_method(array, name, args),
            Object::Table(table) => shared_proc::exec_table_method(table, name, args, runtime),
            Object::Function(_) | Object::RustFunction(_) => {
                Err("Function does not have methods.".to_string())?
            }
        }
    }

    pub fn call(
        callee: StackValue,
        args: &[Object],
        runtime: &mut Runtime,
    ) -> Result<Object, String> {
        match callee {
            StackValue::Object(Object::Function(func)) => {
                shared_proc::execute_func(&func, args, runtime)
            }
            StackValue::Object(Object::Table(table)) => {
                shared_proc::exec_table_method(table, &Cow::from("__call"), args, runtime)
            }
            StackValue::Object(Object::RustFunction(func)) => func(args),
            x => Err(format!("Expected Callable Object, but got {:?}", x))?,
        }
    }

    pub fn set_item(
        target: StackValue,
        accesser: Object,
        value: Object,
    ) -> Result<StackValue, String> {
        // TODO: array bounds check
        let res = match target {
            StackValue::RawArray(mut array) => {
                let index = accesser.ensure_int()?;
                array[index as usize] = value;
                StackValue::RawArray(array)
            }
            StackValue::Object(Object::Array(array)) => {
                let index = accesser.ensure_int()?;
                array.borrow_mut()[index as usize] = value;
                StackValue::Object(Object::Array(array))
            }
            StackValue::Object(Object::Table(table)) => {
                let index = accesser.ensure_string()?;
                if let Some(t) = table.borrow_mut().get_mut(index.as_str()) {
                    *t = value;
                } else {
                    let index = index.to_string();
                    table.borrow_mut().insert(index.into(), value);
                }
                StackValue::Object(Object::Table(table))
            }
            x => Err(format!("Expected Array or Table, but got {:?}", x))?,
        };
        Ok(res)
    }

    pub fn get_item(target: StackValue, accesser: Object) -> Result<Object, String> {
        let res = match target {
            StackValue::RawArray(array) => {
                let index = accesser.ensure_int()?;
                match array.get(index as usize) {
                    Some(x) => x.clone(),
                    None => Object::Nil,
                }
            }
            StackValue::Object(Object::String(string)) => {
                let string = string.get_chars();
                let index = {
                    let i = accesser.ensure_int()?;
                    if i >= 0 {
                        string.len() as i64 + i
                    } else {
                        i
                    }
                };
                match string.get(index as usize) {
                    Some(x) => Object::new_string(x.to_string()),
                    None => Object::Nil,
                }
            }
            StackValue::Object(Object::Array(array)) => {
                let index = accesser.ensure_int()?;
                match array.borrow().get(index as usize) {
                    Some(x) => x.clone(),
                    None => Object::Nil,
                }
            }
            StackValue::Object(Object::Table(table)) => {
                let index = accesser.ensure_string()?;
                match table.borrow().get(index.as_str()) {
                    Some(x) => x.clone(),
                    None => Object::Nil,
                }
            }
            x => Err(format!("Expected Array or Table, but got {:?}", x))?,
        };
        Ok(res)
    }

    pub fn add(lhs: Object, rhs: Object) -> Result<Object, String> {
        // TODO: overflow/underflow check
        let res = match (lhs, rhs) {
            (Object::Int(lhs), Object::Int(rhs)) => Object::Int(lhs + rhs),
            (Object::Int(lhs), Object::Float(rhs)) => Object::Float(lhs as f64 + rhs),
            (Object::Float(lhs), Object::Int(rhs)) => Object::Float(lhs + rhs as f64),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs + rhs),
            (lhs, rhs) => Err(format!(
                "Expected Int or Float, but got {:?} and {:?}",
                lhs, rhs
            ))?,
        };
        Ok(res)
    }

    pub fn sub(lhs: Object, rhs: Object) -> Result<Object, String> {
        // TODO: underflow/overflow check
        let res = match (lhs, rhs) {
            (Object::Int(lhs), Object::Int(rhs)) => Object::Int(lhs - rhs),
            (Object::Int(lhs), Object::Float(rhs)) => Object::Float(lhs as f64 - rhs),
            (Object::Float(lhs), Object::Int(rhs)) => Object::Float(lhs - rhs as f64),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs - rhs),
            (lhs, rhs) => Err(format!(
                "Expected Int or Float, but got {:?} and {:?}",
                lhs, rhs
            ))?,
        };
        Ok(res)
    }

    pub fn mul(lhs: Object, rhs: Object) -> Result<Object, String> {
        // TODO: overflow check
        let res = match (lhs, rhs) {
            (Object::Int(lhs), Object::Int(rhs)) => Object::Int(lhs * rhs),
            (Object::Int(lhs), Object::Float(rhs)) => Object::Float(lhs as f64 * rhs),
            (Object::Float(lhs), Object::Int(rhs)) => Object::Float(lhs * rhs as f64),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs * rhs),
            (lhs, rhs) => Err(format!(
                "Expected Int or Float, but got {:?} and {:?}",
                lhs, rhs
            ))?,
        };
        Ok(res)
    }

    pub fn div(lhs: Object, rhs: Object) -> Result<Object, String> {
        match (lhs, rhs) {
            (Object::Int(lhs), Object::Int(rhs)) => {
                if rhs == 0 {
                    Err("Divided by zero.".to_string())?
                }
                Ok(Object::Int(lhs / rhs))
            }
            (Object::Int(lhs), Object::Float(rhs)) => Ok(Object::Float(lhs as f64 / rhs)),
            (Object::Float(lhs), Object::Int(rhs)) => Ok(Object::Float(lhs / rhs as f64)),
            (Object::Float(lhs), Object::Float(rhs)) => Ok(Object::Float(lhs / rhs)),
            (lhs, rhs) => Err(format!(
                "Expected Int or Float, but got {:?} and {:?}",
                lhs, rhs
            ))?,
        }
    }

    pub fn r#mod(lhs: Object, rhs: Object) -> Result<Object, String> {
        let res = match (lhs, rhs) {
            (Object::Int(lhs), Object::Int(rhs)) => {
                if rhs == 0 {
                    Err("Divided by zero.".to_string())?
                }
                Object::Int(lhs % rhs)
            }
            (Object::Int(lhs), Object::Float(rhs)) => Object::Float(lhs as f64 % rhs),
            (Object::Float(lhs), Object::Int(rhs)) => Object::Float(lhs % rhs as f64),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs % rhs),
            (lhs, rhs) => Err(format!(
                "Expected Int or Float, but got {:?} and {:?}",
                lhs, rhs
            ))?,
        };
        Ok(res)
    }

    pub fn unm(obj: Object) -> Result<Object, String> {
        // TODO: underflow/overflow check
        let res = match obj {
            Object::Int(x) => Object::Int(-x),
            Object::Float(x) => Object::Float(-x),
            x => Err(format!("Expected Int or Float, but got {:?}", x))?,
        };
        Ok(res)
    }

    pub fn less(lhs: Object, rhs: Object) -> Result<Object, String> {
        let boolean = match (lhs, rhs) {
            (Object::Int(lhs), Object::Int(rhs)) => lhs < rhs,
            (Object::Int(lhs), Object::Float(rhs)) => (lhs as f64) < rhs,
            (Object::Float(lhs), Object::Int(rhs)) => lhs < (rhs as f64),
            (Object::Float(lhs), Object::Float(rhs)) => lhs < rhs,
            (lhs, rhs) => Err(format!(
                "Expected Int or Float, but got {:?} and {:?}",
                lhs, rhs
            ))?,
        };
        Ok(Object::Bool(boolean))
    }

    pub fn less_eq(lhs: Object, rhs: Object) -> Result<Object, String> {
        let boolean = match (lhs, rhs) {
            (Object::Int(lhs), Object::Int(rhs)) => lhs <= rhs,
            (Object::Int(lhs), Object::Float(rhs)) => (lhs as f64) <= rhs,
            (Object::Float(lhs), Object::Int(rhs)) => lhs <= (rhs as f64),
            (Object::Float(lhs), Object::Float(rhs)) => lhs <= rhs,
            (lhs, rhs) => Err(format!(
                "Expected Int or Float, but got {:?} and {:?}",
                lhs, rhs
            ))?,
        };
        Ok(Object::Bool(boolean))
    }

    pub fn greater(lhs: Object, rhs: Object) -> Result<Object, String> {
        let boolean = match (lhs, rhs) {
            (Object::Int(lhs), Object::Int(rhs)) => lhs > rhs,
            (Object::Int(lhs), Object::Float(rhs)) => (lhs as f64) > rhs,
            (Object::Float(lhs), Object::Int(rhs)) => lhs > (rhs as f64),
            (Object::Float(lhs), Object::Float(rhs)) => lhs > rhs,
            (lhs, rhs) => Err(format!(
                "Expected Int or Float, but got {:?} and {:?}",
                lhs, rhs
            ))?,
        };
        Ok(Object::Bool(boolean))
    }

    pub fn greater_eq(lhs: Object, rhs: Object) -> Result<Object, String> {
        let boolean = match (lhs, rhs) {
            (Object::Int(lhs), Object::Int(rhs)) => lhs >= rhs,
            (Object::Int(lhs), Object::Float(rhs)) => (lhs as f64) >= rhs,
            (Object::Float(lhs), Object::Int(rhs)) => lhs >= (rhs as f64),
            (Object::Float(lhs), Object::Float(rhs)) => lhs >= rhs,
            (lhs, rhs) => Err(format!(
                "Expected Int or Float, but got {:?} and {:?}",
                lhs, rhs
            ))?,
        };
        Ok(Object::Bool(boolean))
    }

    pub fn concat(lhs: Object, rhs: Object) -> Result<Object, String> {
        // TODO: Improve performance when lhs or rhs is Object::String.
        fn to_string(obj: Object) -> Result<String, String> {
            match obj {
                Object::Int(x) => Ok(x.to_string()),
                Object::Float(x) => Ok(x.to_string()),
                Object::String(x) => Ok(x.to_string()),
                Object::Bool(x) => Ok(if x { "true" } else { "false" }.to_string()),
                Object::Nil => Ok("nil".to_string()),
                x => Err(format!(
                    "Expected String or Stringable Object, but got {:?}",
                    x
                ))?,
            }
        }
        let lhs = to_string(lhs)?;
        let rhs = to_string(rhs)?;
        Ok(Object::new_string(lhs + &rhs))
    }
}
