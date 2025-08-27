//! # Utilities for minidb

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
/// let str = read_from_file("file.txt").expect("Failed to read file");
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::NamedTempFile;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Person {
        name: String,
        age: u8,
    }

    #[test]
    fn test_read_from_file() {
        const CONTENT: &str = "Hello world";

        let file = NamedTempFile::new().expect("Failed to create temporary file");
        let path = file.path();

        std::fs::write(path, CONTENT).expect("Failed to write to file");

        let s = read_from_file(path).expect("Failed to read file");
        assert_eq!(s, CONTENT);
    }
}
