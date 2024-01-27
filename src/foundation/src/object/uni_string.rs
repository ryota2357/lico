use core::{
    borrow::Borrow,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Deref},
};
use ecow::{EcoString, EcoVec};

#[derive(Clone)]
pub struct UniString(Variant);

#[derive(Clone)]
enum Variant {
    /// Only contains ASCII characters or empty string.
    Ascii(EcoString),

    /// One length Non-ASCII character.
    Char(char, EcoString),

    /// Contains Non-ASCII characters and length is more than 1.
    /// NOTE: .0 must be valid UTF-8.
    NonAscii(Box<(EcoVec<u8>, EcoVec<char>)>),
}

impl UniString {
    #[inline]
    pub const fn new() -> Self {
        UniString(Variant::Ascii(EcoString::new()))
    }

    #[inline]
    pub fn from_ascii(s: &str) -> Self {
        debug_assert!(s.is_ascii());
        UniString(Variant::Ascii(EcoString::from(s)))
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Variant::Ascii(s) => s.len(),
            Variant::Char(_, _) => 1,
            Variant::NonAscii(s) => s.1.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_str(&self) -> &str {
        match &self.0 {
            Variant::Ascii(s) => s.as_str(),
            Variant::Char(_, s) => s.as_str(),
            Variant::NonAscii(s) => {
                if cfg!(debug_assertions) {
                    std::str::from_utf8(&s.0).expect("UniString::Variant::0 must be valid UTF-8")
                } else {
                    unsafe { std::str::from_utf8_unchecked(&s.0) }
                }
            }
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            Variant::Ascii(s) => s.as_bytes(),
            Variant::Char(_, s) => s.as_bytes(),
            Variant::NonAscii(s) => &s.0,
        }
    }

    pub fn get(&self, index: usize) -> Option<char> {
        match &self.0 {
            Variant::Ascii(s) => s.as_bytes().get(index).map(|b| *b as char),
            Variant::Char(c, _) => {
                if index == 0 {
                    Some(*c)
                } else {
                    None
                }
            }
            Variant::NonAscii(s) => s.1.get(index).copied(),
        }
    }

    pub fn sub_str(&self, start: usize, end: usize) -> Option<Self> {
        match &self.0 {
            Variant::Ascii(s) => {
                let str = s.as_str().get(start..end)?;
                Some(UniString(Variant::Ascii(EcoString::from(str))))
            }
            Variant::Char(_, _) => match (start, end) {
                (0, 0) => Some(UniString::from("")),
                (0, 1) => Some(self.clone()),
                _ => None,
            },
            Variant::NonAscii(s) => {
                let string = s.1.get(start..end)?.iter().collect::<String>();
                Some(UniString::from(string))
            }
        }
    }
}

impl Default for UniString {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<&str> for UniString {
    fn from(s: &str) -> Self {
        let variant = {
            if s.is_ascii() {
                Variant::Ascii(EcoString::from(s))
            } else {
                let chars = s.chars().collect::<EcoVec<char>>();
                if chars.len() == 1 {
                    Variant::Char(chars[0], EcoString::from(s))
                } else {
                    let bytes = EcoVec::from(s.as_bytes());
                    Variant::NonAscii(Box::new((bytes, chars)))
                }
            }
        };
        UniString(variant)
    }
}

impl From<char> for UniString {
    fn from(c: char) -> Self {
        if c.is_ascii() {
            UniString(Variant::Ascii(EcoString::from(c)))
        } else {
            UniString(Variant::Char(c, EcoString::from(c)))
        }
    }
}

macro_rules! impl_from_for {
    ($ty:ty) => {
        impl From<$ty> for UniString {
            #[inline]
            fn from(s: $ty) -> Self {
                UniString::from(s.as_str())
            }
        }
        impl From<UniString> for $ty {
            #[inline]
            fn from(s: UniString) -> Self {
                s.as_str().into()
            }
        }
    };
}
impl_from_for!(String);
impl_from_for!(EcoString);

impl<T: Into<UniString>> AddAssign<T> for UniString {
    fn add_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        match self {
            UniString(Variant::Ascii(lhs)) => match rhs.0 {
                Variant::Ascii(rhs) => lhs.push_str(rhs.as_str()),
                Variant::Char(rhs, _) => {
                    if lhs.is_empty() {
                        *self = UniString::from(rhs);
                    } else {
                        lhs.push(rhs);
                        let bytes = EcoVec::from(lhs.as_bytes());
                        let chars = lhs.chars().collect::<EcoVec<_>>();
                        self.0 = Variant::NonAscii(Box::new((bytes, chars)));
                    }
                }
                Variant::NonAscii(_) => {
                    lhs.push_str(rhs.as_str());
                    let bytes = EcoVec::from(lhs.as_bytes());
                    let chars = lhs.chars().collect::<EcoVec<_>>();
                    self.0 = Variant::NonAscii(Box::new((bytes, chars)));
                }
            },
            UniString(Variant::Char(lhs_char, lhs_string)) => match rhs.0 {
                Variant::Ascii(rhs) => {
                    if rhs.is_empty() {
                        return;
                    }
                    lhs_string.push_str(rhs.as_str());
                    let bytes = EcoVec::from(lhs_string.as_bytes());
                    let chars = lhs_string.chars().collect::<EcoVec<_>>();
                    self.0 = Variant::NonAscii(Box::new((bytes, chars)));
                }
                Variant::Char(rhs_char, _) => {
                    let bytes = EcoVec::from(format!("{lhs_char}{rhs_char}").as_bytes());
                    let chars = EcoVec::from([*lhs_char, rhs_char]);
                    self.0 = Variant::NonAscii(Box::new((bytes, chars)));
                }
                Variant::NonAscii(mut rhs) => {
                    for byte in lhs_char.to_string().into_bytes().iter().rev() {
                        rhs.0.insert(0, *byte);
                    }
                    rhs.1.insert(0, *lhs_char);
                    self.0 = Variant::NonAscii(rhs);
                }
            },
            UniString(Variant::NonAscii(lhs)) => match rhs.0 {
                Variant::Ascii(rhs) => {
                    lhs.0.extend_from_slice(rhs.as_bytes());
                    lhs.1.extend(rhs.chars().collect::<Vec<_>>());
                }
                Variant::Char(rhs_char, rhs_string) => {
                    lhs.0.extend_from_slice(rhs_string.as_bytes());
                    lhs.1.push(rhs_char);
                }
                Variant::NonAscii(rhs) => {
                    lhs.0.extend(rhs.0);
                    lhs.1.extend(rhs.1);
                }
            },
        }
    }
}

macro_rules! impl_add_for {
    ($ty:ty) => {
        impl Add<$ty> for UniString {
            type Output = Self;

            #[inline]
            fn add(mut self, rhs: $ty) -> Self::Output {
                self += rhs;
                self
            }
        }
        impl Add<UniString> for $ty {
            type Output = UniString;

            #[inline]
            fn add(self, rhs: UniString) -> Self::Output {
                let mut lhs = UniString::from(self);
                lhs += rhs;
                lhs
            }
        }
    };
}
impl_add_for!(&str);
impl_add_for!(String);
impl_add_for!(EcoString);

impl Debug for UniString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            Variant::Ascii(s) => f.debug_tuple("UniString::Ascii").field(s).finish(),
            Variant::Char(c, s) => {
                assert_eq!(c.to_string().as_str(), s);
                f.debug_tuple("UniString::Char").field(c).finish()
            }
            Variant::NonAscii(s) => f
                .debug_struct("UniString::NonAscii")
                .field("bytes", &s.0)
                .field("chars", &s.1)
                .finish(),
        }
    }
}

