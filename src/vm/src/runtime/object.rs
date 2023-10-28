use crate::code::Code;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
    rc::Rc,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Object<'a> {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
    Function(Rc<FunctionObject<'a>>),
    Array(Rc<RefCell<Vec<Object<'a>>>>),
    Table(Rc<RefCell<TableObject<'a>>>),
}

impl Display for Object<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Int(x) => write!(f, "{}", x),
            Object::Float(x) => write!(f, "{}", x),
            Object::String(x) => write!(f, "\"{}\"", x.escape_debug()),
            Object::Bool(x) => write!(f, "{}", if *x { "true" } else { "false" }),
            Object::Nil => write!(f, "nil"),
            Object::Function(x) => {
                write!(
                    f,
                    "<Function:{}-{} ({})>",
                    x.id.0,
                    x.id.1,
                    x.args.join(", ")
                )
            }
            Object::Array(x) => write!(f, "[{}]", {
                let array = x.borrow();
                let content = array
                    .iter()
                    .take(10)
                    .map(|x| format!("{}", x))
                    .collect::<Vec<_>>()
                    .join(", ");
                if array.len() > 10 {
                    format!("{}, ...and more {} items", content, array.len() - 10)
                } else {
                    content
                }
            }),
            Object::Table(x) => write!(f, "<Table ({} fields)>", x.borrow().len(),),
        }
    }
}

impl<'a> Object<'a> {
    pub fn new_function(func: FunctionObject<'a>) -> Self {
        Self::Function(Rc::new(func))
    }
    pub fn new_array(array: Vec<Object<'a>>) -> Self {
        Self::Array(Rc::new(RefCell::new(array)))
    }
    pub fn new_table(table: TableObject<'a>) -> Self {
        Self::Table(Rc::new(RefCell::new(table)))
    }
}

#[derive(Clone, Debug)]
pub struct FunctionObject<'a> {
    pub id: (usize, u8),
    pub env: Vec<(&'a str, Option<Rc<RefCell<Object<'a>>>>)>,
    pub args: Vec<&'a str>,
    pub code: Vec<Code<'a>>,
}

impl PartialEq for FunctionObject<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TableObject<'a> {
    value: HashMap<String, Object<'a>>,
    methods: Option<HashMap<&'a str, FunctionObject<'a>>>,
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
            methods.insert(name, func);
        } else {
            let mut methods = HashMap::new();
            methods.insert(name, func);
            self.methods = Some(methods);
        }
    }
    pub fn get_method(&self, name: &str) -> Option<&FunctionObject<'a>> {
        if let Some(methods) = &self.methods {
            methods.get(name)
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
