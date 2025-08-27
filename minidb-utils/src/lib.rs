//! # Utilities for minidb
//!
//! Useful functions
//!
//! ## Functions
//!
//! **Note:** `async` functions are only available with the `tokio` feature
//!
//! ### File related
//!
//! * [`read_from_file`] - Read a file into a string using a buffer
//! * [`read_from_file_async`] - Read a file asynchronously into a string using a buffer
//!
//! ## Benchmarks
//!
//! ### File related
//!
//! ```text
//! Timer precision: 100 ns
//! fs                       fastest       │ slowest       │ median        │ mean          │ samples │ iters
//! ├─ read_from_file        655.3 µs      │ 1.131 ms      │ 739.4 µs      │ 767.9 µs      │ 100     │ 100
//! ╰─ read_from_file_async  686.7 µs      │ 1.243 ms      │ 753.3 µs      │ 788.3 µs      │ 100     │ 100
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::pedantic, missing_debug_implementations, missing_docs)]

// Copyright (c) 2025, DarkCeptor44
//
// This file is licensed under the GNU Lesser General Public License
// (either version 3 or, at your option, any later version).
//
// This software comes without any warranty, express or implied. See the
// GNU Lesser General Public License for details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this software. If not, see <https://www.gnu.org/licenses/>.

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use anyhow::{Context, Result};
use minidb_shared::MiniDBError;

/// Read a file into a string using a buffer
///
/// ## Arguments
///
/// * `path` - The path to the file to read
///
/// ## Returns
///
/// A string containing the contents of the file
///
/// ## Errors
///
/// Returns an error if the path does not exist, failed to open or failed to be read
///
/// ## Example
///
/// ```rust,no_run
/// use minidb_utils::read_from_file;
///
/// let str = read_from_file("file.txt").unwrap();
/// ```
pub fn read_from_file<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    read_from_file_impl(path.as_ref())
}

fn read_from_file_impl(path: &Path) -> Result<String> {
    let file = File::open(path).context(MiniDBError::FailedToOpenFile(path.to_path_buf()))?;
    let mut reader = BufReader::new(file);
    let mut data = String::new();

    reader
        .read_to_string(&mut data)
        .context(MiniDBError::FailedToReadFile(path.to_path_buf()))?;

    Ok(data)
}

/// Read a file asynchronously into a string using a buffer
///
/// ## Arguments
///
/// * `path` - The path to the file to read
///
/// ## Returns
///
/// A string containing the contents of the file
///
/// ## Errors
///
/// Returns an error if the path does not exist, failed to open or failed to be read
///
/// ## Example
///
/// ```rust,ignore
/// use minidb_utils::read_from_file_async;
///
/// let str = read_from_file_async("file.txt").await.unwrap();
/// ```
#[cfg(feature = "tokio")]
pub async fn read_from_file_async<P>(path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    read_from_file_async_impl(path.as_ref()).await
}

#[cfg(feature = "tokio")]
async fn read_from_file_async_impl(path: &Path) -> Result<String> {
    use tokio::io::AsyncReadExt;

    let file = tokio::fs::File::open(path)
        .await
        .context(MiniDBError::FailedToOpenFile(path.to_path_buf()))?;
    let mut reader = tokio::io::BufReader::new(file);
    let mut data = String::new();

    reader
        .read_to_string(&mut data)
        .await
        .context(MiniDBError::FailedToReadFile(path.to_path_buf()))?;

    Ok(data)
}
