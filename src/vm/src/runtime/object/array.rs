use super::*;
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
            ensure_argument_length!(args, 0);
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
            let mut iter_tbl = TableObject::new(
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
            iter_tbl.add_method(
                "__move_next",
                TableMethod::Builtin(|iter: Rc<RefCell<TableObject<'a>>>, args| {
                    ensure_argument_length!(args, 0);
                    let (array, version, index) = table_extract_values!(iter, {
                        __array: Array, __version: Int, __index: Int,
                    });
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
            );
            iter_tbl.add_method(
                "__current",
                TableMethod::Builtin(|iter, args| {
                    ensure_argument_length!(args, 0);
                    let current = iter.borrow().get("__current").cloned();
                    Ok(current.unwrap_or(Object::Nil))
                }),
            );

            Ok(Object::new_table(iter_tbl))
        }
        "len" => {
            ensure_argument_length!(args, 0);
            Ok(Object::Int(array.borrow().len() as i64))
        }
        "push" => {
            ensure_argument_length!(args, 1);
            array.borrow_mut().push(args[0].clone());
            Ok(Object::Nil)
        }
        "pop" => {
            ensure_argument_length!(args, 0);
            Ok(array.borrow_mut().pop().unwrap_or(Object::Nil))
        }
        _ => Err(format!("array has no method {}", name)),
    }
}
