use super::*;

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
