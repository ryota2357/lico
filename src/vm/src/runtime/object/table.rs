use super::*;

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

macro_rules! split_arguments {
    ($args:expr, $len:expr) => {{
        if $args.len() != ($len + 1) {
            return Err(format!(
                "expected {} arguments, got {}",
                $len,
                $args.len() - 1
            ));
        }
        let table = if let Object::Table(table) = &$args[0] {
            table
        } else {
            unreachable!()
        };
        let rest = &$args[1..];
        (table, rest)
    }};
}

pub fn run_table_default_method<'a>(
    name: &'a str,
    args: &[Object<'a>],
) -> Result<Object<'a>, String> {
    match name {
        "keys" => {
            let (table, _) = split_arguments!(args, 0);
            let array = table.borrow().keys().cloned().map(Object::String).collect();
            Ok(Object::new_array(array))
        }
        "values" => {
            let (table, _) = split_arguments!(args, 0);
            let array = table.borrow().values().cloned().collect();
            Ok(Object::new_array(array))
        }
        "len" => {
            let (table, _) = split_arguments!(args, 0);
            Ok(Object::Int(table.borrow().len() as i64))
        }
        "contains" => {
            let (table, args) = split_arguments!(args, 1);
            let key = if let Object::String(key) = &args[1] {
                key
            } else {
                return Err(format!("expected string, got {:?}", args[1]));
            };
            Ok(Object::Bool(table.borrow().contains_key(key)))
        }
        "remove" => {
            let (table, args) = split_arguments!(args, 1);
            let key = if let Object::String(key) = &args[1] {
                key
            } else {
                return Err(format!("expected string, got {:?}", args[1]));
            };
            Ok(table.borrow_mut().remove(key).unwrap_or(Object::Nil))
        }
        _ => Err(format!("table has no method {}", name)),
    }
}
