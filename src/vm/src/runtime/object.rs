use std::{cell::RefCell, fmt::Display, rc::Rc};

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

impl<'a> Object<'a> {
    pub fn new_function(func: FunctionObject<'a>) -> Self {
        Self::Function(Rc::new(func))
    }
    pub fn new_array(array: Vec<Object<'a>>) -> Self {
        Self::Array(Rc::new(RefCell::new(ArrayObject::new(array))))
    }
    pub fn new_table(table: TableObject<'a>) -> Self {
        Self::Table(Rc::new(RefCell::new(table)))
    }
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
