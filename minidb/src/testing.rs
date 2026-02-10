/// Run a function and print the time it took
#[macro_export]
macro_rules! time_function {
    ($func:expr) => {{
        let start = std::time::Instant::now();
        let result = $func;
        if cfg!(debug_assertions) {
            let s = format!("{}: {:?}", stringify!($func), start.elapsed());
            dbg!(s);
        }
        result
    }};
}
