#[macro_export]
macro_rules! ensure_argument_length {
    ($args:expr, 0) => {
        if !$args.is_empty() {
            return Err(format!("expected 0 arguments, got {}", $args.len()));
        }
    };
    ($args:expr, $len:literal) => {
        if $args.len() != $len {
            return Err(format!("expected {} arguments, got {}", $len, $args.len()));
        }
    };
}

#[macro_export]
macro_rules! table_extract_values {
    ($tbl:expr, { $($key:ident : $type:ident),+ $(,)? }) => {
        {
            if let ($(Some(Object::$type($key)),)*) = ($($tbl.borrow().get(stringify!($key)),)+) {
                ($($key.clone()),+)
            } else {
                unreachable!("{} should have {}.", stringify!($tbl), stringify!($($key),+));
            }
        }
    }
}
