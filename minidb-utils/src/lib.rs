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
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use anyhow::{Context, Result};
use minidb_shared::MiniDBError;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

/// Deserialize [bitcode] data from a file
///
/// ## Arguments
///
/// * `path` - The path to the file to deserialize from
///
/// ## Returns
///
/// The deserialized data
///
/// ## Errors
///
/// Returns an error if:
///
/// * Path does not exist
/// * Failed to open file
/// * Failed to read file
/// * Failed to deserialize
///
/// ## Example
///
/// ```rust,no_run
/// use minidb_utils::deserialize_file;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let p: Person = deserialize_file("person.bin").unwrap();
/// ```
pub fn deserialize_file<P, T>(path: P) -> Result<T>
where
    P: AsRef<Path>,
    T: for<'de> Deserialize<'de>,
{
    deserialize_file_impl(path.as_ref())
}

fn deserialize_file_impl<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let file = File::open(path).context(MiniDBError::FailedToOpenFile(path.to_path_buf()))?;
    let mut reader = BufReader::new(file);
    let mut data = Vec::new();

    reader
        .read_to_end(&mut data)
        .context(MiniDBError::FailedToReadFile(path.to_path_buf()))?;

    let value: T =
        bitcode::deserialize(&data).context(MiniDBError::FailedToDeserializeData(data))?;
    Ok(value)
}

/// Deserialize [bitcode] data from a file asynchronously
///
/// ## Arguments
///
/// * `path` - The path to the file to deserialize from
///
/// ## Returns
///
/// The deserialized value
///
/// ## Errors
///
/// Returns an error if:
///
/// * Path does not exist
/// * Failed to open file
/// * Failed to read file
/// * Failed to deserialize
///
/// ## Example
///
/// ```rust,ignore
/// use minidb_utils::deserialize_file_async;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Person {
///     name: String,
///     age: u8
/// }
///
/// let p: Person = deserialize_file_async("person.bin").await.unwrap();
/// ```
#[cfg(feature = "tokio")]
pub async fn deserialize_file_async<P, T>(path: P) -> Result<T>
where
    P: AsRef<Path>,
    T: for<'de> Deserialize<'de>,
{
    deserialize_file_async_impl(path.as_ref()).await
}

#[cfg(feature = "tokio")]
async fn deserialize_file_async_impl<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    use tokio::io::AsyncReadExt;

    let file = tokio::fs::File::open(path)
        .await
        .context(MiniDBError::FailedToOpenFile(path.to_path_buf()))?;
    let mut reader = tokio::io::BufReader::new(file);
    let mut data = Vec::new();

    reader
        .read_to_end(&mut data)
        .await
        .context(MiniDBError::FailedToReadFile(path.to_path_buf()))?;

    let value: T =
        bitcode::deserialize(&data).context(MiniDBError::FailedToDeserializeData(data))?;
    Ok(value)
}

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

/// Serialize a value to a file using [bitcode]
///
/// ## Arguments
///
/// * `path` - The path to the file to serialize to
/// * `value` - The value to serialize
///
/// ## Errors
///
/// Returns an error if:
///
/// * Failed to create temporary file
/// * Failed to serialize value
/// * Failed to write or flush temporary file
/// * Failed to get temporary file from writer
/// * Failed to persist temporary file
///
/// ## Example
///
/// ```rust,no_run
/// use minidb_utils::serialize_file;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let p = Person {
///     name: String::from("John Doe"),
///     age: 31,
/// };
///
/// serialize_file("person.bin", &p).unwrap();
/// ```
pub fn serialize_file<P, T>(path: P, value: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    serialize_file_impl(path.as_ref(), value)
}

fn serialize_file_impl<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let temp_file = NamedTempFile::new_in(path.parent().unwrap_or(Path::new(".")))
        .context(MiniDBError::FailedToCreateTempFile)?;
    let temp_path = temp_file.path().to_path_buf();

    let mut writer = BufWriter::new(temp_file);
    let data = bitcode::serialize(value).context(MiniDBError::FailedToSerializeValue)?;

    writer
        .write_all(&data)
        .context(MiniDBError::FailedToWriteTempFile(temp_path.clone()))?;
    writer
        .flush()
        .context(MiniDBError::FailedToFlushTempFile(temp_path.clone()))?;

    let temp_file = writer
        .into_inner()
        .context(MiniDBError::FailedToGetInnerWriter)?;
    temp_file
        .persist(path)
        .context(MiniDBError::FailedToPersistTempFile {
            temp: temp_path,
            orig: path.to_path_buf(),
        })?;

    Ok(())
}

/// Serialize a value to a file asynchronously using [bitcode]
///
/// ## Arguments
///
/// * `path` - The path to the file to serialize to
/// * `value` - The value to serialize
///
/// ## Errors
///
/// Returns an error if:
///
/// * Failed to create temporary file
/// * Failed to reopen temporary file
/// * Failed to serialize value
/// * Failed to write or flush temporary file
/// * Failed to get temporary file from writer
/// * Failed to persist temporary file
///
/// ## Example
///
/// ```rust,ignore
/// use minidb_utils::serialize_file_async;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let p = Person {
///     name: String::from("John Doe"),
///     age: 31,
/// };
///
/// serialize_file_async("person.bin", &p).await.unwrap();
/// ```
#[cfg(feature = "tokio")]
pub async fn serialize_file_async<P, T>(path: P, value: &T) -> Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    serialize_file_async_impl(path.as_ref(), value).await
}

#[cfg(feature = "tokio")]
async fn serialize_file_async_impl<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    use tokio::io::AsyncWriteExt;

    let temp_file = NamedTempFile::new_in(path.parent().unwrap_or(Path::new(".")))
        .context(MiniDBError::FailedToCreateTempFile)?;
    let temp_path = temp_file.path().to_path_buf();
    let mut temp_file_async = tokio::fs::File::from_std(
        temp_file
            .reopen()
            .context(MiniDBError::FailedToReopenTempFile(temp_path.clone()))?,
    );
    let mut writer = tokio::io::BufWriter::new(&mut temp_file_async);
    let data = bitcode::serialize(value).context(MiniDBError::FailedToSerializeValue)?;

    writer
        .write_all(&data)
        .await
        .context(MiniDBError::FailedToWriteTempFile(temp_path.clone()))?;
    writer
        .flush()
        .await
        .context(MiniDBError::FailedToFlushTempFile(temp_path.clone()))?;

    temp_file
        .persist(path)
        .context(MiniDBError::FailedToPersistTempFile {
            temp: temp_path,
            orig: path.to_path_buf(),
        })?;

    Ok(())
}
