// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! # IPC
//!
//! This module provides an IPC implementation for MiniDB.
//!
//! **Note**: This module is **experimental** because it was mostly written by AI, but tested by me.

mod client;
mod server;

pub(crate) use client::IpcClient;
pub use server::IpcServer;

use serde::{Deserialize, Serialize};

/// IPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum IpcRequest {
    /// Inserts a key-value pair into the database
    Insert {
        /// The table to insert into
        table: String,

        /// The key to insert
        key: String,

        /// The value to insert
        value: Vec<u8>,
    },

    /// Inserts multiple key-value pairs into the database
    InsertMany {
        /// The table to insert into
        table: String,

        /// The key-value pairs to insert
        items: Vec<(String, Vec<u8>)>,
    },

    /// Gets a key-value pair from the database
    Get {
        /// The table to get from
        table: String,

        /// The key to get
        key: String,
    },

    /// Gets all key-value pairs from the database
    GetAll {
        /// The table to get from
        table: String,
    },

    /// Removes a key-value pair from the database
    Remove {
        /// The table to remove from
        table: String,

        /// The key to remove
        key: String,
    },

    /// Removes multiple key-value pairs from the database
    RemoveMany {
        /// The table to remove from
        table: String,

        /// The keys to remove
        keys: Vec<String>,
    },

    /// Gets a value from the meta table
    GetMeta {
        /// The key to get
        key: String,
    },

    /// Sets a value in the meta table
    SetMeta {
        /// The key to set
        key: String,

        /// The value to set
        value: Vec<u8>,
    },

    /// Updates a key-value pair in the database
    Update {
        /// The table to update
        table: String,

        /// The key to update
        key: String,

        /// The value to update
        value: Vec<u8>,
    },

    /// Updates multiple key-value pairs in the database
    UpdateMany {
        /// The table to update
        table: String,

        /// The key-value pairs to update
        items: Vec<(String, Vec<u8>)>,
    },

    /// Gets a value from the settings table
    GetSetting {
        /// The key to get
        key: String,
    },

    /// Sets a value in the settings table
    SetSetting {
        /// The key to set
        key: String,

        /// The value to set
        value: Vec<u8>,
    },

    /// Checks if a table exists
    IsEmpty {
        /// The table to check
        table: String,
    },

    /// Creates a new table
    CreateTable {
        /// The table to create
        table: String,
    },

    /// Checks the integrity of the database
    CheckIntegrity,

    /// Compacts the database
    Compact,

    /// Begins a transaction
    BeginTransaction,

    /// Commits a transaction
    CommitTransaction,

    /// Rolls back a transaction
    RollbackTransaction,
}

/// IPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum IpcResponse {
    /// A successful response
    Ok,

    /// A boolean response
    Bool(bool),

    /// A [Vec<u8>] response
    Value(Option<Vec<u8>>),

    /// A [Vec<(String, Vec<u8>)>] response
    Rows(Vec<(String, Vec<u8>)>),

    /// An error response
    Error(String),
}
