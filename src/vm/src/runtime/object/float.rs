use super::*;

pub fn run_float_method(float: f64, name: &str, args: &[Object]) -> Result<Object, String> {
    match name {
        // abs() -> Float
        "abs" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.abs()))
        }

        // acos() -> Float
        "acos" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.acos()))
        }

        // acosh() -> Float
        "acosh" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.acosh()))
        }

        // asin() -> Float
        "asin" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.asin()))
        }

        // asinh() -> Float
        "asinh" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.asinh()))
        }

        // atan() -> Float
        "atan" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.atan()))
        }

        // atan2(float) -> Float
        "atan2" => {
            let other = extract_argument!(args, [Float]);
            Ok(Object::Float(float.atan2(other)))
        }

        // atanh() -> Float
        "atanh" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.atanh()))
        }

        // cbrt() -> Float
        "cbar" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.cbrt()))
        }

        // ceil() -> Float
        "ceil" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.ceil()))
        }

        // clamp(min: Float, max: Float) -> Float
        "clamp" => {
            let (min, max) = extract_argument!(args, [Float, Float]);
            Ok(Object::Float(float.clamp(min, max)))
        }

        // cos() -> Float
        "cos" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.cos()))
        }

        // cosh() -> Float
        "cosh" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.cosh()))
        }

        // exp() -> Float
        "exp" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.exp()))
        }

        // exp2() -> Float
        "exp2" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.exp2()))
        }

        // floor() -> Float
        "floor" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.floor()))
        }

        // fract() -> Float
        "fract" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.fract()))
        }

        // ln() -> Float
        "ln" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.ln()))
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
            Ok(Object::Float(float.log(base)))
        }

        // log10() -> Float
        "log10" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.log10()))
        }

        // log2() -> Float
        "log2" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.log2()))
        }

        // pow(exp: Float|Int) -> Float
        "pow" => {
            ensure_argument_length!(args, 1);
            match args[0] {
                Object::Float(exp) => Ok(Object::Float(float.powf(exp))),
                Object::Int(exp) => {
                    let min = i32::MIN as i64;
                    let max = i32::MAX as i64;
                    if min <= exp && exp <= max {
                        Ok(Object::Float(float.powi(exp as i32)))
                    } else {
                        Ok(Object::Float(float.powf(exp as f64)))
                    }
                }
                _ => Err(format!("{} takes an float", name)),
            }
        }

        // recip() -> Float
        "recip" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.recip()))
        }

        // round() -> Float
        "round" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.round()))
        }

        // sin() -> Float
        "sin" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.sin()))
        }

        // sinh() -> Float
        "sinh" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.sinh()))
        }

        // sqrt() -> Float
        "sqrt" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.sqrt()))
        }

        // tan() -> Float
        "tan" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.tan()))
        }

        // tanh() -> Float
        "tanh" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.tanh()))
        }

        // to_degrees() -> Float
        "to_degrees" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.to_degrees()))
        }

        // to_radians() -> Float
        "to_string" => {
            extract_argument!(args, []);
            let string = float.to_string();
            Ok(Object::new_string(string))
        }

        // to_radians() -> Float
        "to_radians" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.to_radians()))
        }

        // trunc() -> Float
        "trunc" => {
            extract_argument!(args, []);
            Ok(Object::Float(float.trunc()))
        }

        _ => Err(format!("{} is not a method of float", name)),
    }
}
