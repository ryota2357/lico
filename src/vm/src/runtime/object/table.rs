use super::*;
use std::{
    borrow::Cow,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Debug, PartialEq)]
pub struct TableObject {
    value: HashMap<Cow<'static, str>, Object>,
    methods: Option<HashMap<Cow<'static, str>, TableMethod>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TableMethod {
    #[allow(clippy::type_complexity)]
    Builtin(fn(Rc<RefCell<TableObject>>, &[Object]) -> Result<Object, String>),
    Custom(Rc<FunctionObject>),
    CustomNoSelf(Rc<FunctionObject>),
}

impl TableObject {
    pub fn new(value: HashMap<Cow<'static, str>, Object>) -> Self {
        Self {
            value,
            methods: None,
        }
    }

    pub fn deep_clone(&self) -> Self {
        let value = self
            .value
            .iter()
            .map(|(k, v)| (k.clone(), v.deep_clone()))
            .collect();
        let methods = self.methods.as_ref().map(|methods| {
            methods
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        });
        Self { value, methods }
    }

    pub fn add_method(&mut self, name: impl Into<Cow<'static, str>>, func: impl Into<TableMethod>) {
        if let Some(methods) = &mut self.methods {
            methods.insert(name.into(), func.into());
        } else {
            let mut methods = HashMap::new();
            methods.insert(name.into(), func.into());
            self.methods = Some(methods);
        }
    }

    pub fn get_method(&self, name: &str) -> Option<TableMethod> {
        if let Some(methods) = &self.methods {
            methods.get(name).map(|f| match f {
                TableMethod::Builtin(f) => TableMethod::Builtin(*f),
                TableMethod::Custom(f) => TableMethod::Custom(Rc::clone(f)),
                TableMethod::CustomNoSelf(f) => TableMethod::CustomNoSelf(Rc::clone(f)),
            })
        } else {
            None
        }
    }
}

impl From<FunctionObject> for TableMethod {
    fn from(func: FunctionObject) -> Self {
        Self::Custom(Rc::new(func))
    }
}

impl From<fn(Rc<RefCell<TableObject>>, &[Object]) -> Result<Object, String>> for TableMethod {
    fn from(func: fn(Rc<RefCell<TableObject>>, &[Object]) -> Result<Object, String>) -> Self {
        Self::Builtin(func)
    }
}

impl Deref for TableObject {
    type Target = HashMap<Cow<'static, str>, Object>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for TableObject {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub fn run_table_default_method(
    table: Rc<RefCell<TableObject>>,
    name: &str,
    args: &[Object],
) -> Result<Object, String> {
    match name {
        // keys() -> Array
        "keys" => {
            extract_argument!(args, []);
            let keys = table
                .borrow()
                .keys()
                .cloned()
                .map(|s| Object::new_string(s.to_string()))
                .collect();
            let array = ArrayObject::new(keys);
            Ok(Object::new_array(array))
        }

        // values() -> Array
        "values" => {
            extract_argument!(args, []);
            let values = table.borrow().values().cloned().collect();
            let array = ArrayObject::new(values);
            Ok(Object::new_array(array))
        }

        // len() -> Int
        "len" => {
            extract_argument!(args, []);
            Ok(Object::Int(table.borrow().len() as i64))
        }

        // contains(key: String) -> Bool
        "contains" => {
            let key = extract_argument!(args, [String]);
            Ok(Object::Bool(table.borrow().contains_key(key.as_str())))
        }

        // remove(key: String) -> Any
        "remove" => {
            let key = extract_argument!(args, [String]);
            Ok(table
                .borrow_mut()
                .remove(key.as_str())
                .unwrap_or(Object::Nil))
        }
        _ => Err(format!("table has no method {}", name)),
    }
}
