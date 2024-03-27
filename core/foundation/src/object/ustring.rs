use core::{
    borrow::Borrow,
    cmp, fmt,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    ops::{Add, AddAssign},
};
use std::rc::Rc;

#[derive(Clone)]
pub struct UString(Variant);

#[derive(Clone)]
enum Variant {
    Empty,
    Occupied(Rc<internal::UnicodeBasedString>),
}

impl UString {
    pub const fn new() -> Self {
        UString(Variant::Empty)
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Variant::Empty => 0,
            Variant::Occupied(x) => x.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        debug_assert!(match &self.0 {
            Variant::Empty => true,
            Variant::Occupied(x) => !x.is_empty(),
        });
        matches!(&self.0, Variant::Empty)
    }

    pub fn is_ascii(&self) -> bool {
        match &self.0 {
            Variant::Empty => true,
            Variant::Occupied(x) => x.is_ascii(),
        }
    }

    pub fn as_str(&self) -> &str {
        match &self.0 {
            Variant::Empty => "",
            Variant::Occupied(x) => x.as_str(),
        }
    }

    pub fn get(&self, index: usize) -> Option<char> {
        match &self.0 {
            Variant::Empty => None,
            Variant::Occupied(x) => x.get(index),
        }
    }

    pub fn sub_string(&self, start: usize, end: usize) -> Option<Self> {
        match &self.0 {
            Variant::Empty => None,
            Variant::Occupied(x) => {
                let sub = x.sub_string(start, end)?;
                let variant = if sub.is_empty() {
                    Variant::Empty
                } else {
                    Variant::Occupied(Rc::new(sub))
                };
                Some(UString(variant))
            }
        }
    }
}

impl Default for UString {
    fn default() -> Self {
        UString::new()
    }
}

impl From<&str> for UString {
    fn from(value: &str) -> Self {
        if value.is_empty() {
            UString(Variant::Empty)
        } else {
            let s = internal::UnicodeBasedString::from(value);
            UString(Variant::Occupied(Rc::new(s)))
        }
    }
}

impl AddAssign for UString {
    fn add_assign(&mut self, rhs: Self) {
        match (&mut self.0, &rhs.0) {
            (_, Variant::Empty) => {}
            (Variant::Empty, Variant::Occupied(_)) => {
                *self = rhs;
            }
            (Variant::Occupied(lhs), Variant::Occupied(rhs)) => {
                Rc::make_mut(lhs).push_str(rhs.as_str());
            }
        }
    }
}
impl AddAssign<&str> for UString {
    fn add_assign(&mut self, rhs: &str) {
        if rhs.is_empty() {
            return;
        }
        match &mut self.0 {
            Variant::Empty => {
                *self = UString::from(rhs);
            }
            Variant::Occupied(s) => {
                Rc::make_mut(s).push_str(rhs);
            }
        }
    }
}

impl Add for UString {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self {
        self += rhs;
        self
    }
}
impl Add<&str> for UString {
    type Output = Self;

    fn add(mut self, rhs: &str) -> Self {
        self += UString::from(rhs);
        self
    }
}
impl Add<UString> for &str {
    type Output = UString;

    fn add(self, rhs: UString) -> UString {
        UString::from(self) + rhs
    }
}

impl AsRef<str> for UString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for UString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Eq for UString {}

impl PartialEq for UString {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Variant::Empty, Variant::Empty) => true,
            (Variant::Occupied(lhs), Variant::Occupied(rhs)) => lhs.eq(rhs),
            _ => false,
        }
    }
}
macro_rules! impl_symmetric_partial_eq {
    ($($ty:ty),*) => {
        $(
            impl PartialEq<$ty> for UString {
                fn eq(&self, o: &$ty) -> bool {
                    <str as PartialEq<str>>::eq(AsRef::as_ref(self), AsRef::as_ref(o))
                }
            }
            impl PartialEq<UString> for $ty {
                fn eq(&self, o: &UString) -> bool {
                    <str as PartialEq<str>>::eq(AsRef::as_ref(self), AsRef::as_ref(o))
                }
            }
        )*
    };
}
impl_symmetric_partial_eq!(str, &str, String, &String);

