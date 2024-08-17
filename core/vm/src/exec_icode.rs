use super::*;
use core::cmp::Ordering;
use foundation::object::{self, Object::*};
use std::rc::Rc;

/// `(pc, exe, runtime)`
type LoopContextRef<'a> = (&'a mut usize, &'a Executable, &'a mut Runtime);

pub(super) fn call<I>(callee: Object, args: I, context: LoopContextRef) -> Status
where
    I: IntoIterator<Item = Object> + 'static,
    I::IntoIter: ExactSizeIterator,
{
    fn set_not_callable_exception(pc: usize, type_name: &str) {
        let message = format!("The object of type '{}' is not callable.", type_name);
        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
    }

    match callee {
        Object::Function(func) => util::exec_function(func, args, context),
        RustFunction(func) => util::exec_rust_function(func, args, context),
        Table(table) => {
            if let Some(method) = table.get_method("__call") {
                util::exec_table_method(method.clone(), args, context)
            } else {
                let pc = *context.0;
                set_not_callable_exception(pc, "table");
                EXCEPTION
            }
        }
        _ => {
            let pc = *context.0;
            set_not_callable_exception(pc, callee.type_name());
            EXCEPTION
        }
    }
}

pub(super) fn call_method<I>(
    receiver: Object,
    name: &UString,
    args: I,
    context: LoopContextRef,
) -> Status
where
    I: IntoIterator<Item = Object> + 'static,
    I::IntoIter: ExactSizeIterator,
{
    fn set_method_not_found_exception(pc: usize, name: &str, type_name: &str) {
        let message = format!(
            "The method '{}' is not found in the object of type '{}'.",
            name, type_name
        );
        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
    }

    fn set_method_argument_length_exception(expected: u8, got: u8, pc: usize) {
        let message = format!(
            "Method call failed: expected {} arguments, got {}.",
            expected, got
        );
        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 1);
    }

    let name = name.as_str();
    let args = args.into_iter();

    use builtin::{
        array::run_method as run_array_method, bool::run_method as run_bool_method,
        float::run_method as run_float_method, function::run_method as run_function_method,
        int::run_method as run_int_method, nil::run_method as run_nil_method,
        rust_function::run_method as run_rust_function_method,
        string::run_method as run_string_method, table::run_method as run_table_method,
        RunMethodResult,
    };

    let result = match receiver {
        Int(int) => run_int_method(name, int, args),
        Float(float) => run_float_method(name, float, args),
        Bool(bool) => run_bool_method(name, bool, args),
        Nil => run_nil_method(name, args),
        String(string) => run_string_method(name, string, args),
        Object::Array(array) => run_array_method(name, array, args),
        Object::Table(ref table) => {
            if let Some(method) = table.get_method(name).cloned() {
                // `iter::once(receiver).chain(args)` is not ExactSizeIterator. (overflow can occur)
                // However, in this case, because the length of arguemnt is limited to u8::MAX by
                // ICode::CallMethod, ExactSizeIterator can be implemented.
                debug_assert!(args.len() <= u8::MAX as usize);
                struct ArgIter<I: Iterator<Item = Object>> {
                    len: usize,
                    iter: I,
                }
                impl<I: Iterator<Item = Object>> Iterator for ArgIter<I> {
                    type Item = Object;
                    fn next(&mut self) -> Option<Self::Item> {
                        self.iter.next()
                    }
                }
                impl<I: Iterator<Item = Object>> ExactSizeIterator for ArgIter<I> {
                    fn len(&self) -> usize {
                        self.len
                    }
                }
                let args = ArgIter {
                    len: args.len() + 1,
                    iter: iter::once(receiver).chain(args),
                };
                return util::exec_table_method(method.clone(), args, context);
            } else {
                run_table_method(name, table.clone(), args)
            }
        }
        Object::Function(func) => run_function_method(name, func, args),
        RustFunction(func) => run_rust_function_method(name, func, args),
    };
    let (pc, _, runtime) = context;
    match result {
        RunMethodResult::Ok(result) => {
            runtime.stack.push(result);
            *pc += 1;
            CONTINUE
        }
        RunMethodResult::NotFound { receiver_type } => {
            set_method_not_found_exception(*pc, name, &receiver_type.to_string());
            EXCEPTION
        }
        RunMethodResult::InvalidArgCount { expected, got } => {
            set_method_argument_length_exception(expected, got, *pc);
            EXCEPTION
        }
        RunMethodResult::InvalidArgType {
            index,
            expected,
            got,
        } => {
            let message = format!(
                "Method call failed: expected argument of type '{}', got '{}'.",
                expected, got
            );
            let extra = index as usize + 2;
            EXCEPTION_LOG.lock().unwrap().push_raw(message, *pc, extra);
            EXCEPTION
        }
        RunMethodResult::ExceptionOccurred => {
            let message = format!("An exception occurred while calling the method '{}'.", name);
            EXCEPTION_LOG.lock().unwrap().push_raw(message, *pc, 1);
            EXCEPTION
        }
    }
}

