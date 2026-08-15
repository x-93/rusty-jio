#[macro_export]
macro_rules! invariant {
    ($cond:expr) => {
        if !$cond {
            panic!("invariant violation: {}", stringify!($cond));
        }
    };
    ($cond:expr, $($arg:tt)+) => {
        if !$cond {
            panic!("invariant violation: {}", format_args!($($arg)+));
        }
    };
}
