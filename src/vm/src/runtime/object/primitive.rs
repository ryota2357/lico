use super::*;

pub fn run_bool_method(bool: bool, name: &str, args: Vec<Object>) -> Result<Object, String> {
    match name {
        "to_string" => {
            ensure_argument_length!(args, 0);
            let string = bool.to_string();
            Ok(Object::new_string(string))
        }
        _ => Err(format!("{} is not a method of bool", name)),
    }
}

pub fn run_nil_method(name: &str, args: Vec<Object>) -> Result<Object, String> {
    match name {
        "to_string" => {
            ensure_argument_length!(args, 0);
            let string = "nil".to_string();
            Ok(Object::new_string(string))
        }
        _ => Err(format!("{} is not a method of nil", name)),
    }
}
