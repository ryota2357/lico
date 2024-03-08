use super::{Array, Object, Table};
use core::{
    alloc::{Allocator, Layout},
    cell::Cell,
    fmt::Debug,
    mem::forget,
    ptr,
    ptr::NonNull,
};
use std::alloc::Global;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,  // Active
    Purple, // Suspicious of circular references
    Gray,   // Checking for circular references
    White,  // Candidates for collection (deallocate)
}

pub trait PmsInner {
    fn ref_count_ref(&self) -> &Cell<usize>;
    fn color_ref(&self) -> &Cell<Color>;

    unsafe fn iter_children_mut(&mut self) -> impl Iterator<Item = &mut Object>;
    unsafe fn into_iter_children(self) -> impl Iterator<Item = Object>;

    fn color(&self) -> Color {
        self.color_ref().get()
    }
    fn paint(&self, color: Color) {
        self.color_ref().set(color);
    }

    fn ref_count(&self) -> usize {
        self.ref_count_ref().get()
    }

    fn inc_ref_count(&self) {
        let ref_count = self.ref_count_ref().get();
        if ref_count == 0 || ref_count == usize::MAX {
            panic!("Reference count overflow");
        }
        self.ref_count_ref().set(ref_count + 1);
    }
    fn dec_ref_count(&self) {
        let ref_count = self.ref_count_ref().get();
        if ref_count == 0 {
            panic!("Reference count underflow");
        }
        self.ref_count_ref().set(ref_count - 1);
    }
}

pub trait PmsObject<I: PmsInner> {
    fn ptr(&self) -> NonNull<I>;
    fn ptr_mut(&mut self) -> &mut NonNull<I>;

    fn inner(&self) -> &I {
        assert!(!is_freed_ptr(self.ptr()));
        unsafe { self.ptr().as_ref() }
    }
    unsafe fn inner_mut(&mut self) -> &mut I {
        assert!(!is_freed_ptr(self.ptr()));
        self.ptr().as_mut()
    }

    unsafe fn deallocate_inner(this: &mut Self) {
        assert!(this.inner().ref_count() == 0);
        assert!(this.inner().color() == Color::White);
        assert!(!is_freed_ptr(this.ptr()));

        let inner = ptr::read(this.ptr().as_ptr());
        for next in inner.into_iter_children() {
            match next {
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
            Global.deallocate(this.ptr().cast(), Layout::for_value(this.ptr().as_ref()));
            *this.ptr_mut() = NonNull::new_unchecked(usize::MAX as *mut _);
        }
    }

    fn custom_drop(this: &mut Self) {
        RecursiveDropGuard::begin_drop();
        match this.inner().color() {
            Color::Black => {
                // First, decrement the reference count.
                this.inner().dec_ref_count();

                // If the reference count is not zero, we can't drop this object.
                // But there is a possibility that this object is a part of a cycle, so we need to do `mark_and_sweep` from this object.
                if this.inner().ref_count() > 0 {
                    unsafe {
                        this.inner().paint(Color::Purple); // Mark as suspicious of cycle reference
                        mark_and_sweep::run(this);
                    }
                    return;
                }

                // If the reference count is zero, we can drop this object.
                unsafe {
                    struct PurpleCollector<'a> {
                        array: Vec<&'a mut Array>,
                        table: Vec<&'a mut Table>,
                    }
                    impl<'a> PurpleCollector<'a> {
                        fn new() -> Self {
                            Self {
                                array: Vec::new(),
                                table: Vec::new(),
                            }
                        }
                        unsafe fn collect<I: PmsInner + 'a>(&mut self, mut ptr: NonNull<I>) {
                            unsafe {
                                assert!(ptr.as_ref().ref_count() == 0);
                                for next in ptr.as_mut().iter_children_mut() {
                                    match next {
                                        Object::Array(array) => {
                                            array.inner().dec_ref_count();
                                            if array.inner().ref_count() == 0 {
                                                array.inner().paint(Color::White);
                                                self.collect(array.ptr());
                                                PmsObject::deallocate_inner(array);
                                            } else {
                                                if array.inner().color() == Color::Purple {
                                                    continue;
                                                }
                                                array.inner().paint(Color::Purple);
                                                self.array.push(array);
                                            }
                                        }
                                        Object::Table(_) => todo!(),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    let mut purple_collector = PurpleCollector::new();
                    purple_collector.collect(this.ptr());
                    PmsObject::deallocate_inner(this);
                    for array in purple_collector.array {
                        mark_and_sweep::run(array);
                    }
                    for table in purple_collector.table {
                        mark_and_sweep::run(table);
                    }
                }
            }
            Color::Purple | Color::Gray | Color::White => {
                unreachable!("drop() is called during mark and sweep")
            }
        }
        RecursiveDropGuard::end_drop();
    }
}

thread_local! {
    static REC_DROP_GUARD: Cell<bool> = const { Cell::new(false) };
}
struct RecursiveDropGuard;
impl RecursiveDropGuard {
    pub fn begin_drop() {
        REC_DROP_GUARD.with(|guard| {
            if guard.get() {
                panic!("Recursive call to drop");
            }
            guard.set(true);
        });
    }
    pub fn end_drop() {
        REC_DROP_GUARD.with(|guard| {
            assert!(guard.get(), "End drop without begin drop");
            guard.set(false);
        });
    }
}

fn is_freed_ptr<T: ?Sized>(ptr: NonNull<T>) -> bool {
    let address = ptr.as_ptr() as *mut () as usize;
    address == usize::MAX
}

mod mark_and_sweep {
    use super::*;

    pub unsafe fn run<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        if item.inner().color() != Color::Purple {
            return;
        }
        paint_gray(item);
        scan_gray(item);
        collect_white(item);
    }

    unsafe fn paint_gray<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        if item.inner().color() == Color::Gray {
            return;
        }
        item.inner().paint(Color::Gray);
        for next in item.inner_mut().iter_children_mut() {
            match next {
                Object::Array(array) => {
                    array.inner().dec_ref_count();
                    paint_gray(array);
                }
                Object::Table(_) => todo!(),
                _ => {}
            }
        }
    }

    unsafe fn scan_gray<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        if item.inner().color() != Color::Gray {
            return;
        }
        let ref_count = item.inner().ref_count();
        if ref_count == 0 {
            item.inner().paint(Color::White);
            for next in item.inner_mut().iter_children_mut() {
                match next {
                    Object::Array(array) => scan_gray(array),
                    Object::Table(_) => todo!(),
                    _ => {}
                }
            }
        } else {
            paint_black(item);
        }
    }

    unsafe fn paint_black<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        if item.inner().color() == Color::Black {
            return;
        }
        item.inner().paint(Color::Black);
        for next in item.inner_mut().iter_children_mut() {
            match next {
                Object::Array(array) => {
                    array.inner().inc_ref_count();
                    paint_black(array);
                }
                Object::Table(_) => todo!(),
                _ => {}
            }
        }
    }

    unsafe fn collect_white<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        if item.inner().color() != Color::White {
            return;
        }
        item.inner().paint(Color::Black);
        for next in item.inner_mut().iter_children_mut() {
            match next {
                Object::Array(array) => collect_white(array),
                Object::Table(_) => todo!(),
                _ => {}
            }
        }
        PmsObject::deallocate_inner(item);
    }
}
