use super::*;

pub fn run_int_method<'a>(
    int: i64,
    name: &'a str,
    args: Vec<Object<'a>>,
) -> Result<Object<'a>, String> {
    match name {
        "abs" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Int(int.abs()))
        }
        "downto" => {
            ensure_argument_length!(args, 1);
            let Object::Int(to) = args[0] else {
                return Err(format!("{} takes an int", name));
            };
            let mut range_tbl = TableObject::new(
                [
                    ("start".to_string(), Object::Int(int)),
                    ("end".to_string(), Object::Int(to)),
                    ("step".to_string(), Object::Int(-1)),
                    ("__current".to_string(), Object::Nil),
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
                    range
                        .borrow_mut()
                        .insert("step".to_string(), Object::Int(step));
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
                                .insert("__current".to_string(), Object::Int(current - 1));
                            Ok(Object::Bool(true))
                        } else {
                            range
                                .borrow_mut()
                                .insert("__current".to_string(), Object::Nil);
                            Ok(Object::Bool(false))
                        }
                    } else {
                        let Some(Object::Int(start)) = range.borrow().get("start").cloned() else {
                            unreachable!("range should have `start` field (int)")
                        };
                        range
                            .borrow_mut()
                            .insert("__current".to_string(), Object::Int(start));
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
        "to_string" => {
            ensure_argument_length!(args, 0);
            Ok(Object::String(int.to_string()))
        }
        "upto" => {
            ensure_argument_length!(args, 1);
            let Object::Int(to) = args[0] else {
                return Err(format!("{} takes an int", name));
            };
            let mut range_tbl = TableObject::new(
                [
                    ("start".to_string(), Object::Int(int)),
                    ("end".to_string(), Object::Int(to)),
                    ("step".to_string(), Object::Int(1)),
                    ("__current".to_string(), Object::Nil),
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
                    range
                        .borrow_mut()
                        .insert("step".to_string(), Object::Int(step));
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
                                    .insert("__current".to_string(), Object::Int(current + 1));
                                Ok(Object::Bool(true))
                            } else {
                                range
                                    .borrow_mut()
                                    .insert("__current".to_string(), Object::Nil);
                                Ok(Object::Bool(false))
                            }
                        } else {
                            let Some(Object::Int(start)) = range.borrow().get("start").cloned()
                            else {
                                unreachable!("range should have `start` field (int)")
                            };
                            range
                                .borrow_mut()
                                .insert("__current".to_string(), Object::Int(start));
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

pub fn run_float_method<'a>(
    float: f64,
    name: &'a str,
    args: Vec<Object<'a>>,
) -> Result<Object<'a>, String> {
    match name {
        "abs" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.abs()))
        }
        "to_string" => {
            ensure_argument_length!(args, 0);
            Ok(Object::String(float.to_string()))
        }
        _ => Err(format!("{} is not a method of float", name)),
    }
}

pub fn run_string_method<'a>(
    string: String,
    name: &'a str,
    args: Vec<Object<'a>>,
) -> Result<Object<'a>, String> {
    match name {
        "len" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Int(string.len() as i64))
        }
        "to_string" => {
            ensure_argument_length!(args, 0);
            Ok(Object::String(string.to_string()))
        }
        _ => Err(format!("{} is not a method of string", name)),
    }
}

pub fn run_bool_method<'a>(
    bool: bool,
    name: &'a str,
    args: Vec<Object<'a>>,
) -> Result<Object<'a>, String> {
    match name {
        "to_string" => {
            ensure_argument_length!(args, 0);
            Ok(Object::String(bool.to_string()))
        }
        _ => Err(format!("{} is not a method of bool", name)),
    }
}

pub fn run_nil_method<'a>(name: &'a str, args: Vec<Object<'a>>) -> Result<Object<'a>, String> {
    match name {
        "to_string" => {
            ensure_argument_length!(args, 0);
            Ok(Object::String("nil".to_string()))
        }
        _ => Err(format!("{} is not a method of nil", name)),
    }
}
