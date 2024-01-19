use super::*;

pub fn run_int_method(int: i64, name: &str, args: &[Object]) -> Result<Object, String> {
    match name {
        // abs() -> Int
        "abs" => {
            extract_argument!(args, []);
            Ok(Object::Int(int.abs()))
        }

        // acos() -> Float
        "acos" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).acos()))
        }

        // acosh() -> Float
        "acosh" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).acosh()))
        }

        // asin() -> Float
        "asin" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).asin()))
        }

        // asinh() -> Float
        "asinh" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).asinh()))
        }

        // atan() -> Float
        "atan" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).atan()))
        }

        // atan2(other: Float) -> Float
        "atan2" => {
            let other = extract_argument!(args, [Float]);
            Ok(Object::Float((int as f64).atan2(other)))
        }

        // atanh() -> Float
        "atanh" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).atanh()))
        }

        // cbrt() -> Float
        "cbar" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).cbrt()))
        }

        // ceil() -> Float
        "ceil" => {
            extract_argument!(args, []);
            Ok(Object::Int(int))
        }

        // clamp(min: Int|Float, max: Int|Float) -> Int
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

        // cos() -> Float
        "cos" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).cos()))
        }

        // cosh() -> Float
        "cosh" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).cosh()))
        }

        // div(int) -> Int
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
                    extract_argument!(args, []);
                    Ok(Object::Table(range))
                }),
            );
            range_tbl.add_method(
                "__move_next",
                TableMethod::Builtin(|range, args| {
                    extract_argument!(args, []);
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
                    extract_argument!(args, []);
                    let current = range.borrow().get("__current").cloned();
                    Ok(current.unwrap_or(Object::Nil))
                }),
            );
            Ok(Object::new_table(range_tbl))
        }

        // exp() -> Float
        "exp" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).exp()))
        }

        // exp2() -> Float
        "exp2" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).exp2()))
        }

        // floor() -> Int
        "floor" => {
            extract_argument!(args, []);
            Ok(Object::Int(int))
        }

        // fract() -> Int
        "fract" => {
            extract_argument!(args, []);
            Ok(Object::Int(0))
        }

        // hypot(float) -> Float
        "ln" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).ln()))
        }

        // log(base: Float|Int) -> Float
        "log" => {
            let base = extract_argument!(args, [
                {
                    Object::Float(base) => *base,
                    Object::Int(base) => *base as f64,
                    _ => return Err(format!("{} takes an float", name)),
                }
            ]);
            Ok(Object::Float((int as f64).log(base)))
        }

        // log10() -> Float
        "log10" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).log10()))
        }

        // log2() -> Float
        "log2" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).log2()))
        }

        // max(amount: Int) -> Int
        "lshift" => {
            let amount = extract_argument!(args, [Int]);
            Ok(Object::Int(int << amount))
        }

        // min(other: Int) -> Int
        "pow" => {
            ensure_argument_length!(args, 1);
            unimplemented!("int pow")
        }

        // round() -> Int
        "recip" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).recip()))
        }

        // round() -> Int
        "round" => {
            extract_argument!(args, []);
            Ok(Object::Int(int))
        }

        // rshift(amount: Int) -> Int
        "rshift" => {
            let amount = extract_argument!(args, [Int]);
            Ok(Object::Int(int >> amount))
        }

        // sin() -> Float
        "sin" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).sin()))
        }

        // sinh() -> Float
        "sinh" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).sinh()))
        }

        // sqrt() -> Float
        "sqrt" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).sqrt()))
        }

        // tan() -> Float
        "tan" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).tan()))
        }

        // tanh() -> Float
        "tanh" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).tanh()))
        }

        // trunc() -> Float
        "to_degrees" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).to_degrees()))
        }

        // trunc() -> String
        "to_string" => {
            extract_argument!(args, []);
            let string = int.to_string();
            Ok(Object::new_string(string))
        }

        // trunc() -> Float
        "to_radians" => {
            extract_argument!(args, []);
            Ok(Object::Float((int as f64).to_radians()))
        }

        // trunc() -> Int
        "trunc" => {
            extract_argument!(args, []);
            Ok(Object::Int(int))
        }

        // upto(to: Int) -> Table
        "upto" => {
            let to = extract_argument!(args, [Int]);
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
                "step", // step(step: Int) -> Nil
                TableMethod::Builtin(|range, args| {
                    let step = extract_argument!(args, [Int]);
                    if step <= 0 {
                        return Err(format!("step should be positive, got {}", step));
                    }
                    range.borrow_mut().insert("step".into(), Object::Int(step));
                    Ok(Object::Nil)
                }),
            );
            range_tbl.add_method(
                "__get_iterator", // __get_iterator() -> Table
                TableMethod::Builtin(|range, args| {
                    extract_argument!(args, []);
                    Ok(Object::Table(Rc::clone(&range)))
                }),
            );
            if int <= to {
                range_tbl.add_method(
                    "__move_next", // __move_next() -> Bool
                    TableMethod::Builtin(|range, args| {
                        extract_argument!(args, []);
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
                    "__move_next", // __move_next() -> Bool
                    TableMethod::Builtin(|_, args| {
                        extract_argument!(args, []);
                        Ok(Object::Bool(false))
                    }),
                );
            }
            range_tbl.add_method(
                "__current", // __current() -> Int|Nil
                TableMethod::Builtin(|range, args| {
                    extract_argument!(args, []);
                    let current = range.borrow().get("__current").cloned();
                    Ok(current.unwrap_or(Object::Nil))
                }),
            );
            Ok(Object::new_table(range_tbl))
        }

        // xor(other: Int) -> Int
        "xor" => {
            let other = extract_argument!(args, [Int]);
            Ok(Object::Int(int ^ other))
        }

        _ => Err(format!("{} is not a method of int", name)),
    }
}
