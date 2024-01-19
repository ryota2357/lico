use super::*;

#[derive(Clone, Debug, Eq)]
pub struct StringObject {
    value: Rc<String>,
    chars: Option<Rc<Vec<char>>>,
}

impl StringObject {
    #[inline]
    pub fn new(value: Rc<String>) -> Self {
        Self { value, chars: None }
    }

    // NOTE: Do not impl `Deref` for `StringObject`.
    //       It causes unexpected behavior due to the fact that the String is a wrapper of Vec<u8>.
    //       e.g. String::len() returns the length of the Vec<u8>, not the length of unicode characters.
    #[inline]
    pub fn inner(&self) -> &Rc<String> {
        &self.value
    }

    pub fn get_chars(&self) -> Rc<Vec<char>> {
        self.chars
            .clone()
            .unwrap_or_else(|| Rc::new(self.value.chars().collect()))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }

    pub fn deep_clone(&self) -> Self {
        Self {
            value: Rc::new(self.value.as_str().to_string()),
            chars: None,
        }
    }
}

impl PartialEq for StringObject {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl PartialOrd for StringObject {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StringObject {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl std::fmt::Display for StringObject {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub fn run_string_method(
    string: StringObject,
    name: &str,
    args: &[Object],
) -> Result<Object, String> {
    match name {
        // len() -> Int
        "len" => {
            extract_argument!(args, []);
            Ok(Object::Int(string.get_chars().len() as i64))
        }

        // to_string() -> String
        "to_string" => {
            extract_argument!(args, []);
            Ok(Object::String(string))
        }
        _ => Err(format!("{} is not a method of string", name)),
    }
}
