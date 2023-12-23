use std::{fmt, ops::Range};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSpan {
    start: u32,
    end: u32,
}

impl TextSpan {
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    #[inline]
    pub const fn start(&self) -> u32 {
        self.start
    }

    #[inline]
    pub const fn end(&self) -> u32 {
        self.end
    }

    #[inline]
    pub const fn len(&self) -> u32 {
        self.end - self.start
    }

    #[inline]
    pub const fn into_range(self) -> Range<u32> {
        self.start..self.end
    }
}

impl fmt::Debug for TextSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TextSpan({}..{})", self.start, self.end)
    }
}

impl fmt::Display for TextSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

macro_rules! impl_from_pair {
    ($($t:ty),*) => {
        $(
            impl From<($t, $t)> for TextSpan {
                #[inline]
                fn from((start, end): ($t, $t)) -> Self {
                    Self::new(start as u32, end as u32)
                }
            }
            impl From<Range<$t>> for TextSpan {
                #[inline]
                fn from(range: Range<$t>) -> Self {
                    Self::new(range.start as u32, range.end as u32)
                }
            }
        )*
    };
}
impl_from_pair!(u8, u16, u32);
