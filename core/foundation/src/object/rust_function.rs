use super::*;
use anyhow::{anyhow, Result};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RustFunction(Inner);

/// This is a size optimazation to avoid the size of [`Object`] being larger than 16 bytes.
///
/// As the size of [`RustFunction`] is 16 bytes, note that it has padding bytes.
/// Rust compiler cannot detect the padding if [`RustFunction`] is struct, but using Option-like
/// enum, it can detect the padding and optimize the size (niche optimazation).
///
/// I don't know the exact condition of this niche optimization, but it is mentioned in the
/// following link: https://rust-lang.github.io/unsafe-code-guidelines/layout/enums.html#discriminant-elision-on-option-like-enums
/// Or my blog post: https://ryota2357.com/blog/2024/rust-niche-opt-memo/
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Inner {
    #[allow(unused, non_camel_case_types)]
    __dummy,
    Data(
        u8,
        fn(Box<dyn ExactSizeIterator<Item = Object>>) -> Result<Object>,
    ),
}

impl RustFunction {
    pub const fn new(
        param_len: u8,
        func: fn(Box<dyn ExactSizeIterator<Item = Object>>) -> Result<Object>,
    ) -> Self {
        RustFunction(Inner::Data(param_len, func))
    }

    pub fn call(&self, args: Box<dyn ExactSizeIterator<Item = Object>>) -> Result<Object> {
        let (param_len, func) = self.data();
        if args.len() != param_len as usize {
            Err(anyhow!(
                "Invalid argument length: expected {}, got {}",
                param_len,
                args.len()
            ))
        } else {
            func(args)
        }
    }

    #[allow(clippy::type_complexity)]
    fn data(
        &self,
    ) -> (
        u8,
        fn(Box<dyn ExactSizeIterator<Item = Object>>) -> Result<Object>,
    ) {
        unsafe {
            match self.0 {
                Inner::Data(param_len, func) => (param_len, func),
                _ => core::hint::unreachable_unchecked(),
            }
        }
    }
}

impl fmt::Debug for RustFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (param_len, func) = self.data();
        f.debug_struct("RustFunction")
            .field("param_len", &param_len)
            .field("func", &func)
            .finish()
    }
}
