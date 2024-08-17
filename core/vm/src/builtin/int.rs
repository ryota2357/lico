use super::*;

pub(crate) fn run_method(
    name: &str,
    receiver: i64,
    args: impl ExactSizeIterator<Item = Object>,
) -> RunMethodResult {
    let args = args.into_iter();

    #[rustfmt::skip]
    let ret = match name {
        // common methods
        "to_string" => method::to_string(receiver, args),

        // number methods
        "abs"   => method::abs(receiver, args),
        "acos"  => method::acos(receiver,args),
        "acosh" => method::acosh(receiver,args),
        "asin"  => method::asin(receiver,args),
        "asinh" => method::asinh(receiver,args),
        "atan"  => method::atan(receiver,args),
        "atan2" => method::atan2(receiver,args),
        "atanh" => method::atanh(receiver,args),
        "cbrt"  => method::cbrt(receiver,args),
        "ceil"  => method::ceil(receiver,args),
        "clamp" => method::clamp(receiver,args),
        "cos"   => method::cos(receiver,args),
        "cosh"  => method::cosh(receiver,args),
        "exp"   => method::exp(receiver,args),
        "exp2"  => method::exp2(receiver,args),
        "floor" => method::floor(receiver,args),
        "fract" => method::fract(receiver,args),
        "ln"    => method::ln(receiver,args),
        "log"   => method::log(receiver,args),
        "log10" => method::log10(receiver,args),
        "log2"  => method::log2(receiver,args),
        "max"   => method::max(receiver,args),
        "min"   => method::min(receiver,args),
        "pow"   => method::pow(receiver,args),
        "round" => method::round(receiver,args),
        "sin"   => method::sin(receiver,args),
        "sinh"  => method::sinh(receiver,args),
        "sqrt"  => method::sqrt(receiver,args),
        "tan"   => method::tan(receiver,args),
        "tanh"  => method::tanh(receiver,args),
        "trunc" => method::trunc(receiver,args),

        // int methods
        "downto" => method::downto(receiver, args),
        "upto" => method::upto(receiver, args),

        _ => RunMethodResult::NotFound {
            receiver_type: TypeFlag::INT,
        }
    };
    ret
}

mod method {
    use super::*;
    use anyhow::anyhow;
    use Object::*;
    use RunMethodResult::*;

    util_macros::gen_method_macro!(i64);

    fn take_next_arg_as_float(
        index: u8,
        args: &mut impl Iterator<Item = Object>,
    ) -> Result<f64, RunMethodResult> {
        let got = match args.next().unwrap() {
            Int(i) => return Result::Ok(i as f64),
            Float(f) => return Result::Ok(f),
            arg => TypeFlag::from(&arg),
        };
        Err(InvalidArgType {
            index,
            expected: TypeFlag::INT | TypeFlag::FLOAT,
            got,
        })
    }

    method!(to_string, 0, |this, args| {
        let string = UString::from(this.to_string().as_str());
        Ok(String(string))
    });

    // abs() -> int
    method!(abs, 0, |this, args| Ok(Int(this.abs())));

    // acos() -> float
    method!(acos, 0, |this, args| Ok(Float((this as f64).acos())));

    // acosh() -> float
    method!(acosh, 0, |this, args| Ok(Float((this as f64).acosh())));

    // asin() -> float
    method!(asin, 0, |this, args| Ok(Float((this as f64).asin())));

    // asinh() -> float
    method!(asinh, 0, |this, args| Ok(Float((this as f64).asinh())));

    // atan() -> float
    method!(atan, 0, |this, args| Ok(Float((this as f64).atan())));

    // atan2(other: int|float) -> float
    method!(atan2, 1, |this, args| {
        let other = match take_next_arg_as_float(0, &mut args) {
            Result::Ok(other) => other,
            Err(err) => return err,
        };
        Ok(Float((this as f64).atan2(other)))
    });

    // atanh() -> float
    method!(atanh, 0, |this, args| Ok(Float((this as f64).atanh())));

    // cbrt() -> float
    method!(cbrt, 0, |this, args| Ok(Float((this as f64).cbrt())));

    // ceil() -> float
    method!(ceil, 0, |this, args| Ok(Float((this as f64).ceil())));

