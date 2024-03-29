use super::*;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq)]
pub struct ArrayObject {
    value: Vec<Object>,
    version: u64,
}

impl ArrayObject {
    pub fn new(array: Vec<Object>) -> Self {
        Self {
            value: array,
            version: 0,
        }
    }

    pub fn deep_clone(&self) -> Self {
        Self {
            value: self.value.iter().map(|x| x.deep_clone()).collect(),
            version: 0,
        }
    }
}

impl Deref for ArrayObject {
    type Target = Vec<Object>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for ArrayObject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.version += 1;
        &mut self.value
    }
}

pub fn run_array_method(
    array: Rc<RefCell<ArrayObject>>,
    name: &str,
    args: &[Object],
) -> Result<Object, String> {
    match name {
        // __get_iterator() -> Table
        "__get_iterator" => {
            extract_argument!(args, []);
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
                    ("__array".into(), Object::Array(Rc::clone(&array))),
                    (
                        "__version".into(),
                        Object::Int(array.borrow().version as i64),
                    ),
                    ("__index".into(), Object::Int(-1)),
                    ("__current".into(), Object::Nil),
                ]
                .into_iter()
                .collect(),
            );
            iter_tbl.add_method(
                "__move_next", // __move_next() -> Bool
                TableMethod::Builtin(|iter: Rc<RefCell<TableObject>>, args| {
                    extract_argument!(args, []);
                    let (array, version, index) = table_extract_values!(iter, {
                        __array: Array, __version: Int, __index: Int,
                    });
                    if version != array.borrow().version as i64 {
                        return Err("array modified during iteration".to_string());
                    }
                    if index + 1 < array.borrow().len() as i64 {
                        iter.borrow_mut()
                            .insert("__index".into(), Object::Int(index + 1));
                        iter.borrow_mut().insert(
                            "__current".into(),
                            array.borrow()[(index + 1) as usize].clone(),
                        );
                        Ok(Object::Bool(true))
                    } else {
                        iter.borrow_mut().insert("__current".into(), Object::Nil);
                        Ok(Object::Bool(false))
                    }
                }),
            );
            iter_tbl.add_method(
                "__current", // __current() -> Object
                TableMethod::Builtin(|iter, args| {
                    extract_argument!(args, []);
                    let current = iter.borrow().get("__current").cloned();
                    Ok(current.unwrap_or(Object::Nil))
                }),
            );

            Ok(Object::new_table(iter_tbl))
        }

        // len() -> Int
        "len" => {
            extract_argument!(args, []);
            Ok(Object::Int(array.borrow().len() as i64))
        }

        // push(value: Object) -> Nil
        "push" => {
            let value = extract_argument!(args, [{ x => x.clone() }]);
            array.borrow_mut().push(value);
            Ok(Object::Nil)
        }

        // pop() -> Object
        "pop" => Ok(array.borrow_mut().pop().unwrap_or(Object::Nil)),

        _ => Err(format!("array has no method {}", name)),
    }
}
