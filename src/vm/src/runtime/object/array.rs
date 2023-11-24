use super::*;
use crate::code::Code;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayObject<'a> {
    value: Vec<Object<'a>>,
    version: u64,
}

impl<'a> ArrayObject<'a> {
    pub fn new(array: Vec<Object<'a>>) -> Self {
        Self {
            value: array,
            version: 0,
        }
    }
}

impl<'a> Deref for ArrayObject<'a> {
    type Target = Vec<Object<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a> DerefMut for ArrayObject<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.version += 1;
        &mut self.value
    }
}

pub fn run_array_method<'a>(
    array: Rc<RefCell<ArrayObject<'a>>>,
    name: &'a str,
    args: Vec<Object<'a>>,
) -> Result<Object<'a>, String> {
    match name {
        "__get_iterator" => {
            if !args.is_empty() {
                return Err(format!("expected 0 arguments, got {}", args.len()));
            }
            // iter = {
            //     __array = array,
            //     __version = array.version,
            //     __index = -1,
            //     __current = Nil,
            //     func __move_next = func()
            //        @rustFunction {
            //            if version != array.version { Err(..)}
            //            if index + 1 < array.len() {
            //                index += 1
            //                current = array[index]
            //                Ok(true)
            //            } else {
            //                current = Nil
            //                Ok(false)
            //            }
            //        }
            //     end,
            //     func __current = func()
            //         self.__current
            //     end,
            // }
            let mut iter = TableObject::new(
                [
                    ("__array".to_string(), Object::Array(Rc::clone(&array))),
                    (
                        "__version".to_string(),
                        Object::Int(array.borrow().version as i64),
                    ),
                    ("__index".to_string(), Object::Int(-1)),
                    ("__current".to_string(), Object::Nil),
                ]
                .into_iter()
                .collect(),
            );
            iter.add_method(
                "__move_next",
                FunctionObject {
                    id: (0, 0),
                    env: vec![],
                    args: vec!["self"],
                    code: vec![
                        Code::LoadRustFunction(|obj| {
                            if obj.len() != 1 {
                                return Err(format!("expected 0 argument, got {}", obj.len() - 1));
                            }

                            let iter = if let Object::Table(iter) = &obj[0] {
                                iter
                            } else {
                                unreachable!()
                            };
                            let (array, version, index) = if let (
                                Some(Object::Array(array)),
                                Some(Object::Int(version)),
                                Some(Object::Int(index)),
                            ) = (
                                iter.borrow().get("__array"),
                                iter.borrow().get("__version"),
                                iter.borrow().get("__index"),
                            ) {
                                (Rc::clone(array), *version, *index)
                            } else {
                                unreachable!()
                            };

                            if version != array.borrow().version as i64 {
                                return Err("array modified during iteration".to_string());
                            }
                            if index + 1 < array.borrow().len() as i64 {
                                iter.borrow_mut()
                                    .insert("__index".to_string(), Object::Int(index + 1));
                                iter.borrow_mut().insert(
                                    "__current".to_string(),
                                    array.borrow()[(index + 1) as usize].clone(),
                                );
                                Ok(Object::Bool(true))
                            } else {
                                iter.borrow_mut()
                                    .insert("__current".to_string(), Object::Nil);
                                Ok(Object::Bool(false))
                            }
                        }),
                        Code::LoadLocal("self"),
                        Code::Call(1),
                        Code::Return,
                    ],
                },
            );
            iter.add_method(
                "__current",
                FunctionObject {
                    id: (0, 0),
                    env: vec![],
                    args: vec!["self"],
                    code: vec![
                        Code::LoadLocal("self"),
                        Code::LoadStringAsRef("__current"),
                        Code::GetItem,
                        Code::Return,
                    ],
                },
            );

            Ok(Object::new_table(iter))
        }
        "len" => {
            if !args.is_empty() {
                return Err(format!("expected 0 arguments, got {}", args.len()));
            }
            Ok(Object::Int(array.borrow().len() as i64))
        }
        "push" => {
            if args.len() != 1 {
                return Err(format!("expected 1 argument, got {}", args.len()));
            }
            array.borrow_mut().push(args[0].clone());
            Ok(Object::Nil)
        }
        "pop" => {
            if !args.is_empty() {
                return Err(format!("expected 0 arguments, got {}", args.len()));
            }
            Ok(array.borrow_mut().pop().unwrap_or(Object::Nil))
        }
        _ => Err(format!("array has no method {}", name)),
    }
}
