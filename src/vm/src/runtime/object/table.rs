use super::*;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Debug, PartialEq)]
pub struct TableObject<'a> {
    value: HashMap<String, Object<'a>>,
    methods: Option<HashMap<&'a str, TableMethod<'a>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TableMethod<'a> {
    #[allow(clippy::type_complexity)]
    Builtin(fn(Rc<RefCell<TableObject<'a>>>, Vec<Object<'a>>) -> Result<Object<'a>, String>),
    Custom(Rc<FunctionObject<'a>>),
}

impl<'a> TableObject<'a> {
    pub fn new(value: HashMap<String, Object<'a>>) -> Self {
        Self {
            value,
            methods: None,
        }
    }

    pub fn add_method(&mut self, name: &'a str, func: impl Into<TableMethod<'a>>) {
        if let Some(methods) = &mut self.methods {
            methods.insert(name, func.into());
        } else {
            let mut methods = HashMap::new();
            methods.insert(name, func.into());
            self.methods = Some(methods);
        }
    }

    pub fn get_method(&self, name: &'a str) -> Option<TableMethod<'a>> {
        if let Some(methods) = &self.methods {
            methods.get(name).map(|f| match f {
                TableMethod::Builtin(f) => TableMethod::Builtin(*f),
                TableMethod::Custom(f) => TableMethod::Custom(Rc::clone(f)),
            })
        } else {
            None
        }
    }
}

impl<'a> From<FunctionObject<'a>> for TableMethod<'a> {
    fn from(func: FunctionObject<'a>) -> Self {
        Self::Custom(Rc::new(func))
    }
}

impl<'a> From<fn(Rc<RefCell<TableObject<'a>>>, Vec<Object<'a>>) -> Result<Object<'a>, String>>
    for TableMethod<'a>
{
    fn from(
        func: fn(Rc<RefCell<TableObject<'a>>>, Vec<Object<'a>>) -> Result<Object<'a>, String>,
    ) -> Self {
        Self::Builtin(func)
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
