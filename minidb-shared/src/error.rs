//! # Database errors

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

use std::path::PathBuf;

use thiserror::Error;

/// Represents errors that can occur when using the database
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MiniDBError {
    /// Failed to deserialize file
    #[error("Failed to deserialize file: {0}")]
    FailedToDeserializeFile(PathBuf),

    /// Failed to remove/delete file
    #[error("Failed to remove file: {0}")]
    FailedToRemoveFile(PathBuf),

    /// Failed to write file
    #[error("Failed to write to file: {0}")]
    FailedToWriteFile(PathBuf),

    /// File does not exist
    #[error("File does not exist: {0}")]
    FileDoesNotExist(PathBuf),

    /// Invalid primary key
    #[error("Invalid primary key: {0}")]
    InvalidKey(String),

    /// Record not found
    #[error("Record not found for table `{table}` with ID `{id}`")]
    RecordNotFound {
        /// The table name
        table: String,

        /// The ID of the record
        id: String,
    },
}
