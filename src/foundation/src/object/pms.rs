use super::{Array, Object, Table};
use core::{cell::Cell, fmt::Debug, ptr::NonNull};

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

pub trait HasPmsInner<I: PmsInner> {
    fn ptr(&self) -> NonNull<I>;
    unsafe fn iter_inner_children_mut(&mut self) -> impl Iterator<Item = &mut Object>;

    fn inner(&self) -> &I {
        assert!(!is_freed_ptr(self.ptr()));
        unsafe { self.ptr().as_ref() }
    }
    unsafe fn inner_mut(&mut self) -> &mut I {
        assert!(!is_freed_ptr(self.ptr()));
        self.ptr().as_mut()
    }
}

fn is_freed_ptr<T: ?Sized>(ptr: NonNull<T>) -> bool {
    let address = ptr.as_ptr() as *mut () as usize;
    address == usize::MAX
}

thread_local! {
    static REC_DROP_GUARD: Cell<bool> = const { Cell::new(false) };
}
pub struct RecursiveDropGuard;
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
            assert!(guard.get() == true, "End drop without begin drop");
            guard.set(false);
        });
    }
}

struct PurpleCollector<'a> {
    array: Vec<&'a mut Array>,
    table: Vec<&'a mut Table>,
}
impl<'a> PurpleCollector<'a> {
    unsafe fn collect<I: PmsInner + 'a>(&mut self, mut ptr: NonNull<I>) {
        assert!(ptr.as_ref().ref_count() == 0);
        for next in ptr.as_mut().data.iter_mut() {
            match next.as_object_mut() {
                Object::Array(array) => {
                    array.inner().dec_ref_count();
                    if array.inner().ref_count() == 0 {
                        array.inner().paint(Color::White);
                        self.collect(array.ptr());
                        // deallocate_inner(array);
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

pub unsafe fn mark_and_sweep<I: PmsInner, T: HasPmsInner<I>>(item: &mut T) {
    if item.inner().ref_count() > 0 {
        mark_and_sweep_impl::paint_gray(item);
        mark_and_sweep_impl::scan_gray(item);
        mark_and_sweep_impl::collect_white(item);
    } else {
        todo!()
    }
}

mod mark_and_sweep_impl {
    use super::*;

    pub unsafe fn paint_gray<I: PmsInner, T: HasPmsInner<I>>(item: &mut T) {
        if item.inner().color() == Color::Gray {
            return;
        }
        item.inner().paint(Color::Gray);
        for next in item.iter_inner_children_mut() {
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

    pub unsafe fn scan_gray<I: PmsInner, T: HasPmsInner<I>>(item: &mut T) {
        if item.inner().color() != Color::Gray {
            return;
        }
        let ref_count = item.inner().ref_count();
        if ref_count == 0 {
            item.inner().paint(Color::White);
            for next in item.iter_inner_children_mut() {
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

    unsafe fn paint_black<I: PmsInner, T: HasPmsInner<I>>(item: &mut T) {
        if item.inner().color() == Color::Black {
            return;
        }
        item.inner().paint(Color::Black);
        for next in item.iter_inner_children_mut() {
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

    pub unsafe fn collect_white<I: PmsInner, T: HasPmsInner<I>>(item: &mut T) {
        if item.inner().color() != Color::White {
            return;
        }
        item.inner().paint(Color::Black);
        for next in item.iter_inner_children_mut() {
            match next {
                Object::Array(array) => collect_white(array),
                Object::Table(_) => todo!(),
                _ => {}
            }
        }
        todo!()
        // Global.deallocate(item.ptr.cast(), Layout::for_value(item.ptr.as_ref()));
        // item.ptr = NonNull::new_unchecked(usize::MAX as *mut _);
    }
}
