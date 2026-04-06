pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur while using MiniDB
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The cipher text is too short
    #[error("ciphertext too short: expected 24 bytes, got {0}")]
    CipherTextTooShort(usize),

    /// Something happened while committing a transaction
    #[error("commit error: {0}")]
    Commit(#[from] redb::CommitError),

    /// Something happened while compacting
    #[error("compacting error: {0}")]
    Compacting(#[from] redb::CompactionError),

    /// Something happened while encrypting or decrypting
    #[error("crypto error: {0}")]
    Crypto(chacha20poly1305::Error),

    /// Something happened while doing database operations
    #[error("database error: {0}")]
    Database(#[from] redb::DatabaseError),

    /// The ID cannot be empty
    #[error("ID cannot be empty")]
    EmptyID,

    /// Something happened while hashing
    #[error("hashing error: {0}")]
    Hashing(argon2::password_hash::Error),

    /// Something happened while serializing to JSON
    #[error("JSON error: {0}")]
    JSON(#[from] serde_json::Error),

    /// The derived key length is incorrect
    #[error("derived key length mismatch: expected 32 bytes, got {0}")]
    KeyLengthMismatch(usize),

    /// Missing hash output
    #[error("missing hash output")]
    MissingHashOutput,

    /// Something happened while serializing or deserializing
    #[error("serialization error: {0}")]
    Serialization(#[from] postcard::Error),

    /// Something happened while doing storage operations
    #[error("storage error: {0}")]
    Storage(#[from] redb::StorageError),

    /// Something happened while doing table operations
    #[error("table error: {0}")]
    Table(#[from] redb::TableError),

    /// Something happened while initializing a table but not using it
    #[error("failed to initialize table `{name}`: {source}")]
    TableInitialization {
        /// The name of the table
        name: String,

        /// The underlying error
        #[source]
        source: redb::TableError,
    },

    /// Something happened while doing transaction operations
    #[error("transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),
}

impl From<argon2::password_hash::Error> for Error {
    fn from(e: argon2::password_hash::Error) -> Self {
        Error::Hashing(e)
    }
}

impl From<chacha20poly1305::Error> for Error {
    fn from(e: chacha20poly1305::Error) -> Self {
        Error::Crypto(e)
    }
}
