#[macro_export]
#[doc(hidden)]
macro_rules! __count {
     () => (0usize);
     ($x:tt $($xs:tt)*) => (1usize + $crate::macros::count!($($xs)*));
}
pub use __count as count;

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_from_variant {
    ($enum_type:ident { $($variant:ident : $type:ty),* $(,)? }) => {
        $(
            impl From<$type> for $enum_type {
                fn from(value: $type) -> Self {
                    $enum_type::$variant(value)
                }
            }
        )*
    };
}
pub use __impl_from_variant as impl_from_variant;
