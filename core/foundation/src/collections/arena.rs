use core::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    num::NonZero,
};

pub struct Index<T> {
    raw: NonZero<u32>,
    phantom: PhantomData<fn() -> T>,
}

impl<T> Index<T> {
    /// # Safety
    /// `raw` must not be zero, and it must be a valid index for the arena.
    pub const unsafe fn new(raw: u32) -> Self {
        debug_assert!(raw != 0, "Index::new(0) is invalid");
        Self {
            raw: NonZero::new_unchecked(raw),
            phantom: PhantomData,
        }
    }

    pub fn as_u32(&self) -> u32 {
        self.raw.get()
    }
}

fn _static_assert_index_size() {
    const {
        assert!(size_of::<Index<u128>>() == 4);
        assert!(size_of::<Index<u128>>() == size_of::<Option<Index<u128>>>());
    }
}

impl<T> Copy for Index<T> {}

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Index<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut type_name = core::any::type_name::<T>();
        if let Some(idx) = type_name.rfind(':') {
            type_name = &type_name[idx + 1..]
        }
        write!(fmt, "Index::<{}>({})", type_name, self.raw)
    }
}

impl<T> Eq for Index<T> {}

impl<T> PartialEq for Index<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T> Ord for Index<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T> PartialOrd for Index<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Hash for Index<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state)
    }
}

pub struct Slice<T> {
    start: NonZero<u32>,
    end: NonZero<u32>,
    phantom: PhantomData<fn() -> [T]>,
}

impl<T> Slice<T> {
    /// # Safety
    /// `start` and `end` must not be zero and `start` must be less than `end`.
    /// Additionally, `start` and `end` must be valid indices for the arena.
    pub const unsafe fn new(start: u32, end: u32) -> Self {
        debug_assert!(start != 0, "Slice::new(0, _) is invalid");
        debug_assert!(end != 0, "Slice::new(_, 0) is invalid");
        debug_assert!(
            start <= end,
            "Slice::new(start, end) where start > end is invalid"
        );
        Self {
            start: NonZero::new_unchecked(start),
            end: NonZero::new_unchecked(end),
            phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        (self.end.get() - self.start.get()) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn as_u64(&self) -> u64 {
        (self.start.get() as u64) << 32 | self.end.get() as u64
    }
}

fn _static_assert_slice_size() {
    const {
        assert!(size_of::<Slice<u128>>() == 8);
        assert!(size_of::<Slice<u128>>() == size_of::<Option<Slice<u128>>>());
    }
}

impl<T> Copy for Slice<T> {}

impl<T> Clone for Slice<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> fmt::Debug for Slice<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut type_name = core::any::type_name::<T>();
        if let Some(idx) = type_name.rfind(':') {
            type_name = &type_name[idx + 1..]
        }
        write!(fmt, "Slice::<{}>({}..{})", type_name, self.start, self.end)
    }
}

impl<T> Eq for Slice<T> {}

impl<T> PartialEq for Slice<T> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

impl<T> Ord for Slice<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start
            .cmp(&other.start)
            .then_with(|| self.end.cmp(&other.end))
    }
}

impl<T> PartialOrd for Slice<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Hash for Slice<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}

#[derive(Clone, Default)]
pub struct Arena<T> {
    data: Vec<T>,
}

impl<T> Arena<T> {
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn alloc(&mut self, value: T) -> Index<T> {
        self.data.push(value);
        let next_index = self.data.len();
        assert!(next_index <= u32::MAX as usize, "Arena is full");
        // SAFETY: As above `self.data.push(value)`, `self.data.len()` is always greater than 0.
        unsafe { Index::new(next_index as u32) }
    }

    pub fn alloc_many(&mut self, iter: impl IntoIterator<Item = T>) -> Slice<T> {
        let start = self.data.len() + 1;
        self.data.extend(iter);
        let end = self.data.len() + 1;
        assert!(end <= u32::MAX as usize, "Arena is full");
        // SAFETY: `self.data.len()` is unsigned integer, so `self.data.len() + 1` is always greater than 0.
        unsafe { Slice::new(start as u32, end as u32) }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: Index<T>) -> &T {
        let raw = index.raw.get() as usize;
        &self.data[raw - 1]
    }

    pub fn get_mut(&mut self, index: Index<T>) -> &mut T {
        let raw = index.raw.get() as usize;
        &mut self.data[raw - 1]
    }

    pub fn get_slice(&self, slice: Slice<T>) -> &[T] {
        let start = slice.start.get() as usize;
        let end = slice.end.get() as usize;
        &self.data[(start - 1)..(end - 1)]
    }

    pub fn get_slice_mut(&mut self, slice: Slice<T>) -> &mut [T] {
        let start = slice.start.get() as usize;
        let end = slice.end.get() as usize;
        &mut self.data[(start - 1)..(end - 1)]
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (Index<T>, &T)> + DoubleEndedIterator {
        self.data.iter().enumerate().map(|(i, v)| {
            let index = Index {
                // SAFETY:
                //  - `i` is unsigned integer, so `i + 1` is always greater than 0.
                //  - `i + 1` does not overflow `u32::MAX` because `i` is ensured to be less than
                //    `u32::MAX` by alloc/alloc_many methods implementation.
                raw: unsafe { NonZero::new_unchecked((i + 1) as u32) },
                phantom: PhantomData,
            };
            (index, v)
        })
    }
}

impl<T: fmt::Debug> fmt::Debug for Arena<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("Arena")
            .field("len", &self.len())
            .field("data", &self.data)
            .finish()
    }
}
