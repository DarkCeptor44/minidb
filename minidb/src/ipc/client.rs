// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{IpcRequest, IpcResponse, error::Result};
use interprocess::local_socket::{
    GenericFilePath, ToFsName, prelude::LocalSocketStream, traits::Stream,
};
use parking_lot::Mutex;
use std::io::{Read, Write};

/// IPC client
#[derive(Debug)]
pub(crate) struct IpcClient {
    stream: Mutex<LocalSocketStream>,
}

impl IpcClient {
    /// Creates a new IPC client
    ///
    /// ## Arguments
    ///
    /// * `ipc_path` - The path to the IPC server to connect to
    ///
    /// ## Returns
    ///
    /// A new IPC client
    ///
    /// ## Errors
    ///
    /// Returns an error if the connection fails
    pub fn connect(ipc_path: &str) -> Result<Self> {
        let name = ipc_path.to_fs_name::<GenericFilePath>()?;
        let stream = LocalSocketStream::connect(name)?;

        Ok(IpcClient {
            stream: Mutex::new(stream),
        })
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

        let mut stream = self.stream.lock();
        stream.write_all(&len.to_le_bytes())?;
        stream.write_all(&req_bytes)?;
        stream.flush()?;

        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes)?;
        let resp_len = u32::from_le_bytes(len_bytes) as usize;

        let mut resp_bytes = vec![0u8; resp_len];
        stream.read_exact(&mut resp_bytes)?;

        let response: IpcResponse = postcard::from_bytes(&resp_bytes)?;

        Ok(response)
    }
}
