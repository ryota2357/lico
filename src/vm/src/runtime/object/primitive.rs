use super::*;

pub fn run_bool_method(bool: bool, name: &str, args: &[Object]) -> Result<Object, String> {
    match name {
        // to_string() -> String
        "to_string" => {
            extract_argument!(args, []);
            let string = bool.to_string();
            Ok(Object::new_string(string))
        }
        _ => Err(format!("{} is not a method of bool", name)),
    }
}

pub fn run_nil_method(name: &str, args: &[Object]) -> Result<Object, String> {
    match name {
        // to_string() -> String
        "to_string" => {
            extract_argument!(args, []);
            let string = "nil".to_string();
            Ok(Object::new_string(string))
        }
        _ => Err(format!("{} is not a method of nil", name)),
    }
}
