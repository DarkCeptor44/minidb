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

    /// Failed to write metadata
    #[error("Failed to write metadata")]
    FailedToWriteMetadata,

    /// Folder already exists
    #[error("Folder already exists and is not empty: {0}")]
    FolderExists(PathBuf),

    /// No path was provided for the database
    #[error("No path was provided for the database")]
    NoDatabasePath,

    /// No tables were found in the database
    #[error("No tables found in database")]
    NoTables,
}
