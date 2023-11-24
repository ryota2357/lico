use super::*;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Debug, PartialEq)]
pub struct TableObject<'a> {
    value: HashMap<String, Object<'a>>,
    methods: Option<HashMap<&'a str, Rc<FunctionObject<'a>>>>,
}

impl<'a> TableObject<'a> {
    pub fn new(value: HashMap<String, Object<'a>>) -> Self {
        Self {
            value,
            methods: None,
        }
    }

    pub fn add_method(&mut self, name: &'a str, func: FunctionObject<'a>) {
        if let Some(methods) = &mut self.methods {
            methods.insert(name, Rc::new(func));
        } else {
            let mut methods = HashMap::new();
            methods.insert(name, Rc::new(func));
            self.methods = Some(methods);
        }
    }

    pub fn get_method(&self, name: &'a str) -> Option<Rc<FunctionObject<'a>>> {
        if let Some(methods) = &self.methods {
            methods.get(name).map(Rc::clone)
        } else {
            None
        }
    }
}

impl<'a> Deref for TableObject<'a> {
    type Target = HashMap<String, Object<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a> DerefMut for TableObject<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub fn run_table_default_method<'a>(
    table: Rc<RefCell<TableObject<'a>>>,
    name: &'a str,
    args: Vec<Object<'a>>,
) -> Result<Object<'a>, String> {
    match name {
        "keys" => {
            if !args.is_empty() {
                return Err(format!("expected 0 arguments, got {}", args.len()));
            }
            let array =
                ArrayObject::new(table.borrow().keys().cloned().map(Object::String).collect());
            Ok(Object::new_array(array))
        }
        "values" => {
            if !args.is_empty() {
                return Err(format!("expected 0 arguments, got {}", args.len()));
            }
            let array = ArrayObject::new(table.borrow().values().cloned().collect());
            Ok(Object::new_array(array))
        }
        "len" => {
            if !args.is_empty() {
                return Err(format!("expected 0 arguments, got {}", args.len()));
            }
            Ok(Object::Int(table.borrow().len() as i64))
        }
        "contains" => {
            if args.len() != 1 {
                return Err(format!("expected 1 argument, got {}", args.len()));
            }
            let key = if let Object::String(key) = &args[0] {
                key
            } else {
                return Err(format!("expected string, got {:?}", args[0]));
            };
            Ok(Object::Bool(table.borrow().contains_key(key)))
        }
        "remove" => {
            if args.len() != 1 {
                return Err(format!("expected 1 argument, got {}", args.len()));
            }
            let key = if let Object::String(key) = &args[0] {
                key
            } else {
                return Err(format!("expected string, got {:?}", args[0]));
            };
            Ok(table.borrow_mut().remove(key).unwrap_or(Object::Nil))
        }
        _ => Err(format!("table has no method {}", name)),
    }
}
