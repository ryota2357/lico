use super::*;

pub fn run_int_method(int: i64, name: &str, args: &[Object]) -> Result<Object, String> {
    match name {
        "abs" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Int(int.abs()))
        }
        "acos" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).acos()))
        }
        "acosh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).acosh()))
        }
        "asin" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).asin()))
        }
        "asinh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).asinh()))
        }
        "atan" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).atan()))
        }
        "atan2" => {
            ensure_argument_length!(args, 1);
            let Object::Float(other) = args[0] else {
                return Err(format!("{} takes an float", name));
            };
            Ok(Object::Float((int as f64).atan2(other)))
        }
        "atanh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).atanh()))
        }
        "cbar" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).cbrt()))
        }
        "ceil" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Int(int))
        }
        "clamp" => {
            ensure_argument_length!(args, 2);
            let a0 = &args[0];
            let a1 = &args[1];
            match (a0, a1) {
                (Object::Int(min), Object::Int(max)) => Ok(Object::Int(int.min(*max).max(*min))),
                (Object::Int(_min), Object::Float(_max)) => unimplemented!("int clamp"),
                (Object::Float(_min), Object::Int(_max)) => unimplemented!("int clamp"),
                (Object::Float(_min), Object::Float(_max)) => unimplemented!("int clamp"),
                _ => Err(format!("{} takes an int", name)),
            }
        }
        "cos" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).cos()))
        }
        "cosh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).cosh()))
        }
        "downto" => {
            ensure_argument_length!(args, 1);
            let Object::Int(to) = args[0] else {
                return Err(format!("{} takes an int", name));
            };
            let mut range_tbl = TableObject::new(
                [
                    ("start".into(), Object::Int(int)),
                    ("end".into(), Object::Int(to)),
                    ("step".into(), Object::Int(-1)),
                    ("__current".into(), Object::Nil),
                ]
                .into_iter()
                .collect(),
            );
            range_tbl.add_method(
                "step",
                TableMethod::Builtin(|range, args| {
                    ensure_argument_length!(args, 1);
                    let Object::Int(step) = args[0] else {
                        return Err(format!("expected int, got {:?}", args[0]));
                    };
                    if step >= 0 {
                        return Err(format!("step should be negative, got {}", step));
                    }
                    range.borrow_mut().insert("step".into(), Object::Int(step));
                    Ok(Object::Nil)
                }),
            );
            range_tbl.add_method(
                "__get_iterator",
                TableMethod::Builtin(|range, args| {
                    ensure_argument_length!(args, 0);
                    Ok(Object::Table(range))
                }),
            );
            range_tbl.add_method(
                "__move_next",
                TableMethod::Builtin(|range, args| {
                    ensure_argument_length!(args, 0);
                    let Some(current) = range.borrow().get("__current").cloned() else {
                        unreachable!("range should have `__current`")
                    };
                    if let Object::Int(current) = current {
                        let (end, step) = table_extract_values!(range, {
                            end: Int, step: Int,
                        });
                        if current + step >= end {
                            range
                                .borrow_mut()
                                .insert("__current".into(), Object::Int(current - 1));
                            Ok(Object::Bool(true))
                        } else {
                            range.borrow_mut().insert("__current".into(), Object::Nil);
                            Ok(Object::Bool(false))
                        }
                    } else {
                        let Some(Object::Int(start)) = range.borrow().get("start").cloned() else {
                            unreachable!("range should have `start` field (int)")
                        };
                        range
                            .borrow_mut()
                            .insert("__current".into(), Object::Int(start));
                        Ok(Object::Bool(true))
                    }
                }),
            );
            range_tbl.add_method(
                "__current",
                TableMethod::Builtin(|range, args| {
                    ensure_argument_length!(args, 0);
                    let current = range.borrow().get("__current").cloned();
                    Ok(current.unwrap_or(Object::Nil))
                }),
            );
            Ok(Object::new_table(range_tbl))
        }
        "exp" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).exp()))
        }
        "exp2" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).exp2()))
        }
        "floor" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Int(int))
        }
        "fract" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Int(0))
        }
        "ln" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).ln()))
        }
        "log" => {
            ensure_argument_length!(args, 1);
            match args[0] {
                Object::Int(base) => Ok(Object::Float((int as f64).log(base as f64))),
                Object::Float(base) => Ok(Object::Float((int as f64).log(base))),
                _ => Err(format!("{} takes an int", name)),
            }
        }
        "log10" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).log10()))
        }
        "log2" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).log2()))
        }
        "pow" => {
            ensure_argument_length!(args, 1);
            unimplemented!("int pow")
        }
        "recip" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).recip()))
        }
        "round" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Int(int))
        }
        "sin" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).sin()))
        }
        "sinh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).sinh()))
        }
        "sqrt" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).sqrt()))
        }
        "tan" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).tan()))
        }
        "tanh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).tanh()))
        }
        "to_degrees" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).to_degrees()))
        }
        "to_string" => {
            ensure_argument_length!(args, 0);
            let string = int.to_string();
            Ok(Object::new_string(string))
        }
        "to_radians" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float((int as f64).to_radians()))
        }
        "trunc" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Int(int))
        }
        "upto" => {
            ensure_argument_length!(args, 1);
            let Object::Int(to) = args[0] else {
                return Err(format!("{} takes an int", name));
            };
            let mut range_tbl = TableObject::new(
                [
                    ("start".into(), Object::Int(int)),
                    ("end".into(), Object::Int(to)),
                    ("step".into(), Object::Int(1)),
                    ("__current".into(), Object::Nil),
                ]
                .into_iter()
                .collect(),
            );
            range_tbl.add_method(
                "step",
                TableMethod::Builtin(|range, args| {
                    ensure_argument_length!(args, 1);
                    let Object::Int(step) = args[0] else {
                        return Err(format!("expected int, got {:?}", args[0]));
                    };
                    if step <= 0 {
                        return Err(format!("step should be positive, got {}", step));
                    }
                    range.borrow_mut().insert("step".into(), Object::Int(step));
                    Ok(Object::Nil)
                }),
            );
            range_tbl.add_method(
                "__get_iterator",
                TableMethod::Builtin(|range, args| {
                    ensure_argument_length!(args, 0);
                    Ok(Object::Table(Rc::clone(&range)))
                }),
            );
            if int <= to {
                range_tbl.add_method(
                    "__move_next",
                    TableMethod::Builtin(|range, args| {
                        ensure_argument_length!(args, 0);
                        let Some(current) = range.borrow().get("__current").cloned() else {
                            unreachable!("range should have `__current`")
                        };
                        if let Object::Int(current) = current {
                            let (end, step) = table_extract_values!(range, {
                                end: Int, step: Int,
                            });
                            if current + step <= end {
                                range
                                    .borrow_mut()
                                    .insert("__current".into(), Object::Int(current + 1));
                                Ok(Object::Bool(true))
                            } else {
                                range.borrow_mut().insert("__current".into(), Object::Nil);
                                Ok(Object::Bool(false))
                            }
                        } else {
                            let Some(Object::Int(start)) = range.borrow().get("start").cloned()
                            else {
                                unreachable!("range should have `start` field (int)")
                            };
                            range
                                .borrow_mut()
                                .insert("__current".into(), Object::Int(start));
                            Ok(Object::Bool(true))
                        }
                    }),
                );
            } else {
                range_tbl.add_method(
                    "__move_next",
                    TableMethod::Builtin(|_, args| {
                        ensure_argument_length!(args, 0);
                        Ok(Object::Bool(false))
                    }),
                );
            }
            range_tbl.add_method(
                "__current",
                TableMethod::Builtin(|range, args| {
                    ensure_argument_length!(args, 0);
                    let current = range.borrow().get("__current").cloned();
                    Ok(current.unwrap_or(Object::Nil))
                }),
            );
            Ok(Object::new_table(range_tbl))
        }
        _ => Err(format!("{} is not a method of int", name)),
    }
}
