use super::{array::Inner as ArrayInner, table::Inner as TableInner, Array, Object, Table};
use core::{alloc::Layout, cell::Cell, fmt::Debug, mem, ptr, ptr::NonNull};
use std::alloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Color {
    Black,  // Active
    Purple, // Suspicious of circular references
    Gray,   // Checking for circular references
    White,  // Candidates for collection (deallocate)
}

macro_rules! debug_assert_ptr_is_not_freed {
    ($ptr:expr) => {
        debug_assert!({
            let address = $ptr.as_ptr() as *mut () as usize;
            address != usize::MAX
        });
    };
}

/// # Safety
/// TODO
pub(crate) unsafe trait PmsInner {
    fn ref_count_ref(&self) -> &Cell<usize>;
    fn color_ref(&self) -> &Cell<Color>;

    unsafe fn iter_children_mut(&mut self) -> impl Iterator<Item = &mut Object>;
    unsafe fn drain_children(&mut self) -> impl Iterator<Item = Object>;

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
        assert_ne!(ref_count, usize::MAX, "Reference count overflow");
        self.ref_count_ref().set(ref_count + 1);
    }
    fn dec_ref_count(&self) {
        let ref_count = self.ref_count_ref().get();
        assert_ne!(ref_count, 0, "Reference count underflow");
        self.ref_count_ref().set(ref_count - 1);
    }
}

/// # Safety
/// TODO
pub(crate) unsafe trait PmsObject<I: PmsInner> {
    fn ptr(&self) -> NonNull<I>;

    unsafe fn from_inner(ptr: NonNull<I>) -> Self;

    fn inner(&self) -> &I {
        debug_assert_ptr_is_not_freed!(self.ptr());
        unsafe { self.ptr().as_ref() }
    }
    unsafe fn inner_mut(&mut self) -> &mut I {
        debug_assert_ptr_is_not_freed!(self.ptr());
        self.ptr().as_mut()
    }