impl Display for UniString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for UniString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for UniString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Deref for UniString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Eq for UniString {}

impl PartialEq for UniString {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Hash for UniString {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

macro_rules! impl_symmetric_partial_eq_for {
    ($ty:ty) => {
        impl PartialEq<$ty> for UniString {
            #[inline]
            fn eq(&self, o: &$ty) -> bool {
                <str as PartialEq<str>>::eq(AsRef::as_ref(self), AsRef::as_ref(o))
            }
        }
        impl PartialEq<UniString> for $ty {
            #[inline]
            fn eq(&self, o: &UniString) -> bool {
                <str as PartialEq<str>>::eq(AsRef::as_ref(self), AsRef::as_ref(o))
            }
        }
    };
}
impl_symmetric_partial_eq_for!(str);
impl_symmetric_partial_eq_for!(&str);
impl_symmetric_partial_eq_for!(String);
impl_symmetric_partial_eq_for!(&String);

impl Ord for UniString {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for UniString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! check {
        ($target:expr, {
            variant = $variant:ident,
            len = $len:literal,
            str = $str:literal $(,)?
        }) => {
            match &$target.0 {
                Variant::$variant(..) => {}
                _ => panic!(
                    "expected variant: {}\nactual variant:   {}",
                    stringify!($variant),
                    match &$target.0 {
                        Variant::Ascii(_) => "Ascii",
                        Variant::Char(_, _) => "Char",
                        Variant::NonAscii(_) => "NonAscii",
                    }
                ),
            }
            assert_eq!($target.len(), $len);
            assert_eq!($target.as_str(), $str);
        };
    }

