#[macro_export]
#[doc(hidden)]
macro_rules! __count {
     () => (0usize);
     ($x:tt $($xs:tt)*) => (1usize + $crate::macros::count!($($xs)*));
}
pub use __count as count;
