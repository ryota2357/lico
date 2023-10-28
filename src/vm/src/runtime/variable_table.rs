use super::*;
use std::{cell::RefCell, rc::Rc};

#[derive(Default, Clone, Debug, PartialEq)]
pub struct VariableTable<'a> {
    scopes: Vec<internal::Scope<'a>>,
}

impl<'a> VariableTable<'a> {
    pub fn new() -> Self {
        Self {
            scopes: vec![internal::Scope::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(internal::Scope::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes
            .pop()
            .expect("[INTERNAL] Cannot pop scope because there are no scopes.");
    }

    pub fn insert(&mut self, name: &'a str, object: Object<'a>) {
        self.scopes
            .last_mut()
            .expect("[INTERNAL] Cannot insert variable because there are no scopes.")
            .push(name, internal::Entity::Value(object));
    }

    pub fn insert_ref(&mut self, name: &'a str, ref_object: Rc<RefCell<Object<'a>>>) {
        self.scopes
            .last_mut()
            .expect("[INTERNAL] Cannot insert variable because there are no scopes.")
            .push(name, internal::Entity::Shared(ref_object));
    }

    pub fn erase(&mut self, count: usize) {
        self.scopes
            .last_mut()
            .expect("[INTERNAL] Cannot erase variables because there are no scopes.")
            .drop(count);
    }

    pub fn edit(&mut self, name: &'a str, object: Object<'a>) -> Result<(), String> {
        self.scopes
            .last_mut()
            .expect("[INTERNAL] Cannot edit variable because there are no scopes.")
            .edit(name, object)
    }

    pub fn get(&self, name: &'a str) -> Option<Object<'a>> {
        self.scopes
            .last()
            .expect("[INTERNAL] Cannot get variable because there are no scopes.")
            .get(name)
    }

    pub fn get_ref(&mut self, name: &'a str) -> Option<Rc<RefCell<Object<'a>>>> {
        self.scopes
            .last_mut()
            .expect("[INTERNAL] Cannot get variable because there are no scopes.")
            .get_ref(name)
    }

    pub fn dump(&self, indent: usize) {
        println!("{}[VariableTable]", " ".repeat(indent));
        for scope in self.scopes.iter() {
            scope.dump(indent + 2);
        }
    }
}

mod internal {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Entity<'a> {
        Value(Object<'a>),
        Shared(Rc<RefCell<Object<'a>>>),
    }

    #[derive(Default, Clone, Debug, PartialEq)]
    pub struct Scope<'a> {
        names: Vec<&'a str>,
        entities: Vec<Entity<'a>>,
    }

    impl<'a> Scope<'a> {
        pub fn new() -> Self {
            Self {
                names: vec![],
                entities: vec![],
            }
        }

        pub fn push(&mut self, name: &'a str, entity: Entity<'a>) {
            self.names.push(name);
            self.entities.push(entity);
        }

        pub fn drop(&mut self, count: usize) {
            if count > self.names.len() {
                panic!(
                    "[INTERNAL] Cannot drop {} variables because there are only {} variables in scope.",
                    count,
                    self.names.len()
                );
            }
            self.names.truncate(self.names.len() - count);
            self.entities.truncate(self.entities.len() - count);
        }

        pub fn get(&self, name: &'a str) -> Option<Object<'a>> {
            match self._search(name) {
                Some(index) => match &self.entities[index] {
                    Entity::Value(object) => Some(object.clone()),
                    Entity::Shared(object) => Some(object.borrow().clone()),
                },
                None => None,
            }
        }

        pub fn get_ref(&mut self, name: &'a str) -> Option<Rc<RefCell<Object<'a>>>> {
            match self._search(name) {
                Some(index) => match &self.entities[index] {
                    Entity::Value(object) => {
                        let res = Rc::new(RefCell::new(object.clone()));
                        self.entities[index] = Entity::Shared(Rc::clone(&res));
                        Some(res)
                    }
                    Entity::Shared(object) => Some(Rc::clone(object)),
                },
                None => None,
            }
        }

        pub fn edit(&mut self, name: &'a str, object: Object<'a>) -> Result<(), String> {
            match self._search(name) {
                Some(index) => match &self.entities[index] {
                    Entity::Value(_) => {
                        self.entities[index] = Entity::Value(object);
                        Ok(())
                    }
                    Entity::Shared(entity) => {
                        *(entity.borrow_mut()) = object;
                        Ok(())
                    }
                },
                None => Err(format!(
                    "Cannot edit variable '{}' because it does not exist.",
                    name
                )),
            }
        }

        fn _search(&self, name: &'a str) -> Option<usize> {
            self.names
                .iter()
                .rev()
                .position(|n| *n == name)
                .map(|index| self.names.len() - index - 1)
        }

        pub fn dump(&self, indent: usize) {
            println!("{}[Scope]", " ".repeat(indent));
            for (name, entity) in self.names.iter().zip(self.entities.iter()) {
                println!("{}{name}: {entity:?}", " ".repeat(indent + 2));
            }
        }
    }
}