    // clamp(min: int|float, max: int|float) -> int|float
    method!(clamp, 2, |this, args| {
        match (args.next().unwrap(), args.next().unwrap()) {
            (Int(min), Int(max)) => Ok(Int(this.clamp(min, max))),
            (Int(min), Float(max)) => {
                if max.is_nan() {
                    EXCEPTION_LOG.lock().unwrap().error("error: max is NaN");
                }
                if this <= min {
                    Ok(Int(min))
                } else if this as f64 >= max {
                    Ok(Float(max))
                } else {
                    Ok(Int(this))
                }
            }
            (Float(min), Int(max)) => {
                if min.is_nan() {
                    EXCEPTION_LOG.lock().unwrap().error("error: min is NaN");
                }
                if this as f64 <= min {
                    Ok(Float(min))
                } else if this >= max {
                    Ok(Int(max))
                } else {
                    Ok(Float(this as f64))
                }
            }
            (Float(min), Float(max)) => {
                if min.is_nan() {
                    EXCEPTION_LOG.lock().unwrap().error("error: min is NaN");
                }
                if max.is_nan() {
                    EXCEPTION_LOG.lock().unwrap().error("error: max is NaN");
                }
                if this as f64 <= min {
                    Ok(Float(min))
                } else if this as f64 >= max {
                    Ok(Float(max))
                } else {
                    Ok(Int(this))
                }
            }
            (arg, _) => InvalidArgType {
                index: 0,
                expected: TypeFlag::INT | TypeFlag::FLOAT,
                got: TypeFlag::from(&arg),
            },
        }
    });

    // cos() -> float
    method!(cos, 0, |this, args| Ok(Float((this as f64).cos())));

    // cosh() -> float
    method!(cosh, 0, |this, args| Ok(Float((this as f64).cosh())));

    // exp() -> float
    method!(exp, 0, |this, args| Ok(Float((this as f64).exp())));

    // exp2() -> float
    method!(exp2, 0, |this, args| Ok(Float((this as f64).exp2())));

    // floor() -> int
    method!(floor, 0, |this, args| Ok(Int(this)));

    // fract() -> int
    method!(fract, 0, |this, args| Ok(Float((this as f64).fract())));

    // ln(f) -> float
    method!(ln, 0, |this, args| Ok(Float((this as f64).ln())));

    // log(base: float|int) -> float
    method!(log, 1, |this, args| {
        let base = match take_next_arg_as_float(0, &mut args) {
            Result::Ok(base) => base,
            Err(err) => return err,
        };
        Ok(Float((this as f64).log(base)))
    });

    // log10() -> float
    method!(log10, 0, |this, args| Ok(Float((this as f64).log10())));

    // log2() -> float
    method!(log2, 0, |this, args| Ok(Float((this as f64).log2())));

    // max(other: int) -> int
    method!(max, 1, |this, args| {
        match args.next().unwrap() {
            Int(other) => Ok(Int(this.max(other))),
            Float(other) => {
                if this as f64 >= other {
                    Ok(Int(this))
                } else {
                    Ok(Float(other))
                }
            }
            other => InvalidArgType {
                index: 0,
                expected: TypeFlag::INT,
                got: TypeFlag::from(&other),
            },
        }
    });

    // min(other: int) -> int
    method!(min, 1, |this, args| {
        match args.next().unwrap() {
            Int(other) => Ok(Int(this.min(other))),
            Float(other) => {
                if this as f64 <= other {
                    Ok(Int(this))
                } else {
                    Ok(Float(other))
                }
            }
            other => InvalidArgType {
                index: 0,
                expected: TypeFlag::INT,
                got: TypeFlag::from(&other),
            },
        }
    });

    // pow(exp: int|float) -> float
    method!(pow, 1, |this, args| {
        let exp = match take_next_arg_as_float(0, &mut args) {
            Result::Ok(exp) => exp,
            Err(err) => return err,
        };
        Ok(Float((this as f64).powf(exp)))
    });

    // round() -> int
    method!(round, 0, |this, args| Ok(Int(this)));

    // sin() -> float
    method!(sin, 0, |this, args| Ok(Float((this as f64).sin())));

    // sinh() -> float
    method!(sinh, 0, |this, args| Ok(Float((this as f64).sinh())));

