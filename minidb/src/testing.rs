// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Run a function and print the time it took
///
/// ## Arguments
///
/// * `func` - The function to run
///
/// ## Example
///
/// ```rust,no_run
/// use minidb::time_function;
///
/// fn add(a: u32, b: u32) -> u32 {
///     a + b
/// }
///
/// let c = time_function!(add(1, 2));
/// println!("c: {}", c);
/// ```
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
