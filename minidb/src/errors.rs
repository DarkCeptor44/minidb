use std::path::PathBuf;

use thiserror::Error;

/// Represents errors that can occur when using the database's methods
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DBError {
    /// Failed to create database
    #[error("Failed to create database directory: {0}")]
    FailedToCreateDatabase(PathBuf),

    /// Failed to create table
    #[error("Failed to create table directory: {0}")]
    FailedToCreateTableDir(PathBuf),

    /// Failed to read metadata
    #[error("Failed to read metadata")]
    FailedToReadMetadata,

    /// Failed to serialize file
    #[error("Failed to serialize file: {0}")]
    FailedToSerializeFile(PathBuf),

    /// Failed to write metadata
    #[error("Failed to write metadata")]
    FailedToWriteMetadata,

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

    /// Metadata not found
    #[error("Metadata not found")]
    NoMetadata,

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
}
