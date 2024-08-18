use super::*;
use core::{
    alloc::Layout,
    cell::Cell,
    fmt, hint,
    ptr::{self, drop_in_place, NonNull},
    slice,
};
use std::alloc;

pub struct Executable {
    // | Header | Data (array of `ICode`) |
    //          ^ ptr
    ptr: NonNull<ICode>,
}

struct Header {
    // Fields should not require heap allocation
    // If you need to store a reference, fix `Clone` and `Drop` implementation.
    count: Cell<usize>,
    len: usize,
}

impl Executable {
    pub fn new<C>(code_iter: C) -> Self
    where
        C: IntoIterator<Item = ICode>,
        C::IntoIter: ExactSizeIterator,
    {
        let code_iter = code_iter.into_iter();
        let len = code_iter.len();

        let layout = Self::layout(len);
        let allocation = unsafe {
            // SAFETY: Since `Self` always has header (even if data is empty), layout is non-zero.
            alloc::alloc(layout)
        };
        if allocation.is_null() {
            alloc::handle_alloc_error(layout);
        }

        let ptr = unsafe {
            // SAFETY: `allocation` is non-null.
            *(allocation as *mut Header) = Header {
                count: Cell::new(1),
                len,
            };
            NonNull::new_unchecked(allocation.add(Self::header_offset())).cast()
        };
        for (i, icode) in code_iter.enumerate() {
            unsafe {
                // SAFETY: Since `i` is less than `len`, it is in bounds.
                ptr.add(i).write(icode);
            }
        }
        Executable { ptr }
    }

    /// Note that this requires heap memory access.
    pub fn len(&self) -> usize {
        self.header().len
    }

    /// Note that this requires heap memory access.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// # Safety
    ///
    /// `index` must be less than `self.len()`
    pub unsafe fn fetch(&self, index: usize) -> &ICode {
        debug_assert!(index < self.len());
        unsafe { &*self.ptr.as_ptr().add(index) }
    }

    /// # Safety
    ///
    /// `index` must be less than `self.len()`
    pub unsafe fn write(&self, index: usize, icode: ICode) {
        debug_assert!(index < self.len());
        unsafe {
            self.ptr.as_ptr().add(index).write(icode);
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        ptr::addr_eq(self.ptr.as_ptr(), other.ptr.as_ptr())
    }

    fn header(&self) -> &Header {
        unsafe { &*(self.allocation() as *const Header) }
    }

    unsafe fn allocation(&self) -> *mut u8 {
        self.ptr.as_ptr().cast::<u8>().sub(Self::header_offset())
    }

    /// The layout for the header and data.
    const fn layout(len: usize) -> Layout {
        // SAFETY:
        // - `Self::size(capacity)` guarantees that it rounded up the alignment does not overflow `isize::MAX`.
        // - `Self::align()` guarantees that it is valid alignment for the header and data.
        unsafe { Layout::from_size_align_unchecked(Self::size(len), Self::align()) }
    }

    const fn header_offset() -> usize {
        max(size_of::<Header>(), Self::align())
    }

    /// The alignment for the header and data.
    ///
    /// The return value is guaranteed to be:
    /// - non-zero.
    /// - a power of two.
    const fn align() -> usize {
        const {
            assert!(align_of::<ICode>() != 0);
            assert!(align_of::<ICode>() % 2 == 0);
        }
        max(align_of::<Header>(), align_of::<ICode>())
    }

    /// The size of the header and data.
    ///
    /// The return value is guaranteed to be:
    /// - non-zero.
    /// - When rounded up to the nearest multiple of `Self::align()`, does not overflow `isize::MAX`.
    const fn size(len: usize) -> usize {
        let Some(data_size) = size_of::<ICode>().checked_mul(len) else {
            size_overflow_panic()
        };
        let Some(size) = Self::header_offset().checked_add(data_size) else {
            size_overflow_panic()
        };
        if size >= (isize::MAX as usize - Self::align()) {
            size_overflow_panic()
        }
        size
    }
}

impl Clone for Executable {
    fn clone(&self) -> Self {
        let header = self.header();

        // see Rc::inc_strong
        let count = header.count.get();
        unsafe {
            hint::assert_unchecked(count != 0);
        }
        let count = count.wrapping_add(1);
        header.count.set(count);
        if count == 0 {
            std::process::abort();
        }

        Executable { ptr: self.ptr }
    }
}

impl Drop for Executable {
    fn drop(&mut self) {
        let header = self.header();
        let count = header.count.get();
        if count == 1 {
            let len = header.len;
            unsafe {
                // destroy the ICode array
                let data_slice = ptr::slice_from_raw_parts_mut(self.ptr.as_ptr(), len);
                drop_in_place(data_slice);

                // No `drop_in_place` for the header is needed.
                // Because none of the fields in the header are references.
                // free
                let allocation = self.allocation();
                let layout = Self::layout(len);
                alloc::dealloc(allocation, layout);
            }
        } else {
            header.count.set(count - 1);
        }
    }
}

impl fmt::Debug for Executable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data_len = self.len();
        let data_slice = unsafe {
            // SAFETY: `self.ptr` points to the beginning of the data, and `data_len` is the length of it.
            slice::from_raw_parts(self.ptr.as_ptr(), data_len)
        };
        f.debug_struct("Executable")
            .field("len", &data_len)
            .field("data", &data_slice)
            .finish()
    }
}

impl fmt::Display for Executable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data_len = self.len();
        let data_slice = unsafe {
            // SAFETY: `self.ptr` points to the beginning of the data, and `data_len` is the length of it.
            slice::from_raw_parts(self.ptr.as_ptr(), data_len)
        };
        for icode in data_slice {
            writeln!(f, "{}", icode)?;
        }
        Ok(())
    }
}

impl PartialEq for Executable {
    fn eq(&self, other: &Self) -> bool {
        let len = self.len();
        if len != other.len() {
            return false;
        }
        for i in 0..len {
            unsafe {
                // SAFETY: `len` is the same for both `self` and `other`.
                //          And obviously, `i` is less than `len`.
                if self.fetch(i) != other.fetch(i) {
                    return false;
                }
            }
        }
        true
    }
}

#[cold]
const fn size_overflow_panic() -> ! {
    panic!("overflow: too many elements");
}

/// const version of `std::cmp::max::<usize>(x, y)`.
const fn max(x: usize, y: usize) -> usize {
    if x > y {
        x
    } else {
        y
    }
}

// TODO: move test dir and add more tests.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let code = vec![
            ICode::LoadIntObject(0),
            ICode::LoadIntObject(1),
            ICode::LoadIntObject(2),
            ICode::LoadIntObject(3),
        ];
        unsafe {
            let exe = Executable::new(code);
            assert_eq!(exe.fetch(1), &ICode::LoadIntObject(1));

            let exe2 = Executable::clone(&exe);
            assert_eq!(exe.fetch(2), &ICode::LoadIntObject(2));
            assert_eq!(exe.header().count.get(), 2);

            exe.write(2, ICode::Unload);
            assert_eq!(exe2.fetch(2), &ICode::Unload);

            drop(exe);
            assert_eq!(exe2.header().count.get(), 1);
        }
    }
}
