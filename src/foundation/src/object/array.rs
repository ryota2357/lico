use super::{pms::*, private::TObject, Object};

use core::{
    alloc::{Allocator, Layout},
    cell::Cell,
    marker::PhantomData,
    mem::forget,
    ptr,
    ptr::NonNull,
};
use std::alloc::Global;

#[derive(Debug)]
pub struct Array<T: TObject = Object> {
    ptr: NonNull<Inner<T>>,
    phantom: PhantomData<Inner<T>>,
}
impl<T: TObject> HasPmsInner<Inner<T>> for Array<T> {
    fn ptr(&self) -> NonNull<Inner<T>> {
        self.ptr
    }
    unsafe fn iter_inner_children_mut(&mut self) -> impl Iterator<Item = &mut Object> {
        self.inner_mut().data.iter_mut().map(|x| x.as_object_mut())
    }
}

pub struct Inner<T: TObject> {
    data: Vec<T>,
    version: u64,
    _ref_count: Cell<usize>,
    _color: Cell<Color>,
}
impl<T: TObject> PmsInner for Inner<T> {
    fn ref_count_ref(&self) -> &Cell<usize> {
        &self._ref_count
    }
    fn color_ref(&self) -> &Cell<Color> {
        &self._color
    }
}

impl<T: TObject> Array<T> {
    pub fn new() -> Self {
        Array::from(Vec::new())
    }
    fn inner_mut(&mut self) -> &mut Inner<T> {
        unsafe {
            let inner = HasPmsInner::inner_mut(self);
            inner.version += 1;
            inner
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner().data.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.inner_mut().data.iter_mut()
    }
}

impl Array {
    pub fn len(&self) -> usize {
        self.inner().data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn version(&self) -> u64 {
        self.inner().version
    }

    pub fn get(&self, index: usize) -> Option<Object> {
        self.inner().data.get(index).cloned()
    }
    pub fn set(&mut self, index: usize, value: Object) {
        self.inner_mut().data[index] = value;
    }

    pub fn push(&mut self, value: Object) {
        self.inner_mut().data.push(value);
    }
    pub fn pop(&mut self) -> Option<Object> {
        self.inner_mut().data.pop()
    }
    pub fn insert(&mut self, index: usize, element: Object) {
        self.inner_mut().data.insert(index, element);
    }
    pub fn remove(&mut self, index: usize) -> Object {
        self.inner_mut().data.remove(index)
    }
    pub fn clear(&mut self) {
        self.inner_mut().data.clear();
    }

    pub fn contains(&self, value: &Object) -> bool {
        self.inner().data.contains(value)
    }
}

impl<T: TObject> Default for Array<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: TObject> From<Vec<T>> for Array<T> {
    fn from(array: Vec<T>) -> Self {
        let ptr = Box::leak(Box::new(Inner {
            data: array,
            version: 0,
            _ref_count: Cell::new(1),
            _color: Cell::new(Color::Black),
        }));
        Array {
            ptr: NonNull::from(ptr),
            phantom: PhantomData,
        }
    }
}

impl<T: TObject> Clone for Array<T> {
    fn clone(&self) -> Self {
        self.inner().inc_ref_count();
        Self {
            ptr: self.ptr,
            phantom: PhantomData,
        }
    }
}

impl<T: TObject> PartialEq for Array<T> {
    fn eq(&self, other: &Self) -> bool {
        // We can't use pointer for `eq` since the `Object` is `PartialEq` and not `Eq`.
        // Even if pointers are same, `false` must be returned when they contain (point) data for which the reflection rule does not hold.
        self.inner().data.eq(&other.inner().data)
    }
}

unsafe fn deallocate_inner<T: TObject>(array: &mut Array<T>) {
    assert!(array.inner().ref_count() == 0);

    let inner = ptr::read(array.ptr.as_ptr());
    for next in inner.data.into_iter() {
        match next.into_object() {
            Object::Int(_) => {}          // No need to drop for `i64`  (Copy type)
            Object::Float(_) => {}        // No need to drop for `f64`  (Copy type)
            Object::Bool(_) => {}         // No need to drop for `bool` (Copy type)
            Object::Nil => {}             // No need to drop for `()`   (Nothing to drop)
            Object::RustFunction(_) => {} // No need to drop for `fn`   (Copy type)
            Object::String(next) => {
                drop(next);
            }
            Object::Function(next) => {
                drop(next);
            }
            Object::Array(next) => {
                forget(next);
            }
            Object::Table(_) => {
                todo!();
            }
        }
    }
    Global.deallocate(array.ptr.cast(), Layout::for_value(array.ptr.as_ref()));
    array.ptr = NonNull::new_unchecked(usize::MAX as *mut _);
}

unsafe impl<#[may_dangle] T: TObject> Drop for Array<T> {
    fn drop(&mut self) {
        RecursiveDropGuard::begin_drop();

        match self.inner().color() {
            Color::Black => {
                // First, decrement the reference count.
                self.inner().dec_ref_count();

                // If the reference count is not zero, we can't drop this object.
                // But there is a possibility that this object is a part of a cycle, so we need to do `mark_and_sweep` from this object.
                if self.inner().ref_count() > 0 {
                    unsafe {
                        self.inner().paint(Color::Purple); // Mark as suspicious of cycle reference
                        mark_and_sweep(self);
                    }
                    return;
                }

                // If the reference count is zero, we can drop this object.
                unsafe {
                    struct CollectPurple<'a>(Vec<&'a mut Array>);
                    impl<'a> CollectPurple<'a> {
                        unsafe fn collect<T: TObject + 'a>(&mut self, mut ptr: NonNull<Inner<T>>) {
                            assert!(ptr.as_ref().ref_count() == 0);
                            for next in ptr.as_mut().data.iter_mut() {
                                match next.as_object_mut() {
                                    Object::Array(array) => {
                                        array.inner().dec_ref_count();
                                        if array.inner().ref_count() == 0 {
                                            array.inner().paint(Color::White);
                                            self.collect(array.ptr);
                                            deallocate_inner(array);
                                        } else {
                                            if array.inner().color() == Color::Purple {
                                                continue;
                                            }
                                            array.inner().paint(Color::Purple);
                                            self.0.push(array);
                                        }
                                    }
                                    Object::Table(_) => todo!(),
                                    _ => {}
                                }
                            }
                        }
                    }
                    let mut collect_purple = CollectPurple(Vec::new());
                    collect_purple.collect(self.ptr);
                    deallocate_inner(self);
                    // mark_and_sweep(&mut collect_purple.0);
                }
            }
            Color::Purple | Color::Gray | Color::White => {
                unreachable!("drop() is called during mark and sweep")
            }
        }

