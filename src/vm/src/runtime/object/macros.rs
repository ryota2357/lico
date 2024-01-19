#[macro_export]
macro_rules! ensure_argument_length {
    ($args:expr, 0) => {
        if !$args.is_empty() {
            return Err(format!(
                "Wrong number of arguments: expected 0, got {}",
                $args.len()
            ));
        }
    };
    ($args:expr, $len:expr) => {
        if $args.len() != $len {
            return Err(format!(
                "Wrong number of arguments: expected {}, got {}",
                $len,
                $args.len()
            ));
        }
    };
}

#[macro_export]
macro_rules! count {
     () => (0usize);
     ($x:tt $($xs:tt)*) => (1usize + count!($($xs)*));
}

#[macro_export]
macro_rules! extract_argument {
    ($args:expr, []) => {
        ensure_argument_length!($args, 0);
    };
    ($args:expr, [ $type:ident ]) => {
        {
            let (x,) = extract_argument!($args, [ $type, ]);
            x
        }
    };
    ($args:expr, [ $($type:ident),+ $(,)? ]) => {
        {
            ensure_argument_length!($args, count!($($type)*));
            let mut iter = $args.iter().rev();
            (
                $({
                    let next = unsafe { iter.next().unwrap_unchecked() };
                    let Object::$type(x) = next else {
                        return Err(format!(
                            "Mismatched argument type: expected {}, got {}",
                            stringify!($type).to_lowercase(),
                            next.typename()
                        ));
                    };
                    x.clone()
                },)*
            )
        }
    };
    ($args:expr, [ { $($pat:pat => $expr:expr),* $(,)? } ]) => {
        {
            let (x,) = extract_argument!($args, [ { $($pat => $expr),* }, ]);
            x
        }
    };
    ($args:expr, [ $({ $($pat:pat => $expr:expr),* $(,)? }),+ $(,)? ]) => {
        {
            ensure_argument_length!($args, count!($({ $($pat => $expr)* })*));
            let mut iter = $args.iter().rev();
            (
                $({
                    let next = unsafe { iter.next().unwrap_unchecked() };
                    match next {
                        $($pat => $expr,)*
                    }
                },)*
            )
        }
    }
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
