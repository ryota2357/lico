use super::*;

pub(crate) fn run_method(
    name: &str,
    receiver: f64,
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
        _ => RunMethodResult::NotFound {
            receiver_type: TypeFlag::FLOAT,
        }
    };
    ret
}

mod method {
    use super::*;
    use Object::*;
    use RunMethodResult::*;

    util_macros::gen_method_macro!(f64);

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
        Ok(Object::String(string))
    });

    // abs() -> float
    method!(abs, 0, |this, args| Ok(Float(this.abs())));

    // acos() -> float
    method!(acos, 0, |this, args| Ok(Float(this.acos())));

    // acosh() -> float
    method!(acosh, 0, |this, args| Ok(Float(this.acosh())));

    // asin() -> float
    method!(asin, 0, |this, args| Ok(Float(this.asin())));

    // asinh() -> float
    method!(asinh, 0, |this, args| Ok(Float(this.asinh())));

    // atan() -> float
    method!(atan, 0, |this, args| Ok(Float(this.atan())));

    // atan2(other: int|float) -> float
    method!(atan2, 1, |this, args| {
        let other = match take_next_arg_as_float(0, &mut args) {
            Result::Ok(other) => other,
            Err(err) => return err,
        };
        Ok(Float(this.atan2(other)))
    });

    // atanh() -> float
    method!(atanh, 0, |this, args| Ok(Float(this.atanh())));

    // cbrt() -> float
    method!(cbrt, 0, |this, args| Ok(Float(this.cbrt())));

    // ceil() -> float
    method!(ceil, 0, |this, args| Ok(Float(this.ceil())));

    // clamp(min: Float, max: Float) -> float
    method!(clamp, 2, |this, args| {
        let min = match take_next_arg_as_float(0, &mut args) {
            Result::Ok(min) => min,
            Err(err) => return err,
        };
        let max = match take_next_arg_as_float(1, &mut args) {
            Result::Ok(max) => max,
            Err(err) => return err,
        };
        if min.is_nan() {
            EXCEPTION_LOG.lock().unwrap().error("error: min is NaN");
            ExceptionOccurred
        } else if max.is_nan() {
            EXCEPTION_LOG.lock().unwrap().error("error: max is NaN");
            ExceptionOccurred
        } else if min > max {
            EXCEPTION_LOG.lock().unwrap().error("error: min > max");
            ExceptionOccurred
        } else {
            Ok(Float(this.clamp(min, max)))
        }
    });

    // cos() -> float
    method!(cos, 0, |this, args| Ok(Float(this.cos())));

    // cosh() -> float
    method!(cosh, 0, |this, args| Ok(Float(this.cosh())));

    // exp() -> float
    method!(exp, 0, |this, args| Ok(Float(this.exp())));

    // exp2() -> float
    method!(exp2, 0, |this, args| Ok(Float(this.exp2())));

    // floor() -> float
    method!(floor, 0, |this, args| Ok(Float(this.floor())));

    // fract() -> float
    method!(fract, 0, |this, args| Ok(Float(this.fract())));

    // ln() -> float
    method!(ln, 0, |this, args| Ok(Float(this.ln())));

    // log(base: float|int) -> float
    method!(log, 1, |this, args| {
        let base = match take_next_arg_as_float(0, &mut args) {
            Result::Ok(base) => base,
            Err(err) => return err,
        };
        Ok(Float(this.log(base)))
    });

    // log10() -> float
    method!(log10, 0, |this, args| Ok(Float(this.log10())));

    // log2() -> float
    method!(log2, 0, |this, args| Ok(Float(this.log2())));

    // max(other: int|float) -> int|float
    method!(max, 1, |this, args| {
        match args.next().unwrap() {
            Int(other) => {
                if this as i64 >= other {
                    Ok(Float(this))
                } else {
                    Ok(Int(other))
                }
            }
            Float(other) => Ok(Float(this.max(other))),
            other => InvalidArgType {
                index: 0,
                expected: TypeFlag::INT | TypeFlag::FLOAT,
                got: TypeFlag::from(&other),
            },
        }
    });

    // min(other: int|float) -> int|float
    method!(min, 1, |this, args| {
        match args.next().unwrap() {
            Int(other) => {
                if this as i64 <= other {
                    Ok(Float(this))
                } else {
                    Ok(Int(other))
                }
            }
            Float(other) => Ok(Float(this.min(other))),
            other => InvalidArgType {
                index: 0,
                expected: TypeFlag::INT | TypeFlag::FLOAT,
                got: TypeFlag::from(&other),
            },
        }
    });

    // pow(exp: float|int) -> float
    method!(pow, 1, |this, args| {
        let exp = match take_next_arg_as_float(0, &mut args) {
            Result::Ok(exp) => exp,
            Err(err) => return err,
        };
        Ok(Float(this.powf(exp)))
    });

    // round() -> float
    method!(round, 0, |this, args| Ok(Float(this.round())));

    // sin() -> float
    method!(sin, 0, |this, args| Ok(Float(this.sin())));

    // sinh() -> float
    method!(sinh, 0, |this, args| Ok(Float(this.sinh())));

    // sqrt() -> float
    method!(sqrt, 0, |this, args| Ok(Float(this.sqrt())));

    // tan() -> float
    method!(tan, 0, |this, args| Ok(Float(this.tan())));

    // tanh() -> float
    method!(tanh, 0, |this, args| Ok(Float(this.tanh())));

    // trunc() -> float
    method!(trunc, 0, |this, args| Ok(Float(this.trunc())));
}
