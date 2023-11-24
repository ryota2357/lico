use std::{cell::RefCell, fmt::Display, rc::Rc};

#[macro_use]
mod macros;

mod array;
pub use array::*;

mod function;
pub use function::*;

mod table;
pub use table::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Object<'a> {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Nil,
    Function(Rc<FunctionObject<'a>>),
    Array(Rc<RefCell<ArrayObject<'a>>>),
    Table(Rc<RefCell<TableObject<'a>>>),
    RustFunction(fn(&[Object<'a>]) -> Result<Object<'a>, String>),
}

macro_rules! ensure_fn {
    ($name:ident -> $inner_type:ty, $pattern:pat => $result:expr) => {
        pub fn $name(self) -> Result<$inner_type, String> {
            match self {
                $pattern => $result,
                _ => Err(format!(
                    "Expected `{}`, got `{}`",
                    stringify!($name)[7..].to_string(), // remove "ensure_"
                    self.typename()
                )),
            }
        }
    };
}

impl<'a> Object<'a> {
    pub fn new_function(func: FunctionObject<'a>) -> Self {
        Self::Function(Rc::new(func))
    }

    pub fn new_array(array: ArrayObject<'a>) -> Self {
        Self::Array(Rc::new(RefCell::new(array)))
    }

    pub fn new_table(table: TableObject<'a>) -> Self {
        Self::Table(Rc::new(RefCell::new(table)))
    }

    pub fn typename(&self) -> &'static str {
        match self {
            Object::Int(_) => "int",
            Object::Float(_) => "float",
            Object::String(_) => "string",
            Object::Bool(_) => "bool",
            Object::Nil => "nil",
            Object::Function(_) => "function",
            Object::Array(_) => "array",
            Object::Table(_) => "table",
            Object::RustFunction(_) => "rust_function",
        }
    }

    ensure_fn!(
        ensure_int -> i64,
        Object::Int(x) => Ok(x)
    );
    ensure_fn!(
        ensure_float -> f64,
        Object::Float(x) => Ok(x)
    );
    ensure_fn!(
        ensure_string -> String,
        Object::String(x) => Ok(x)
    );
    ensure_fn!(
        ensure_bool -> bool,
        Object::Bool(x) => Ok(x)
    );
    ensure_fn!(
        ensure_function -> Rc<FunctionObject<'a>>,
        Object::Function(x) => Ok(x)
    );
    ensure_fn!(
        ensure_array -> Rc<RefCell<ArrayObject<'a>>>,
        Object::Array(x) => Ok(x)
    );
    ensure_fn!(
        ensure_table -> Rc<RefCell<TableObject<'a>>>,
        Object::Table(x) => Ok(x)
    );
}

impl Display for Object<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Int(x) => write!(f, "{}", x),
            Object::Float(x) => write!(f, "{}", x),
            Object::String(x) => write!(f, "{}", x),
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
                    .map(|x| match x {
                        Object::String(x) => {
                            let x = x
                                .replace('\\', "\\\\")
                                .replace('\n', "\\n")
                                .replace('\r', "\\r")
                                .replace('\t', "\\t")
                                .replace('\0', "\\0");
                            let has_single_quote = x.contains('\'');
                            let has_double_quote = x.contains('"');
                            match (has_single_quote, has_double_quote) {
                                (true, true) => format!("\"{}\"", x.replace('\"', "\\\"")),
                                (_, false) => format!("\"{}\"", x),
                                (false, _) => format!("'{}'", x),
                            }
                        }
                        _ => format!("{}", x),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if array.len() > 10 {
                    format!("{}, ...and more {} items", content, array.len() - 10)
                } else {
                    content
                }
            }),
            Object::Table(x) => write!(f, "<Table ({} fields)>", x.borrow().len(),),
            Object::RustFunction(x) => write!(f, "<RustFunction:{:?}>", x),
        }
    }
}
