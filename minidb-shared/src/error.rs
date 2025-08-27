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
    /// Failed to create database
    #[error("Failed to create database directory: {0}")]
    FailedToCreateDatabase(PathBuf),

    /// Failed to create table
    #[error("Failed to create table directory: {0}")]
    FailedToCreateTableDir(PathBuf),

    /// Failed to create temporary file
    #[error("Failed to create temporary file")]
    FailedToCreateTempFile,

    /// Failed to deserialize file
    #[error("Failed to deserialize file: {0}")]
    FailedToDeserializeFile(PathBuf),

    /// Failed to deserialize [`bitcode`] data
    #[error("Failed to deserialize data: {0:?}")]
    FailedToDeserializeData(Vec<u8>),

    /// Failed to flush temporary file
    #[error("Failed to flush temporary file: {0}")]
    FailedToFlushTempFile(PathBuf),

    /// Failed to get temporary file from writer
    #[error("Failed to get temporary file from writer")]
    FailedToGetInnerWriter,

    /// Failed to open file
    #[error("Failed to open file: {0}")]
    FailedToOpenFile(PathBuf),

    /// Failed to persist temporary file
    #[error("Failed to persist temporary file `{temp}`: {orig}")]
    FailedToPersistTempFile {
        /// The temporary file
        temp: PathBuf,

        /// The original file
        orig: PathBuf,
    },

    /// Failed to read directory
    #[error("Failed to read directory: {0}")]
    FailedToReadDir(PathBuf),

    /// Failed to read file
    #[error("Failed to read file: {0}")]
    FailedToReadFile(PathBuf),

    /// Failed to remove/delete file
    #[error("Failed to remove file: {0}")]
    FailedToRemoveFile(PathBuf),

    /// Failed to reopen temporary file
    #[error("Failed to reopen temporary file: {0}")]
    FailedToReopenTempFile(PathBuf),

    /// Failed to serialize file
    #[error("Failed to serialize file: {0}")]
    FailedToSerializeFile(PathBuf),

    /// Failed to serialize value to [bitcode](https://docs.rs/bitcode)
    #[error("Failed to serialize value")]
    FailedToSerializeValue,

    /// Failed to write file
    #[error("Failed to write to file: {0}")]
    FailedToWriteFile(PathBuf),

    /// Failed to write to temporary file
    #[error("Failed to write to temporary file: {0}")]
    FailedToWriteTempFile(PathBuf),

    /// File does not exist
    #[error("File does not exist: {0}")]
    FileDoesNotExist(PathBuf),

    /// Folder already exists
    #[error("Folder already exists and is not empty: {0}")]
    FolderExists(PathBuf),

    /// Referenced record does not exist
    #[error("Field `{field}` references table `{table}` with ID `{id}`, which does not exist")]
    ForeignKeyViolation {
        /// The field name
        field: String,

        /// The referenced table
        table: String,

        /// The ID of the referenced record
        id: String,
    },

    /// Invalid primary key
    #[error("Invalid primary key: {0}")]
    InvalidKey(String),

    /// Invalid foreign key
    #[error("Field `{field}` references table `{table}` with ID `{id}`, which is invalid")]
    InvalidForeignKey {
        /// The field name
        field: String,

        /// The referenced table
        table: String,

        /// The ID of the referenced record
        id: String,
    },

    /// No path was provided for the database
    #[error("No path was provided for the database")]
    NoDatabasePath,

    /// No tables were found in the database
    #[error("No tables found in database")]
    NoTables,

    /// Record already exists
    #[error("Record already exists for table `{table}` with ID `{id}`")]
    RecordAlreadyExists {
        /// The table name
        table: String,

        /// The ID of the record
        id: String,
    },

    /// Record not found
    #[error("Record not found for table `{table}` with ID `{id}`")]
    RecordNotFound {
        /// The table name
        table: String,

        /// The ID of the record
        id: String,
    },
}