    /*
     * Check 5 types of string:
     *   1. empty string                        -> Ascii
     *   2. 1 length ASCII string               -> Ascii
     *   3. more than 1 length ASCII string     -> Ascii
     *   4. 1 length Non-ASCII string           -> Char
     *   5. more than 1 length Non-ASCII string -> NonAscii
     */

    #[test]
    fn construct_with_new() {
        let empty = UniString::new();
        check!(empty, { variant = Ascii, len = 0, str = "" });
    }

    #[test]
    fn construct_with_str() {
        let empty = UniString::from("");
        check!(empty, { variant = Ascii, len = 0, str = "" });

        let a_char = UniString::from("a");
        check!(a_char, { variant = Ascii, len = 1, str = "a" });

        let ascii = UniString::from("abc");
        check!(ascii, { variant = Ascii, len = 3, str = "abc" });

        let u_char = UniString::from("😃");
        check!(u_char, { variant = Char, len = 1, str = "😃" });

        let non_ascii = UniString::from("あいうえお");
        check!(non_ascii, {
            variant = NonAscii,
            len = 5,
            str = "あいうえお"
        });
    }

    #[test]
    fn construct_with_char() {
        let ascii = UniString::from('a');
        check!(ascii, { variant = Ascii, len = 1, str = "a" });

        let uchar = UniString::from('あ');
        check!(uchar, { variant = Char, len = 1, str = "あ" });
    }

    #[test]
    fn add_with_empty() {
        let empty = UniString::new();

        let empty2 = empty.clone() + "";
        check!(empty2, { variant = Ascii, len = 0, str = "" });

        let a_char = empty.clone() + "a";
        check!(a_char, { variant = Ascii, len = 1, str = "a" });

        let ascii = empty.clone() + "abc";
        check!(ascii, { variant = Ascii, len = 3, str = "abc" });

        let u_char = empty.clone() + "👍";
        check!(u_char, { variant = Char, len = 1, str = "👍" });

        let non_ascii = empty + "你好世界";
        check!(non_ascii, {
            variant = NonAscii,
            len = 4,
            str = "你好世界"
        });
    }

    #[test]
    fn add_with_ascii() {
        let ascii = UniString::from("abc");

        let empty = "" + ascii.clone();
        check!(empty, { variant = Ascii, len = 3, str = "abc" });

        let a_char = "a" + ascii.clone();
        check!(a_char, { variant = Ascii, len = 4, str = "aabc" });

        let ascii2 = "+~" + ascii.clone();
        check!(ascii2, { variant = Ascii, len = 5, str = "+~abc" });

        let u_char = "👀" + ascii.clone();
        check!(u_char, { variant = NonAscii, len = 4, str = "👀abc" });

        let non_ascii = "Спасибо большое " + ascii;
        check!(non_ascii, {
            variant = NonAscii,
            len = 19,
            str = "Спасибо большое abc"
        });
    }

    #[test]
    fn add_with_a_char() {
        let a_char = UniString::from("a");

        let empty = a_char.clone() + "";
        check!(empty, { variant = Ascii, len = 1, str = "a" });

        let a_char2 = a_char.clone() + "a";
        check!(a_char2, { variant = Ascii, len = 2, str = "aa" });

        let ascii = a_char.clone() + "abc";
        check!(ascii, { variant = Ascii, len = 4, str = "aabc" });

        let u_char = a_char.clone() + "🫚";
        check!(u_char, { variant = NonAscii, len = 2, str = "a🫚" });

        let non_ascii = a_char + " 맛있는 음식";
        check!(non_ascii, {
            variant = NonAscii,
            len = 8,
            str = "a 맛있는 음식"
        });
    }