pub(super) fn set_item(
    container: Object,
    key: Object,
    value: Object,
    context: LoopContextRef,
) -> Status {
    let (pc, _, _) = context;
    match (container, key) {
        (Table(mut table), String(key)) => {
            table.insert(key, value);
            *pc += 1;
            CONTINUE
        }
        (Table(_), key) => {
            util::set_container_key_type_exception("table", key.type_name(), *pc);
            EXCEPTION
        }
        (Array(mut array), Int(index)) => {
            let fixed_index = match util::ensure_array_index(&array, index, *pc) {
                Some(i) => i,
                None => return EXCEPTION,
            };
            array.set(fixed_index as usize, value);
            *pc += 1;
            CONTINUE
        }
        (Array(_), key) => {
            util::set_container_key_type_exception("array", key.type_name(), *pc);
            EXCEPTION
        }
        (container, _) => {
            util::set_not_indexable_exception(container.type_name(), *pc);
            EXCEPTION
        }
    }
}

pub(super) fn get_item(container: Object, key: Object, context: LoopContextRef) -> Status {
    let (pc, _, runtime) = context;
    let result = match (container, key) {
        (Table(table), String(key)) => table.get(&key).cloned().unwrap_or(Nil),
        (Table(_), key) => {
            util::set_container_key_type_exception("table", key.type_name(), *pc);
            return EXCEPTION;
        }
        (Array(array), Int(index)) => {
            let fixed_index = match util::ensure_array_index(&array, index, *pc) {
                Some(i) => i,
                None => return EXCEPTION,
            };
            array.get(fixed_index as usize).cloned().unwrap_or(Nil)
        }
        (Array(_), key) => {
            util::set_container_key_type_exception("array", key.type_name(), *pc);
            return EXCEPTION;
        }
        (container, _) => {
            util::set_not_indexable_exception(container.type_name(), *pc);
            return EXCEPTION;
        }
    };
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn add(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    let result = match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => Int(lhs + rhs),
        (Int(lhs), Float(rhs)) => Float(lhs as f64 + rhs),
        (Float(lhs), Int(rhs)) => Float(lhs + rhs as f64),
        (Float(lhs), Float(rhs)) => Float(lhs + rhs),
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__add", &lhs, &rhs) {
                return util::exec_table_method(method, [lhs, rhs], context);
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception("+", &lhs, &rhs, *pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn sub(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    let result = match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => Int(lhs - rhs),
        (Int(lhs), Float(rhs)) => Float(lhs as f64 - rhs),
        (Float(lhs), Int(rhs)) => Float(lhs - rhs as f64),
        (Float(lhs), Float(rhs)) => Float(lhs - rhs),
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__sub", &lhs, &rhs) {
                return util::exec_table_method(method, [lhs, rhs], context);
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception("-", &lhs, &rhs, *pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn mul(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    let result = match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => Int(lhs * rhs),
        (Int(lhs), Float(rhs)) => Float(lhs as f64 * rhs),
        (Float(lhs), Int(rhs)) => Float(lhs * rhs as f64),
        (Float(lhs), Float(rhs)) => Float(lhs * rhs),
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__mul", &lhs, &rhs) {
                return util::exec_table_method(method, [lhs, rhs], context);
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception("*", &lhs, &rhs, *pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn div(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    let result = match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => {
            if rhs == 0 {
                let message = "Division by zero.".to_string();
                let pc = *context.0;
                EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
                return EXCEPTION;
            } else {
                Int(lhs / rhs)
            }
        }
        (Int(lhs), Float(rhs)) => Float(lhs as f64 / rhs),
        (Float(lhs), Int(rhs)) => Float(lhs / rhs as f64),
        (Float(lhs), Float(rhs)) => Float(lhs / rhs),
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__div", &lhs, &rhs) {
                return util::exec_table_method(method, [lhs, rhs], context);
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception("/", &lhs, &rhs, *pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn r#mod(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    let result = match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => {
            if rhs == 0 {
                let message = "Division by zero.".to_string();
                let pc = *context.0;
                EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
                return EXCEPTION;
            } else {
                Int(lhs % rhs)
            }
        }
        (Int(lhs), Float(rhs)) => Float(lhs as f64 % rhs),
        (Float(lhs), Int(rhs)) => Float(lhs % rhs as f64),
        (Float(lhs), Float(rhs)) => Float(lhs % rhs),
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__mod", &lhs, &rhs) {
                return util::exec_table_method(method, [lhs, rhs], context);
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception("%", &lhs, &rhs, *pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn unm(value: Object, context: LoopContextRef) -> Status {
    let result = match value {
        Int(value) => Int(-value),
        Float(value) => Float(-value),
        value => {
            if let Some(method) = util::find_unary_method("__unm", &value) {
                return util::exec_table_method(method, [value], context);
            } else {
                let (pc, _, _) = context;
                util::set_unary_type_exception("-", &value, *pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn unp(value: Object, context: LoopContextRef) -> Status {
    let result = match value {
        Int(value) => Int(value),
        Float(value) => Float(value),
        value => {
            if let Some(method) = util::find_unary_method("__unp", &value) {
                return util::exec_table_method(method, [value], context);
            } else {
                let (pc, _, _) = context;
                util::set_unary_type_exception("+", &value, *pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn not(value: Object, context: LoopContextRef) -> Status {
    if let Some(method) = util::find_unary_method("__not", &value) {
        util::exec_table_method(method, [value], context)
    } else {
        let (pc, _, runtime) = context;
        let result = Bool(value.is_falsey());
        runtime.stack.push(result);
        *pc += 1;
        CONTINUE
    }
}

pub(super) fn eq(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    if let Some(method) = util::find_binary_method("__eq", &lhs, &rhs) {
        util::exec_table_method(method, [lhs, rhs], context)
    } else {
        let result = lhs == rhs;
        let (pc, _, runtime) = context;
        runtime.stack.push(Bool(result));
        *pc += 1;
        CONTINUE
    }
}

pub(super) fn not_eq(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    if let Some(method) = util::find_binary_method("__ne", &lhs, &rhs) {
        util::exec_table_method(method, [lhs, rhs], context)
    } else if let Some(method) = util::find_binary_method("__eq", &lhs, &rhs) {
        util::exec_table_method_with_post_exec(method, [lhs, rhs], context, |obj| {
            Ok(Bool(obj.is_falsey()))
        })
    } else {
        let result = lhs != rhs;
        let (pc, _, runtime) = context;
        runtime.stack.push(Bool(result));
        *pc += 1;
        CONTINUE
    }
}

pub(super) fn less(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    partial_cmp_with("<", lhs, rhs, context, |ordering| {
        matches!(ordering, Some(Ordering::Less))
    })
}

pub(super) fn less_eq(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    partial_cmp_with("<=", lhs, rhs, context, |ordering| {
        matches!(ordering, Some(Ordering::Less | Ordering::Equal))
    })
}

pub(super) fn greater(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    partial_cmp_with(">", lhs, rhs, context, |ordering| {
        matches!(ordering, Some(Ordering::Greater))
    })
}

pub(super) fn greater_eq(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    partial_cmp_with(">=", lhs, rhs, context, |ordering| {
        matches!(ordering, Some(Ordering::Greater | Ordering::Equal))
    })
}

fn partial_cmp_with(
    op: &'static str,
    lhs: Object,
    rhs: Object,
    context: LoopContextRef,
    f: fn(Option<Ordering>) -> bool,
) -> Status {
    let result = match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => lhs.partial_cmp(&rhs),
        (Int(lhs), Float(rhs)) => (lhs as f64).partial_cmp(&rhs),
        (Float(lhs), Int(rhs)) => lhs.partial_cmp(&(rhs as f64)),
        (Float(lhs), Float(rhs)) => lhs.partial_cmp(&rhs),
        (lhs, rhs) => {
            let pc = *context.0;
            if let Some(method) = util::find_binary_method("__cmp", &lhs, &rhs) {
                return util::exec_table_method_with_post_exec(
                    method,
                    [lhs, rhs],
                    context,
                    move |obj| {
                        let ordering = match obj {
                            Int(value) => value.partial_cmp(&0),
                            Float(value) => value.partial_cmp(&0.0),
                            Nil => None,
                            obj => {
                                let message = format!("The type of result of __cmp method must be int, float or nil, but got '{}'.", obj.type_name());
                                EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
                                return Err(());
                            }
                        };
                        Ok(Bool(f(ordering)))
                    },
                );
            } else {
                util::set_binary_type_exception(op, &lhs, &rhs, pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    let result = f(result);
    runtime.stack.push(Bool(result));
    *pc += 1;
    CONTINUE
}

pub(super) fn concat(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    let result = match (lhs, rhs) {
        (String(lhs), String(rhs)) => String(lhs + rhs),
        (String(lhs), rhs) => String(lhs + rhs.to_string().as_str()),
        (lhs, String(rhs)) => {
            let lhs = UString::from(lhs.to_string().as_str());
            String(lhs + rhs)
        }
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__concat", &lhs, &rhs) {
                return util::exec_table_method(method, [lhs, rhs], context);
            }
            let lhs = UString::from(lhs.to_string().as_str());
            let rhs = UString::from(rhs.to_string().as_str());
            String(lhs + rhs)
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn bit_and(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => {
            let (pc, _, runtime) = context;
            let result = Int(lhs & rhs);
            runtime.stack.push(result);
            *pc += 1;
            CONTINUE
        }
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__band", &lhs, &rhs) {
                util::exec_table_method(method, [lhs, rhs], context)
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception("&", &lhs, &rhs, *pc);
                EXCEPTION
            }
        }
    }
}

pub(super) fn bit_or(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => {
            let (pc, _, runtime) = context;
            let result = Int(lhs | rhs);
            runtime.stack.push(result);
            *pc += 1;
            CONTINUE
        }
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__bor", &lhs, &rhs) {
                util::exec_table_method(method, [lhs, rhs], context)
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception("|", &lhs, &rhs, *pc);
                EXCEPTION
            }
        }
    }
}

pub(super) fn bit_xor(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => {
            let (pc, _, runtime) = context;
            let result = Int(lhs ^ rhs);
            runtime.stack.push(result);
            *pc += 1;
            CONTINUE
        }
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__bxor", &lhs, &rhs) {
                util::exec_table_method(method, [lhs, rhs], context)
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception("^", &lhs, &rhs, *pc);
                EXCEPTION
            }
        }
    }
}

pub(super) fn bit_not(value: Object, context: LoopContextRef) -> Status {
    match value {
        Int(value) => {
            let (pc, _, runtime) = context;
            let result = Int(!value);
            runtime.stack.push(result);
            *pc += 1;
            CONTINUE
        }
        value => {
            if let Some(method) = util::find_unary_method("__bnot", &value) {
                util::exec_table_method(method, [value], context)
            } else {
                let (pc, _, _) = context;
                util::set_unary_type_exception("~", &value, *pc);
                EXCEPTION
            }
        }
    }
}

pub(super) fn shift_l(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    let result = match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => Int(lhs << rhs),
        (Int(lhs), Float(rhs)) => Int(lhs << rhs as i64),
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__shl", &lhs, &rhs) {
                return util::exec_table_method(method, [lhs, rhs], context);
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception("<<", &lhs, &rhs, *pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn shift_r(lhs: Object, rhs: Object, context: LoopContextRef) -> Status {
    let result = match (lhs, rhs) {
        (Int(lhs), Int(rhs)) => Int(lhs >> rhs),
        (Int(lhs), Float(rhs)) => Int(lhs >> rhs as i64),
        (lhs, rhs) => {
            if let Some(method) = util::find_binary_method("__shr", &lhs, &rhs) {
                return util::exec_table_method(method, [lhs, rhs], context);
            } else {
                let (pc, _, _) = context;
                util::set_binary_type_exception(">>", &lhs, &rhs, *pc);
                return EXCEPTION;
            }
        }
    };
    let (pc, _, runtime) = context;
    runtime.stack.push(result);
    *pc += 1;
    CONTINUE
}

pub(super) fn get_iter(value: Object, context: LoopContextRef) -> Status {
    if let Some(method) = util::find_unary_method("__iter", &value) {
        util::exec_table_method(method, [value], context)
    } else {
        todo!("define TextRange in GetIter (compiler/src/compile/icodesource.rs), set error to EXCEPTION_LOG");
        // EXCEPTION
    }
}

pub(super) fn iter_move_next(iter: Object, context: LoopContextRef) -> Status {
    if let Some(method) = util::find_unary_method("__move_next", &iter) {
        util::exec_table_method(method, [iter], context)
    } else {
        todo!("define TextRange in IterMoveNext (compiler/src/compile/icodesource.rs), set error to EXCEPTION_LOG");
        // EXCEPTION
    }
}

pub(super) fn iter_current(iter: Object, context: LoopContextRef) -> Status {
    if let Some(method) = util::find_unary_method("__current", &iter) {
        util::exec_table_method(method, [iter], context)
    } else {
        todo!("define TextRange in IterCurrent (compiler/src/compile/icodesource.rs), set error to EXCEPTION_LOG");
        // EXCEPTION
    }
}

mod util {
    use super::*;

    pub(super) fn find_binary_method(
        name: &'static str,
        lhs: &Object,
        rhs: &Object,
    ) -> Option<TableMethod> {
        if let Table(ref tbl) = lhs {
            if let Some(method) = tbl.get_method(name) {
                return Some(method.clone());
            }
        }
        if let Table(ref tbl) = rhs {
            if let Some(method) = tbl.get_method(name) {
                return Some(method.clone());
            }
        }
        None
    }

    pub(super) fn find_unary_method(name: &'static str, value: &Object) -> Option<TableMethod> {
        if let Table(ref tbl) = value {
            if let Some(method) = tbl.get_method(name) {
                return Some(method.clone());
            }
        }
        None
    }

    pub(super) fn exec_table_method<I>(
        method: TableMethod,
        args: I,
        context: LoopContextRef,
    ) -> Status
    where
        I: IntoIterator<Item = Object> + 'static,
        I::IntoIter: ExactSizeIterator,
    {
        match method {
            TableMethod::Native(func) => exec_rust_function(func, args, context),
            TableMethod::Custom(func) => exec_function(func, args, context),
        }
    }

    pub(super) fn exec_table_method_with_post_exec<I>(
        method: TableMethod,
        args: I,
        context: LoopContextRef,
        post_exec: impl FnOnce(Object) -> Result<Object, ()> + 'static,
    ) -> Status
    where
        I: IntoIterator<Item = Object> + 'static,
        I::IntoIter: ExactSizeIterator,
    {
        match method {
            TableMethod::Native(func) => {
                exec_rust_function_with_post_exec(func, args, context, post_exec)
            }
            TableMethod::Custom(func) => {
                exec_function_with_post_exec(func, args, context, post_exec)
            }
        }
    }

    pub(super) fn exec_function<I>(
        func: object::Function,
        args: I,
        context: LoopContextRef,
    ) -> Status
    where
        I: IntoIterator<Item = Object>,
        I::IntoIter: ExactSizeIterator,
    {
        exec_function_with_core(func, args.into_iter(), context, None)
    }

    pub(super) fn exec_function_with_post_exec<I>(
        func: object::Function,
        args: I,
        context: LoopContextRef,
        post_exec: impl FnOnce(Object) -> Result<Object, ()> + 'static,
    ) -> Status
    where
        I: IntoIterator<Item = Object>,
        I::IntoIter: ExactSizeIterator,
    {
        exec_function_with_core(func, args.into_iter(), context, Some(Box::new(post_exec)))
    }

    #[allow(clippy::type_complexity)]
    fn exec_function_with_core(
        func: object::Function,
        args: impl ExactSizeIterator<Item = Object>,
        context: LoopContextRef,
        post_exec: Option<Box<dyn FnOnce(Object) -> Result<Object, ()>>>,
    ) -> Status {
        let (pc, exe, runtime) = context;

        if func.param_len() != args.len() as u8 {
            set_function_argument_length_exception(func.param_len(), args.len(), *pc);
            return EXCEPTION;
        }

        let next_exe = func.executable();
        if exe.ptr_eq(next_exe) {
            runtime.leave_hook.set(*pc, post_exec);
            runtime.local_table.push_scope();
            for env_obj in func.environment() {
                runtime.local_table.add_ref(Rc::clone(env_obj));
            }
            for arg in args {
                runtime.local_table.add(arg);
            }
            let next_pc = func.start_index();
            *pc = next_pc;
        } else {
            let result = {
                let mut runtime = Runtime::new();
                for env_obj in func.environment() {
                    runtime.local_table.add_ref(Rc::clone(env_obj));
                }
                for arg in args {
                    runtime.local_table.add(arg);
                }
                loop_(Executable::clone(next_exe), &mut runtime).map_err(|_| {
                    let message = "Error occurred while calling function.".to_string();
                    EXCEPTION_LOG.lock().unwrap().push_raw(message, *pc, 0);
                })?;
                let mut result = runtime.stack.pop();
                if let Some(post_exec) = post_exec {
                    result = post_exec(result)?;
                }
                result
            };
            runtime.stack.push(result);
            *pc += 1;
        }
        CONTINUE
    }

    pub(super) fn exec_rust_function<I>(
        func: object::RustFunction,
        args: I,
        context: LoopContextRef,
    ) -> Status
    where
        I: IntoIterator<Item = Object> + 'static,
        I::IntoIter: ExactSizeIterator,
    {
        exec_rust_function_core(func, Box::new(args.into_iter()), context, None)
    }

    pub(super) fn exec_rust_function_with_post_exec<I>(
        func: object::RustFunction,
        args: I,
        context: LoopContextRef,
        post_exec: impl FnOnce(Object) -> Result<Object, ()> + 'static,
    ) -> Status
    where
        I: IntoIterator<Item = Object> + 'static,
        I::IntoIter: ExactSizeIterator,
    {
        exec_rust_function_core(
            func,
            Box::new(args.into_iter()),
            context,
            Some(Box::new(post_exec)),
        )
    }

    #[allow(clippy::type_complexity)]
    fn exec_rust_function_core(
        func: object::RustFunction,
        args: Box<dyn ExactSizeIterator<Item = Object>>,
        context: LoopContextRef,
        post_exec: Option<Box<dyn FnOnce(Object) -> Result<Object, ()>>>,
    ) -> Status {
        let (pc, _, runtime) = context;

        if func.param_len() != args.len() as u8 {
            set_function_argument_length_exception(func.param_len(), args.len(), *pc);
            return EXCEPTION;
        }

        let mut result = func.call(args).map_err(|err| {
            let message = format!("Rust function call failed:\n{:?}", err);
            EXCEPTION_LOG.lock().unwrap().push_raw(message, *pc, 0);
        })?;
        if let Some(post_exec) = post_exec {
            result = post_exec(result)?;
        }
        runtime.stack.push(result);
        *pc += 1;
        CONTINUE
    }

    pub(super) fn ensure_array_index(
        array: &object::Array,
        index: i64,
        pc: usize,
    ) -> Option<usize> {
        #[cold]
        fn set_index_out_of_range_exception(index: i64, len: usize, pc: usize) {
            let message = format!(
                "Index out of range {}..{}, got {}.",
                -(len as i64),
                len,
                index
            );
            EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
        }

        let fixed_index = if index < 0 {
            index + array.len() as i64
        } else {
            index
        };
        if fixed_index < 0 || fixed_index as usize >= array.len() {
            set_index_out_of_range_exception(index, array.len(), pc);
            return None;
        }
        Some(fixed_index as usize)
    }

    #[cold]
    pub(super) fn set_container_key_type_exception(
        container_type: &str,
        key_type: &str,
        pc: usize,
    ) {
        let message = format!(
            "The key of type '{}' is not valid for the container of type '{}'.",
            key_type, container_type
        );
        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
    }

    #[cold]
    pub(super) fn set_not_indexable_exception(type_name: &str, pc: usize) {
        let message = format!("The object of type '{}' is not indexable.", type_name);
        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
    }

    #[cold]
    fn set_function_argument_length_exception(expected: u8, got: usize, pc: usize) {
        let message = format!(
            "Function call failed: expected {} arguments, got {}.",
            expected, got
        );
        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
    }

    #[cold]
    pub(super) fn set_binary_type_exception(
        op: &'static str,
        lhs: &Object,
        rhs: &Object,
        pc: usize,
    ) {
        let message = format!(
            "Operator '{}' cannot be applied to operands type of '{}' and '{}'.",
            op,
            lhs.type_name(),
            rhs.type_name()
        );
        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
    }

    #[cold]
    pub(super) fn set_unary_type_exception(op: &'static str, value: &Object, pc: usize) {
        let message = format!(
            "Operator '{}' cannot be applied to operand type of '{}'.",
            op,
            value.type_name()
        );
        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
    }
}
