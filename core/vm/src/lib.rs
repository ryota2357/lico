#![feature(impl_trait_in_assoc_type)]

use core::iter;
use foundation::{
    il::{Executable, ICode, Module},
    object::*,
};

mod runtime;
use runtime::*;

mod exception;
pub use exception::*;

mod builtin;

type Status = Result<(), ()>;
const EXCEPTION: Status = Err(());
const CONTINUE: Status = Ok(());

mod exec_icode;

/// Execute the module.
/// If the return value is `None`, it means that the execution was interrupted by an exception. You
/// can get the exception information from `vm::EXCEPTION_LOG`.
pub fn execute(module: &Module) -> Option<Object> {
    let mut runtime = Runtime::new();
    for (_, rfunc) in module.default_rfuncs() {
        runtime.local_table.add((*rfunc).into());
    }
    let exe = Executable::clone(module.executable());
    match loop_(exe, &mut runtime) {
        Ok(_) => {
            let result = runtime.stack.pop();
            Some(result)
        }
        Err(_) => {
            EXCEPTION_LOG.lock().unwrap().fixup(module.source_info());
            None
        }
    }
}

fn loop_(exe: Executable, runtime: &mut Runtime) -> Status {
    use ICode::*;

    let mut pc = 0;
    let exe_len = exe.len();

    loop {
        let code = unsafe {
            assert!(pc < exe_len);
            exe.fetch(pc)
        };
        match code {
            LoadIntObject(x) => {
                runtime.stack.push(Object::Int(*x));
                pc += 1;
            }
            LoadFloatObject(x) => {
                runtime.stack.push(Object::Float(*x));
                pc += 1;
            }
            LoadStringObject(x) => {
                runtime.stack.push(Object::String(x.clone()));
                pc += 1;
            }
            LoadBoolObject(x) => {
                runtime.stack.push(Object::Bool(*x));
                pc += 1;
            }
            LoadNilObject => {
                runtime.stack.push(Object::Nil);
                pc += 1;
            }
            LoadLocal(id) => {
                let value = runtime.local_table.get(*id).clone();
                runtime.stack.push(value);
                pc += 1;
            }

            Unload => {
                runtime.stack.pop();
                pc += 1;
            }

            StoreLocal(id) => {
                let value = runtime.stack.pop();
                runtime.local_table.set(*id, value);
                pc += 1;
            }
            StoreNewLocal => {
                let value = runtime.stack.pop();
                runtime.local_table.add(value);
                pc += 1;
            }

            MakeArray(len) => {
                let mut array = Array::with_capacity(*len);
                for _ in 0..*len {
                    array.push(runtime.stack.pop());
                }
                runtime.stack.push(array.into());
                pc += 1;
            }
            MakeTable(len) => {
                let mut table = Table::with_capacity(*len);
                for i in 0..*len {
                    let (key, value) = runtime.stack.pop2();
                    if let Object::String(key) = key {
                        table.insert(key, value);
                    } else {
                        let message = format!(
                            "The type of table key must be a string, but got '{}'",
                            key.type_name()
                        );
                        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, i);
                        return EXCEPTION;
                    }
                }
                runtime.stack.push(table.into());
                pc += 1;
            }

            DropLocal(count) => {
                runtime.local_table.drop(*count);
                pc += 1;
            }

            Jump(offset) => {
                pc = (pc as isize + *offset) as usize;
            }
            JumpIfTrue(offset) => {
                let value = runtime.stack.pop();
                if value.is_truthy() {
                    pc = (pc as isize + *offset) as usize;
                } else {
                    pc += 1;
                }
            }
            JumpIfFalse(offset) => {
                let value = runtime.stack.pop();
                if value.is_falsey() {
                    pc = (pc as isize + *offset) as usize;
                } else {
                    pc += 1;
                }
            }

            Call(arg_len) => match arg_len {
                0 => {
                    let calee = runtime.stack.pop();
                    exec_icode::call(calee, [], (&mut pc, &exe, runtime))?;
                }
                1 => {
                    let arg = runtime.stack.pop();
                    let calee = runtime.stack.pop();
                    exec_icode::call(calee, [arg], (&mut pc, &exe, runtime))?;
                }
                2 => {
                    let (arg1, arg2) = runtime.stack.pop2();
                    let calee = runtime.stack.pop();
                    exec_icode::call(calee, [arg1, arg2], (&mut pc, &exe, runtime))?;
                }
                3 => {
                    let (arg1, arg2, arg3) = runtime.stack.pop3();
                    let calee = runtime.stack.pop();
                    exec_icode::call(calee, [arg1, arg2, arg3], (&mut pc, &exe, runtime))?;
                }
                _ => {
                    let mut args = Vec::with_capacity(*arg_len as usize);
                    for _ in 0..*arg_len {
                        args.push(runtime.stack.pop());
                    }
                    let calee = runtime.stack.pop();
                    args.reverse();
                    exec_icode::call(calee, args, (&mut pc, &exe, runtime))?;
                }
            },
            CallMethod(arg_len, name) => match arg_len {
                0 => {
                    let receiver = runtime.stack.pop();
                    exec_icode::call_method(receiver, name, [], (&mut pc, &exe, runtime))?;
                }
                1 => {
                    let arg = runtime.stack.pop();
                    let receiver = runtime.stack.pop();
                    exec_icode::call_method(receiver, name, [arg], (&mut pc, &exe, runtime))?;
                }
                2 => {
                    let (arg1, arg2) = runtime.stack.pop2();
                    let receiver = runtime.stack.pop();
                    let args = [arg1, arg2];
                    exec_icode::call_method(receiver, name, args, (&mut pc, &exe, runtime))?;
                }
                3 => {
                    let (arg1, arg2, arg3) = runtime.stack.pop3();
                    let receiver = runtime.stack.pop();
                    let args = [arg1, arg2, arg3];
                    exec_icode::call_method(receiver, name, args, (&mut pc, &exe, runtime))?;
                }
                _ => {
                    let mut args = Vec::with_capacity(*arg_len as usize);
                    for _ in 0..*arg_len {
                        args.push(runtime.stack.pop());
                    }
                    let receiver = runtime.stack.pop();
                    args.reverse();
                    exec_icode::call_method(receiver, name, args, (&mut pc, &exe, runtime))?;
                }
            },

            SetItem => {
                let (container, key, value) = runtime.stack.pop3();
                exec_icode::set_item(container, key, value, (&mut pc, &exe, runtime))?;
            }
            GetItem => {
                let (container, key) = runtime.stack.pop2();
                exec_icode::get_item(container, key, (&mut pc, &exe, runtime))?;
            }

            SetMethod(name) => {
                let (mut table, func) = runtime.stack.pop2();
                let table: &mut Object = &mut table;
                match (table, func) {
                    (Object::Table(table), Object::Function(func)) => {
                        table.set_method(name.as_str().into(), func);
                    }
                    (Object::Table(table), Object::RustFunction(func)) => {
                        table.set_method(name.as_str().into(), func);
                    }
                    (Object::Table(_), obj) => panic!(
                        "[BUG] `SetMethod` is called with the object of type '{}'",
                        obj.type_name()
                    ),
                    (other, _) => {
                        let message = format!(
                            "Cannot set method to the object of type `{}`",
                            other.type_name()
                        );
                        EXCEPTION_LOG.lock().unwrap().push_raw(message, pc, 0);
                        break EXCEPTION;
                    }
                }
                pc += 1;
            }

            Add => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::add(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            Sub => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::sub(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            Mul => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::mul(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            Div => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::div(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            Mod => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::r#mod(lhs, rhs, (&mut pc, &exe, runtime))?;
            }

            Unm => {
                let value = runtime.stack.pop();
                exec_icode::unm(value, (&mut pc, &exe, runtime))?;
            }
            Unp => {
                let value = runtime.stack.pop();
                exec_icode::unp(value, (&mut pc, &exe, runtime))?;
            }
            Not => {
                let value = runtime.stack.pop();
                exec_icode::not(value, (&mut pc, &exe, runtime))?;
            }

            Eq => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::eq(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            NotEq => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::not_eq(lhs, rhs, (&mut pc, &exe, runtime))?;
            }

            Less => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::less(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            LessEq => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::less_eq(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            Greater => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::greater(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            GreaterEq => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::greater_eq(lhs, rhs, (&mut pc, &exe, runtime))?;
            }

            Concat => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::concat(lhs, rhs, (&mut pc, &exe, runtime))?;
            }

            BitAnd => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::bit_and(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            BitOr => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::bit_or(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            BitXor => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::bit_xor(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            BitNot => {
                let value = runtime.stack.pop();
                exec_icode::bit_not(value, (&mut pc, &exe, runtime))?;
            }

            ShiftL => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::shift_l(lhs, rhs, (&mut pc, &exe, runtime))?;
            }
            ShiftR => {
                let (lhs, rhs) = runtime.stack.pop2();
                exec_icode::shift_r(lhs, rhs, (&mut pc, &exe, runtime))?;
            }

            GetIter => {
                let value = runtime.stack.pop();
                exec_icode::get_iter(value, (&mut pc, &exe, runtime))?;
            }
            IterMoveNext => {
                let iter = runtime.stack.pop();
                exec_icode::iter_move_next(iter, (&mut pc, &exe, runtime))?;
            }
            IterCurrent => {
                let iter = runtime.stack.pop();
                exec_icode::iter_current(iter, (&mut pc, &exe, runtime))?;
            }

            BeginFuncSection => {
                pc += 1;
                let (param_len, start_pc) = match unsafe { exe.fetch(pc) } {
                    FuncSetProperty(param_len, start_pc) => (*param_len, *start_pc),
                    _ => panic!("[BUG] `FuncSetProperty` is expected"),
                };
                let env_iter = iter::from_fn(|| {
                    pc += 1;
                    match unsafe { exe.fetch(pc) } {
                        FuncAddCapture(id) => Some(runtime.local_table.get_ref(*id)),
                        EndFuncSection => None,
                        _ => panic!("[BUG] `FuncAddCapture` is expected"),
                    }
                });
                let func = Function::new(exe.clone(), param_len, start_pc, env_iter);
                runtime.stack.push(Object::Function(func));
            }
            FuncSetProperty(_, _) => panic!("[BUG] `FuncSetProperty` is not allowed here"),
            FuncAddCapture(_) => panic!("[BUG] `FuncAddCapture` is not allowed here"),
            EndFuncSection => panic!("[BUG] `EndFuncSection` is not allowed here"),

            Nop => {
                pc += 1;
            }
            Leave => match runtime.leave_hook.pop() {
                Some(hook) => {
                    if let Some(post_exec) = hook.post_exec {
                        let value = runtime.stack.pop();
                        runtime.stack.push(post_exec(value)?);
                    }
                    pc = hook.ra;
                }
                None => break Ok(()),
            },
        }
    }
}
