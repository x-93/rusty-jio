//! Invariant assertions.

#[macro_export]
macro_rules! jio_assert {
    ($cond:expr) => {
        if !$cond {
            panic!("Jio invariant assertion failed: {}", stringify!($cond));
        }
    };
    ($cond:expr, $($arg:tt)+) => {
        if !$cond {
            panic!("Jio invariant assertion failed: {}", format_args!($($arg)+));
        }
    };
}
