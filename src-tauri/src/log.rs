// info宏 仅在debug模式下打印
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("prefix: {}", format_args!($($arg)*));
    };
}

// error宏 仅在debug模式下打印
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!("prefix: {}", format_args!($($arg)*));
    };
}
