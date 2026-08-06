// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::error::Result;
use interprocess::local_socket::{
    GenericFilePath, ToFsName, prelude::LocalSocketStream, traits::Stream,
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// IPC client
#[derive(Debug)]
pub struct IpcClient {
    stream: LocalSocketStream,
}

impl IpcClient {
    /// Creates a new IPC client
    ///
    /// ## Arguments
    ///
    /// * `pipe_name` - The name of the pipe to connect to
    ///
    /// ## Returns
    ///
    /// A new IPC client
    ///
    /// ## Errors
    ///
    /// Returns an error if the connection fails
    pub fn connect<S>(pipe_name: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let name = pipe_name.as_ref().to_fs_name::<GenericFilePath>()?;
        let stream = LocalSocketStream::connect(name)?;

        Ok(Self { stream })
    }

    /// Sends a request to the IPC server
    ///
    /// ## Arguments
    ///
    /// * `request` - The request to send
    ///
    /// ## Returns
    ///
    /// A [`Result`] containing the response from the server
    ///
    /// ## Errors
    ///
    /// Returns an error if the request fails
    pub fn send_request(&self, request: &IpcRequest) -> Result<IpcResponse> {
        let req_bytes = postcard::to_stdvec(request)?;
        let len = u32::try_from(req_bytes.len()).unwrap_or(u32::MAX);

        let mut stream_ref = &self.stream;
        stream_ref.write_all(&len.to_le_bytes())?;
        stream_ref.write_all(&req_bytes)?;
        stream_ref.flush()?;

        let mut len_bytes = [0u8; 4];
        stream_ref.read_exact(&mut len_bytes)?;
        let resp_len = u32::from_le_bytes(len_bytes) as usize;

        let mut resp_bytes = vec![0u8; resp_len];
        stream_ref.read_exact(&mut resp_bytes)?;

        let response: IpcResponse = postcard::from_bytes(&resp_bytes)?;

        Ok(response)
    }
}

/// IPC request
#[derive(Debug, Clone, Serialize)]
pub enum IpcRequest {
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
#[derive(Debug, Clone, Deserialize)]
pub enum IpcResponse {
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
