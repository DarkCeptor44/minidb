use std::path::PathBuf;

use thiserror::Error;

/// Represents errors that can occur when using the utilities crate functions
#[derive(Debug, Error, PartialEq, Eq)]
pub enum UtilsError {
    /// Failed to create temporary file
    #[error("Failed to create temporary file")]
    FailedToCreateTempFile,

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

    /// Failed to reopen temporary file
    #[error("Failed to reopen temporary file: {0}")]
    FailedToReopenTempFile(PathBuf),

    /// Failed to serialize value to [bitcode]
    #[error("Failed to serialize value")]
    FailedToSerializeValue,

    /// Failed to write file
    #[error("Failed to write to temporary file: {0}")]
    FailedToWriteTempFile(PathBuf),
}