        RecursiveDropGuard::end_drop();
    }
}

// mod mark_and_sweep {
//     use super::*;
//
//     pub unsafe fn mark_and_sweep<T: TObject>(item: &mut [&mut Array<T>]) {
//         for item in item.iter_mut() {
//             if item.inner().color() != Color::Purple {
//                 return;
//             }
//             paint_gray(item);
//             scan_gray(item);
//             collect_white(item);
//         }
//     }
//
//     unsafe fn paint_gray<T: TObject>(item: &mut Array<T>) {
//         if item.inner().color() == Color::Gray {
//             return;
//         }
//         item.inner().paint(Color::Gray);
//         for next in item.inner_mut().data.iter_mut() {
//             match next.as_object_mut() {
//                 Object::Array(array) => {
//                     array.inner().dec_ref_count();
//                     paint_gray(array);
//                 }
//                 Object::Table(_) => todo!(),
//                 _ => {}
//             }
//         }
//     }
//
//     unsafe fn scan_gray<T: TObject>(item: &mut Array<T>) {
//         if item.inner().color() != Color::Gray {
//             return;
//         }
//         let ref_count = item.inner().ref_count();
//         if ref_count == 0 {
//             item.inner().paint(Color::White);
//             for next in item.inner_mut().data.iter_mut() {
//                 match next.as_object_mut() {
//                     Object::Array(array) => scan_gray(array),
//                     Object::Table(_) => todo!(),
//                     _ => {}
//                 }
//             }
//         } else {
//             paint_black(item);
//         }
//     }
//
//     unsafe fn paint_black<T: TObject>(item: &mut Array<T>) {
//         if item.inner().color() == Color::Black {
//             return;
//         }
//         item.inner().paint(Color::Black);
//         for next in item.inner_mut().data.iter_mut() {
//             match next.as_object_mut() {
//                 Object::Array(array) => {
//                     array.inner().inc_ref_count();
//                     paint_black(array);
//                 }
//                 Object::Table(_) => todo!(),
//                 _ => {}
//             }
//         }
//     }
//
//     unsafe fn collect_white<T: TObject>(item: &mut Array<T>) {
//         if item.inner().color() != Color::White {
//             return;
//         }
//         item.inner().paint(Color::Black);
//         for next in item.inner_mut().data.iter_mut() {
//             match next.as_object_mut() {
//                 Object::Array(array) => collect_white(array),
//                 Object::Table(_) => todo!(),
//                 _ => {}
//             }
//         }
//         Global.deallocate(item.ptr.cast(), Layout::for_value(item.ptr.as_ref()));
//         item.ptr = NonNull::new_unchecked(usize::MAX as *mut _);
//     }
// }