impl Ord for UString {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for UString {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for UString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Display for UString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Debug for UString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Variant::Empty => f.write_str(""),
            Variant::Occupied(s) => f.write_str(s.as_str()),
        }
    }
}

mod internal {
    use compact_str::CompactString;

    #[derive(Clone)]
    pub struct UnicodeBasedString(Variant);

    #[derive(Clone)]
    enum Variant {
        Ascii(CompactString),
        NonAscii(CompactString, Vec<usize>),
    }
    use Variant::*;

    impl UnicodeBasedString {
        pub fn len(&self) -> usize {
            match &self.0 {
                Ascii(s) => s.len(),
                NonAscii(_, v) => v.len() - 1,
            }
        }

        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        pub fn is_ascii(&self) -> bool {
            debug_assert!(match &self.0 {
                Ascii(x) => x.is_ascii(),
                NonAscii(x, _) => !x.is_ascii(),
            });
            matches!(&self.0, Ascii(_))
        }

        pub fn as_str(&self) -> &str {
            match &self.0 {
                Ascii(s) => s.as_str(),
                NonAscii(s, _) => s.as_str(),
            }
        }

        pub fn get(&self, index: usize) -> Option<char> {
            match &self.0 {
                Ascii(s) => s.as_bytes().get(index).map(|b| *b as char),
                NonAscii(s, pos) => {
                    let start = *pos.get(index)?;
                    let end = *pos.get(index + 1)?;
                    let str = s.get(start..end)?;
                    let mut chars = str.chars();
                    let char = unsafe { chars.next().unwrap_unchecked() };
                    debug_assert_eq!(chars.next(), None);
                    Some(char)
                }
            }
        }

        pub fn sub_string(&self, start: usize, end: usize) -> Option<Self> {
            match &self.0 {
                Ascii(s) => {
                    let str = s.get(start..end)?;
                    Some(UnicodeBasedString(Ascii(CompactString::from(str))))
                }
                NonAscii(s, pos) => {
                    let start = *pos.get(start)?;
                    let end = *pos.get(end)?;
                    let str = s.get(start..end)?;
                    Some(UnicodeBasedString::from(str))
                }
            }
        }

        pub fn push_str(&mut self, string: &str) {
            match (&mut self.0, string.is_ascii()) {
                (Ascii(s), true) => {
                    s.push_str(string);
                }
                (Ascii(s), false) => {
                    // ここ、CompactString 再生成しない方法ないの？
                    s.push_str(string);
                    let s = CompactString::from(s.as_str());
                    let points = s
                        .char_indices()
                        .map(|(i, _)| i)
                        .chain(core::iter::once(s.len()))
                        .collect();
                    self.0 = NonAscii(s, points);
                }
                (NonAscii(s, pos), _) => {
                    s.push_str(string);
                    let last_byte = unsafe { pos.pop().unwrap_unchecked() };
                    pos.extend(string.char_indices().map(|(i, _)| last_byte + i));
                    pos.push(s.len());
                }
            }
        }
    }

    impl PartialEq for UnicodeBasedString {
        fn eq(&self, other: &Self) -> bool {
            match (&self.0, &other.0) {
                (Ascii(_), NonAscii(_, _)) | (NonAscii(_, _), Ascii(_)) => false,
                (Ascii(s), Ascii(t)) | (NonAscii(s, _), NonAscii(t, _)) => s.eq(t),
            }
        }
    }

    impl<T: AsRef<str>> From<T> for UnicodeBasedString {
        fn from(value: T) -> Self {
            let str = value.as_ref();
            let string = CompactString::from(str);
            if str.is_ascii() {
                UnicodeBasedString(Ascii(string))
            } else {
                let pos = str
                    .char_indices()
                    .map(|(i, _)| i)
                    .chain(core::iter::once(str.len()))
                    .collect();
                UnicodeBasedString(NonAscii(string, pos))
            }
        }
    }
}