    #[test]
    fn add_with_u_char() {
        let u_char = UniString::from("");

        let empty = u_char.clone() + "";
        check!(empty, { variant = Char, len = 1, str = "" });

        let a_char = u_char.clone() + "a";
        check!(a_char, { variant = NonAscii, len = 2, str = "a" });

        let ascii = u_char.clone() + "abc";
        check!(ascii, { variant = NonAscii, len = 4, str = "abc" });

        let u_char2 = u_char.clone() + "🎉";
        check!(u_char2, { variant = NonAscii, len = 2, str = "🎉" });

        let non_ascii = u_char + "";
        check!(non_ascii, {
            variant = NonAscii,
            len = 5,
            str = ""
        });
    }

    #[test]
    fn add_with_non_ascii() {
        let non_ascii = UniString::from("Hello 世界");

        let empty = non_ascii.clone() + "";
        check!(empty, {
            variant = NonAscii,
            len = 8,
            str = "Hello 世界"
        });

        let a_char = non_ascii.clone() + "?";
        check!(a_char, {
            variant = NonAscii,
            len = 9,
            str = "Hello 世界?"
        });

        let ascii = non_ascii.clone() + "!!";
        check!(ascii, {
            variant = NonAscii,
            len = 10,
            str = "Hello 世界!!"
        });

        let u_char = non_ascii.clone() + "。";
        check!(u_char, {
            variant = NonAscii,
            len = 9,
            str = "Hello 世界。"
        });

        let non_ascii2 = non_ascii.clone() + "、やっとあえたね";
        check!(non_ascii2, {
            variant = NonAscii,
            len = 16,
            str = "Hello 世界、やっとあえたね"
        });
    }

    #[test]
    fn with_other_type() {
        let uni_string = UniString::from("0");

        let a = uni_string.clone() + "1";
        assert_eq!(a, "01");
        assert_eq!("01", a);

        let b = "1" + uni_string.clone();
        assert_eq!(b, "10");
        assert_eq!("10", b);

        let c = uni_string.clone() + String::from("2");
        assert_eq!(c, "02");
        assert_eq!("02", c);

        let d = String::from("2") + uni_string.clone();
        assert_eq!(d, "20");
        assert_eq!("20", d);

        let e = uni_string.clone() + EcoString::from("3");
        assert_eq!(e, "03");
        assert_eq!("03", e);

        let f = EcoString::from("3") + uni_string.clone();
        assert_eq!(f, "30");
        assert_eq!("30", f);
    }

    #[test]
    fn index() {
        let ascii = UniString::from("abc");
        assert_eq!(ascii.get(0), Some('a'));
        assert_eq!(ascii.get(1), Some('b'));
        assert_eq!(ascii.get(2), Some('c'));
        assert_eq!(ascii.get(3), None);

        let u_char = UniString::from("");
        assert_eq!(u_char.get(0), Some(''));
        assert_eq!(u_char.get(1), None);

        //                               0 1 2 345678901
        let non_ascii = UniString::from("やあ、ryota2357");
        assert_eq!(non_ascii.get(0), Some('や'));
        assert_eq!(non_ascii.get(2), Some('、'));
        assert_eq!(non_ascii.get(6), Some('t'));
        assert_eq!(non_ascii.get(11), Some('7'));
        assert_eq!(non_ascii.get(12), None);
    }

    #[test]
    fn sub_str() {
        let ascii = UniString::from("abc");
        assert_eq!(ascii.sub_str(0, 3), Some(UniString::from("abc")));
        assert_eq!(ascii.sub_str(1, 2), Some(UniString::from("b")));
        assert_eq!(ascii.sub_str(1, 3), Some(UniString::from("bc")));
        assert_eq!(ascii.sub_str(3, 3), Some(UniString::from("")));
        assert_eq!(ascii.sub_str(2, 4), None);

        let u_char = UniString::from("👍");
        assert_eq!(u_char.sub_str(0, 1), Some(UniString::from("👍")));
        assert_eq!(u_char.sub_str(0, 0), Some(UniString::from("")));
        assert_eq!(u_char.sub_str(1, 2), None);

        let non_ascii = UniString::from("あaいbうcえdおe");
        assert_eq!(
            non_ascii.sub_str(0, 10),
            Some(UniString::from("あaいbうcえdおe"))
        );
        assert_eq!(non_ascii.sub_str(1, 1), Some(UniString::from("")));
        assert_eq!(non_ascii.sub_str(1, 2), Some(UniString::from("a")));
        assert_eq!(non_ascii.sub_str(1, 5), Some(UniString::from("aいbう")));
        assert_eq!(non_ascii.sub_str(5, 11), None);
    }
}