    // sqrt() -> float
    method!(sqrt, 0, |this, args| Ok(Float((this as f64).sqrt())));

    // tan() -> float
    method!(tan, 0, |this, args| Ok(Float((this as f64).tan())));

    // tanh() -> float
    method!(tanh, 0, |this, args| Ok(Float((this as f64).tanh())));

    // trunc() -> int
    method!(trunc, 0, |this, args| Ok(Int(this)));

    // downto(min: int|float) -> table
    method!(downto, 1, |this, args| {
        create_range_iter_table(this, args.next().unwrap(), true)
    });

    // upto(max: int|float) -> table
    method!(upto, 1, |this, args| {
        create_range_iter_table(this, args.next().unwrap(), false)
    });

    fn create_range_iter_table(start: i64, limit: Object, reverse: bool) -> RunMethodResult {
        let limit = match limit {
            limit @ (Int(_) | Float(_)) => limit,
            arg => {
                return InvalidArgType {
                    index: 0,
                    expected: TypeFlag::INT | TypeFlag::FLOAT,
                    got: TypeFlag::from(&arg),
                }
            }
        };
        let mut iter_tbl = object::Table::from([
            ("start".into(), Int(start)),
            ("end".into(), limit),
            ("__current".into(), Nil),
        ]);
        iter_tbl.set_method(
            "__get_iter".into(), // __get_iter() -> table
            TableMethod::Native(object::RustFunction::new(1, |mut args| {
                Result::Ok(args.next().unwrap())
            })),
        );
        fn get_current(tbl: &object::Table) -> Option<anyhow::Result<i64>> {
            match tbl.get("__current")? {
                Int(current) => Some(Result::Ok(*current)),
                invalid => Some(Err(anyhow!(
                    "The field '__current' is not an integer: got '{}'",
                    invalid.type_name()
                ))),
            }
        }
        fn get_end(tbl: &object::Table) -> anyhow::Result<i64> {
            match tbl.get("end") {
                Some(Int(end)) => Result::Ok(*end),
                Some(invalid) => Err(anyhow!(
                    "The field 'end' is not an integer: got '{}'",
                    invalid.type_name()
                )),
                None => Err(anyhow!("The field 'end' is not found")),
            }
        }
        if reverse {
            iter_tbl.set_method(
                "__move_next".into(), // __move_next() -> bool
                TableMethod::Native(object::RustFunction::new(1, |mut args| {
                    let Table(mut this) = args.next().unwrap() else {
                        panic!("[BUG?] unexpected type of `self`")
                    };
                    let current = match get_current(&this) {
                        Some(Result::Ok(current)) => current,
                        Some(Err(err)) => return Err(err),
                        None => return Result::Ok(Bool(false)),
                    };
                    let end = get_end(&this)?;
                    if current > end {
                        // TODO: use `entry` after implementing `Table::entry`
                        this.insert("__current".into(), Int(current - 1));
                        Result::Ok(Bool(true))
                    } else {
                        Result::Ok(Bool(false))
                    }
                })),
            );
        } else {
            iter_tbl.set_method(
                "__move_next".into(), // __move_next() -> bool
                TableMethod::Native(object::RustFunction::new(1, |mut args| {
                    let Table(mut this) = args.next().unwrap() else {
                        panic!("[BUG?] unexpected type of `self`")
                    };
                    let current = match get_current(&this) {
                        Some(Result::Ok(current)) => current,
                        Some(Err(err)) => return Err(err),
                        None => return Result::Ok(Bool(false)),
                    };
                    let end = get_end(&this)?;
                    if current < end {
                        // TODO: use `entry` after implementing `Table::entry`
                        this.insert("__current".into(), Int(current + 1));
                        Result::Ok(Bool(true))
                    } else {
                        Result::Ok(Bool(false))
                    }
                })),
            );
        }
        iter_tbl.set_method(
            "__current".into(), // __current() -> int
            TableMethod::Native(object::RustFunction::new(1, |mut args| {
                let Table(this) = args.next().unwrap() else {
                    panic!("[BUG?] unexpected type of `self")
                };
                Result::Ok(this.get("__current").cloned().unwrap_or(Nil))
            })),
        );
        Ok(Table(iter_tbl))
    }
}