    fn custom_drop(this: &mut Self) {
        RecursiveDropGuard::begin_drop();

        if this.inner().color() != Color::Black {
            unreachable!("drop() is called during mark and sweep");
        }

        // First, decrement the reference count.
        this.inner().dec_ref_count();

        // If the reference count is not zero, we can't drop this object.
        // But there is a possibility that this object is a part of a cycle, so we need to do `mark_and_sweep` from this object.
        if this.inner().ref_count() > 0 {
            unsafe {
                this.inner().paint(Color::Purple); // Mark as suspicious of cycle reference
                mark_and_sweep::run(this);
            }
            RecursiveDropGuard::end_drop();
            return;
        }

        // If the reference count is zero, we can drop this object.
        unsafe {
            // Mark this object as white as it is a candidate for collection.
            this.inner().paint(Color::White);

            // Next, we need to collect objects that can be traced from `this` that are suspected to be circular references.
            //
            // Q: Why do not we call `mem::drop_in_place()` on `this.prt()`?
            // A: It has a performance problems.
            //    Please imagine the following case:
            //      - Root node has only k leaves.
            //      - Each leaf has circular references between leaves.
            //      - `this` is the root node.
            //    In this case, we can drop all leaves, but we can't drop all leaves until we call `drop()` for the last leaf if we
            //    call `mem::drop_in_place()` on `this.ptr()`. At this time, O(k^2) for a graph with k complete leaves.
            //
            // To collect objects for which circular references are suspected, we use `PurpleCollector`.
            // `PurpleCollector` is a struct that collects objects for which circular references are suspected and marks them as purple.
            struct PurpleCollector {
                // In the `Object` enum, only `Array` and `Table` are `PmsObject`.
                // If you add other `PmsObject` variants in the future, you will need to add them here as well.
                array: Vec<NonNull<ArrayInner>>,
                table: Vec<NonNull<TableInner>>,
            }
            impl PurpleCollector {
                fn new() -> Self {
                    Self {
                        array: Vec::new(),
                        table: Vec::new(),
                    }
                }
                /// Collect purple objects (suspected of circular references) that can be traced from the object pointed to by `ptr`.
                /// White objects found during tracing are applied `PmsObject::deallocate_inner()` recursively.
                ///
                /// Given `ptr` must be `ref_count() == 0`.
                unsafe fn collect<I: PmsInner>(&mut self, mut ptr: NonNull<I>) {
                    debug_assert_eq!(ptr.as_ref().ref_count(), 0);
                    debug_assert_eq!(ptr.as_ref().color(), Color::White);

                    // For each child that can be traced from `ptr`
                    for next in ptr.as_mut().iter_children_mut() {
                        match next {
                            // If the child is `PmsObject`...
                            // (`_check_suspicious()` is just a function to cut out common processes, and I think it is not a good name.)
                            Object::Array(array) => {
                                if let Some(array) = self._check_suspicious(array) {
                                    self.array.push(array.ptr());
                                }
                            }
                            Object::Table(table) => {
                                if let Some(table) = self._check_suspicious(table) {
                                    self.table.push(table.ptr());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                /// This function must be called only from `collect()`.
                unsafe fn _check_suspicious<'a, I: PmsInner + 'a, T: PmsObject<I>>(
                    &mut self,
                    item: &'a mut T,
                ) -> Option<&'a mut T> {
                    if cfg!(debug_assertions) {
                        let color = item.inner().color();
                        assert!(
                            color == Color::Black || color == Color::Purple,
                            "Expected black or purple, but got {color:?}",
                        );
                    }

                    // From the prerequisites (callee position) of this function, the parent object of `item` is `ref_count() == 0`.
                    item.inner().dec_ref_count();

                    if item.inner().ref_count() == 0 {
                        item.inner().paint(Color::White);
                        self.collect(item.ptr());
                        deallocate_inner(item);
                        None
                    } else {
                        // To avoid double collection, we need to check whether its color is purple.
                        if item.inner().color() == Color::Purple {
                            return None;
                        }
                        item.inner().paint(Color::Purple);
                        Some(item)
                    }
                }
                unsafe fn finish(self) -> (Vec<NonNull<ArrayInner>>, Vec<NonNull<TableInner>>) {
                    let Self {
                        mut array,
                        mut table,
                    } = self;
                    array.retain_mut(|x| x.as_ref().color() == Color::Purple);
                    table.retain_mut(|x| x.as_ref().color() == Color::Purple);
                    (array, table)
                }
            }

            // Collect purple objects and apply `mark_and_sweep::run()` for them.
            let mut purple_collector = PurpleCollector::new();
            purple_collector.collect(this.ptr());
            let (purple_array_ptrs, purple_table_ptrs) = purple_collector.finish();

            deallocate_inner(this);

            for array_ptr in purple_array_ptrs {
                mark_and_sweep::run_with_inner_ptr::<_, Array>(array_ptr);
            }
            for table_ptr in purple_table_ptrs {
                mark_and_sweep::run_with_inner_ptr::<_, Table>(table_ptr);
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
    fn begin_drop() {
        REC_DROP_GUARD.with(|guard| {
            assert!(!guard.get(), "Recursive call to drop");
            guard.set(true);
        });
    }
    fn end_drop() {
        REC_DROP_GUARD.with(|guard| {
            assert!(guard.get(), "End drop without begin drop");
            guard.set(false);
        });
    }
}

unsafe fn deallocate_inner<I: PmsInner, T: PmsObject<I> + ?Sized>(this: &mut T) {
    debug_assert_ptr_is_not_freed!(this.ptr());
    debug_assert_eq!(this.inner().ref_count(), 0);

    let ptr = if cfg!(debug_assertions) {
        mem::replace(
            &mut this.ptr(),
            NonNull::new_unchecked(usize::MAX as *mut _),
        )
    } else {
        this.ptr()
    };
    let mut inner = ptr::read(ptr.as_ptr());
    for next in inner.drain_children() {
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
                mem::forget(next);
            }
            Object::Table(table) => {
                mem::forget(table);
            }
        }
    }
    drop(inner);
    alloc::dealloc(ptr.as_ptr().cast(), Layout::for_value(ptr.as_ref()));
}

mod mark_and_sweep {
    use super::*;

    pub(super) unsafe fn run_with_inner_ptr<I: PmsInner, T: PmsObject<I>>(ptr: NonNull<I>) {
        let mut object: T = PmsObject::from_inner(ptr);
        run(&mut object);
        mem::forget(object);
    }

    pub(super) unsafe fn run<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        if item.inner().color() != Color::Purple {
            return;
        }
        paint_gray(item);
        scan_gray(item);
        collect_white(item);
    }

    /// Tentatively removing. (試験削除)
    /// Paint the object (`item`) gray and decrement the reference count recursively.
    unsafe fn paint_gray<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        // Infinite recursion prevention.
        // If `item` is gray, the object that can be traced from `item` has already been tentatively removed, so nothing needs to be done.
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
                Object::Table(table) => {
                    table.inner().dec_ref_count();
                    paint_gray(table);
                }
                _ => {}
            }
        }
    }

    /// Mark white if the object (`item`) is gray and its reference count is zero, and recursively apply `scan_gray()` to its children.
    /// The object that is marked gray but its reference count is not zero is passed to `paint_black()`.
    unsafe fn scan_gray<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        // We want to scan only gray objects.
        if item.inner().color() != Color::Gray {
            return;
        }

        if item.inner().ref_count() == 0 {
            // To prevent infinite recursion, we need to paint the object white before calling `scan_gray()` for its children.
            item.inner().paint(Color::White);

            for next in item.inner_mut().iter_children_mut() {
                match next {
                    Object::Array(array) => scan_gray(array),
                    Object::Table(table) => scan_gray(table),
                    _ => {}
                }
            }
        } else {
            // We can't remove this object (`item`) as its reference count is not zero.
            // Repaint `item` and its children black.
            paint_black(item);
        }
    }

    /// Paint the object (`item`) black and recursively paint its children black.
    unsafe fn paint_black<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        // Infinite recursion prevention.
        // - All PmsObject are initially black.
        // - From the implementations of `PmsObject::custom_drop()`, the children of black objects are also black.
        // So do nothing if `item` is already black.
        if item.inner().color() == Color::Black {
            return;
        }
        item.inner().paint(Color::Black);

        for next in item.inner_mut().iter_children_mut() {
            // In `paint_gray()`, we decremented the reference count for tentatively removing.
            // So we need to increment the reference count here.
            match next {
                Object::Array(array) => {
                    array.inner().inc_ref_count();
                    paint_black(array);
                }
                Object::Table(table) => {
                    table.inner().inc_ref_count();
                    paint_black(table);
                }
                _ => {}
            }
        }
    }

    unsafe fn collect_white<I: PmsInner, T: PmsObject<I> + ?Sized>(item: &mut T) {
        // We want to collect only white objects.
        if item.inner().color() != Color::White {
            return;
        }

        // TODO: なぜ黒に塗るのか説明する。
        //       黒: OK、灰: 良さそうだけど、どう？、白: 無限再帰起こすからだめ、紫: mark_and_sweep::run() の繰り返し呼び出しで死ぬ
        item.inner().paint(Color::Black);
        for next in item.inner_mut().iter_children_mut() {
            match next {
                Object::Array(array) => collect_white(array),
                Object::Table(table) => collect_white(table),
                _ => {}
            }
        }
        deallocate_inner(item);
    }
}
