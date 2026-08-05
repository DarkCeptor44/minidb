// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::error::Result;
use interprocess::local_socket::{
    GenericFilePath, ToFsName, prelude::LocalSocketStream, traits::Stream,
};

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
        todo!() // TODO implement IPC request sending
    }
}

/// IPC request
#[derive(Debug, Clone)]
pub enum IpcRequest {
    Insert {
        table: String,
        key: String,
        value: Vec<u8>,
    },
    InsertMany {
        table: String,
        items: Vec<(String, Vec<u8>)>,
    },

    Get {
        table: String,
        key: String,
    },
    GetAll {
        table: String,
    },

    Remove {
        table: String,
        key: String,
    },
    RemoveMany {
        table: String,
        keys: Vec<String>,
    },

    GetMeta {
        key: String,
    },
    SetMeta {
        key: String,
        value: Vec<u8>,
    },

    Update {
        table: String,
        key: String,
        value: Vec<u8>,
    },
    UpdateMany {
        table: String,
        items: Vec<(String, Vec<u8>)>,
    },

    GetSetting {
        key: String,
    },
    SetSetting {
        key: String,
        value: Vec<u8>,
    },

    IsEmpty {
        table: String,
    },

    CreateTable {
        table: String,
    },

    CheckIntegrity,
    Compact,

    BeginTransaction,
    CommitTransaction,
    RollbackTransaction,
}

/// IPC response
#[derive(Debug, Clone)]
pub enum IpcResponse {
    Ok,
    Bool(bool),
    Value(Option<Vec<u8>>),
    Rows(Vec<(String, Vec<u8>)>),
    IsEmpty(bool),
    Error(String),
}
