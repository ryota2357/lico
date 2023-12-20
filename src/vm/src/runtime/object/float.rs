use super::*;

pub fn run_float_method(float: f64, name: &str, args: Vec<Object>) -> Result<Object, String> {
    match name {
        "abs" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.abs()))
        }
        "acos" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.acos()))
        }
        "acosh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.acosh()))
        }
        "asin" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.asin()))
        }
        "asinh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.asinh()))
        }
        "atan" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.atan()))
        }
        "atan2" => {
            ensure_argument_length!(args, 1);
            let Object::Float(other) = args[0] else {
                return Err(format!("{} takes an float", name));
            };
            Ok(Object::Float(float.atan2(other)))
        }
        "atanh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.atanh()))
        }
        "cbar" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.cbrt()))
        }
        "ceil" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.ceil()))
        }
        "clamp" => {
            ensure_argument_length!(args, 2);
            let Object::Float(min) = args[0] else {
                return Err(format!("{} takes an float", name));
            };
            let Object::Float(max) = args[1] else {
                return Err(format!("{} takes an float", name));
            };
            Ok(Object::Float(float.clamp(min, max)))
        }
        "cos" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.cos()))
        }
        "cosh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.cosh()))
        }
        "exp" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.exp()))
        }
        "exp2" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.exp2()))
        }
        "floor" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.floor()))
        }
        "fract" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.fract()))
        }
        "ln" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.ln()))
        }
        "log" => {
            ensure_argument_length!(args, 1);
            match args[0] {
                Object::Float(base) => Ok(Object::Float(float.log(base))),
                Object::Int(base) => Ok(Object::Float(float.log(base as f64))),
                _ => Err(format!("{} takes an float", name)),
            }
        }
        "log10" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.log10()))
        }
        "log2" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.log2()))
        }
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
        "recip" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.recip()))
        }
        "round" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.round()))
        }
        "sin" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.sin()))
        }
        "sinh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.sinh()))
        }
        "sqrt" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.sqrt()))
        }
        "tan" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.tan()))
        }
        "tanh" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.tanh()))
        }
        "to_degrees" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.to_degrees()))
        }
        "to_string" => {
            ensure_argument_length!(args, 0);
            let string = float.to_string();
            Ok(Object::new_string(string))
        }
        "to_radians" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.to_radians()))
        }
        "trunc" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Float(float.trunc()))
        }
        _ => Err(format!("{} is not a method of float", name)),
    }
}
