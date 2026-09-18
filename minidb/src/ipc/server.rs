// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    Backend, Error, IpcRequest, IpcResponse, META_TABLE, MiniDB, SETTINGS_TABLE, error::Result,
};
use interprocess::local_socket::{
    GenericFilePath, ListenerOptions, ToFsName, prelude::LocalSocketStream, traits::ListenerExt,
};
use postcard::{from_bytes, to_stdvec};
use redb::{
    Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition, TableError,
    TableHandle, WriteTransaction,
};
use std::io::{Error as IoError, Read, Result as IoResult, Write};

macro_rules! run_write_op {
    ($local_db:expr, $active_txn:expr, $table:expr, |$t:ident| $body:expr) => {
        if let Some(txn) = $active_txn.as_ref() {
            let mut $t = txn.open_table(get_table(&$table))?;
            let val = $body;
            Ok(val)
        } else {
            let txn = $local_db.begin_write()?;
            let val = {
                let mut $t = txn.open_table(get_table(&$table))?;
                $body
            };
            txn.commit()?;
            Ok(val)
        }
    };
}

macro_rules! run_read_op {
    ($local_db:expr, $active_txn:expr, $table:expr, |$t:ident| $body:expr) => {{
        let __res: crate::error::Result<_> = if let Some(txn) = $active_txn.as_ref() {
            match txn.open_table(get_table(&$table)) {
                Ok($t) => Ok(Some($body)),
                Err(TableError::TableDoesNotExist(_)) => Ok(None),
                Err(e) => Err(e.into()),
            }
        } else {
            let txn = $local_db.begin_read()?;
            match txn.open_table(get_table(&$table)) {
                Ok($t) => Ok(Some($body)),
                Err(TableError::TableDoesNotExist(_)) => Ok(None),
                Err(e) => Err(e.into()),
            }
        };
        __res
    }};
}

/// IPC server helper
///
/// This takes a reference to the database and a path to listen on, making sure the first process has the exclusive lock on the database file. Then other processes can connect directly to the database using the IPC path. Check the example below
///
/// ## Examples
///
/// ```rust,no_run
/// use minidb::{MiniDB, Table};
/// use minidb::ipc::IpcServer;
/// use serde::{Serialize, Deserialize};
/// use std::time::Duration;
///
/// const IPC_PATH: &str = r"\\.\pipe\my_ipc_server";
///
/// #[derive(Serialize, Deserialize)]
/// struct Person {
///     id: String,
///     name: String,
///     age: u8,
/// }
///
/// impl Table for Person {
///     const TABLE: minidb::TableDefinition<'_, &'static str, &[u8]> = minidb::TableDefinition::new("people");
///
///     fn get_id(&self) -> &str {
///         &self.id
///     }
///
///     fn set_id(&mut self, id: String) {
///         self.id = id;
///     }
/// }
///
/// let mut db1 = MiniDB::builder()
///     .path("path/to/db")
///     .table::<Person>()
///     .open()
///     .unwrap();
///
/// std::thread::spawn(move || {
///         IpcServer::listen(&mut db1, IPC_PATH).expect("failed to listen to IPC server");
///     });
/// std::thread::sleep(Duration::from_millis(50));
///
/// let db2 = MiniDB::builder()
///     .table::<Person>()
///     .ipc_path(IPC_PATH)
///     .open()
///     .unwrap();
///
/// // do stuff with db2
/// ```
///
/// Alternatively, you can keep a single database instance and any outside process running the same code will automatically use IPC as fallback:
///
/// ```rust,no_run
/// use minidb::{MiniDB, Table};
/// use minidb::ipc::IpcServer;
/// use serde::{Serialize, Deserialize};
/// use std::time::Duration;
///
/// const IPC_PATH: &str = r"\\.\pipe\my_ipc_server";
///
/// #[derive(Serialize, Deserialize)]
/// struct Person {
///     id: String,
///     name: String,
///     age: u8,
/// }
///
/// impl Table for Person {
///     const TABLE: minidb::TableDefinition<'_, &'static str, &[u8]> = minidb::TableDefinition::new("people");
///
///     fn get_id(&self) -> &str {
///         &self.id
///     }
///
///     fn set_id(&mut self, id: String) {
///         self.id = id;
///     }
/// }
///
/// let mut db = MiniDB::builder()
///     .path("path/to/db")
///     .table::<Person>()
///     .ipc_path(IPC_PATH)
///     .open()
///     .unwrap();
///
/// std::thread::spawn(move || {
///         IpcServer::listen(&mut db, IPC_PATH).expect("failed to listen to IPC server");
///     });
/// std::thread::sleep(Duration::from_millis(50));
///
/// // do stuff with db
/// ```
#[derive(Debug)]
pub struct IpcServer;

fn get_table(name: &str) -> TableDefinition<'_, &'static str, &'static [u8]> {
    TableDefinition::new(name)
}

impl IpcServer {
    /// Listens for IPC requests at the specified path
    ///
    /// ## Arguments
    ///
    /// `ipc_path` - The path to the IPC server
    ///
    /// ## Errors
    ///
    /// Returns an error if the connection fails
    pub fn listen<S>(db: &MiniDB, ipc_path: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        let name = ipc_path.as_ref().to_fs_name::<GenericFilePath>()?;
        let listener = ListenerOptions::new().name(name).create_sync()?;

        let local_db = match &db.backend {
            Backend::Local(db) => db,
            Backend::Ipc(_) => {
                return Err(Error::Ipc(
                    "Cannot run IPC server on an IPC client database".to_string(),
                ));
            }
        };

        std::thread::scope(|s| {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                s.spawn(move || {
                    Self::handle_connection(local_db, &mut stream);
                });
            }
        });

        Ok(())
    }

    fn handle_connection(local_db: &Database, stream: &mut LocalSocketStream) {
        let mut active_txn: Option<WriteTransaction> = None;

        loop {
            let mut len_bytes = [0u8; 4];
            if stream.read_exact(&mut len_bytes).is_err() {
                break;
            }
            let len = u32::from_le_bytes(len_bytes) as usize;

            let mut req_bytes = vec![0u8; len];
            if stream.read_exact(&mut req_bytes).is_err() {
                break;
            }

            let request: IpcRequest = match from_bytes(&req_bytes) {
                Ok(req) => req,
                Err(e) => {
                    let _ = Self::send_response(stream, &IpcResponse::Error(e.to_string()));
                    continue;
                }
            };

            let response = Self::handle_request(local_db, &mut active_txn, request);

            if Self::send_response(stream, &response).is_err() {
                break;
            }
        }
    }

    fn handle_request(
        local_db: &Database,
        active_txn: &mut Option<WriteTransaction>,
        request: IpcRequest,
    ) -> IpcResponse {
        let res = match request {
            IpcRequest::BeginTransaction
            | IpcRequest::CommitTransaction
            | IpcRequest::RollbackTransaction => {
                Ok(Self::handle_txn_op(local_db, active_txn, &request))
            }
            IpcRequest::CheckIntegrity | IpcRequest::Compact => {
                Ok(Self::handle_db_op(local_db, active_txn.as_ref(), &request))
            }
            IpcRequest::Get { .. }
            | IpcRequest::GetAll { .. }
            | IpcRequest::GetMeta { .. }
            | IpcRequest::GetSetting { .. }
            | IpcRequest::IsEmpty { .. } => Self::handle_read_op(local_db, active_txn, request),
            _ => Self::handle_write_op(local_db, active_txn, request),
        };
        match res {
            Ok(resp) => resp,
            Err(e) => IpcResponse::Error(e.to_string()),
        }
    }

    fn handle_txn_op(
        local_db: &Database,
        active_txn: &mut Option<WriteTransaction>,
        request: &IpcRequest,
    ) -> IpcResponse {
        match request {
            IpcRequest::BeginTransaction => {
                if active_txn.is_some() {
                    IpcResponse::Error("Transaction already in progress".to_string())
                } else {
                    match local_db.begin_write() {
                        Ok(txn) => {
                            *active_txn = Some(txn);
                            IpcResponse::Ok
                        }
                        Err(e) => IpcResponse::Error(e.to_string()),
                    }
                }
            }
            IpcRequest::CommitTransaction => {
                if let Some(txn) = active_txn.take() {
                    match txn.commit() {
                        Ok(()) => IpcResponse::Ok,
                        Err(e) => IpcResponse::Error(e.to_string()),
                    }
                } else {
                    IpcResponse::Error("No active transaction to commit".to_string())
                }
            }
            IpcRequest::RollbackTransaction => {
                if active_txn.take().is_some() {
                    IpcResponse::Ok
                } else {
                    IpcResponse::Error("No active transaction to rollback".to_string())
                }
            }
            _ => unreachable!(),
        }
    }

    fn handle_db_op(
        _local_db: &Database,
        active_txn: Option<&WriteTransaction>,
        request: &IpcRequest,
    ) -> IpcResponse {
        if active_txn.is_some() {
            return IpcResponse::Error(
                "Cannot perform database maintenance while a transaction is in progress"
                    .to_string(),
            );
        }

        match request {
            IpcRequest::CheckIntegrity => IpcResponse::Error(
                "Checking integrity is not supported over multi-client IPC".to_string(),
            ),
            IpcRequest::Compact => IpcResponse::Error(
                "Compacting database is not supported over multi-client IPC".to_string(),
            ),
            _ => unreachable!(),
        }
    }

    fn handle_read_op(
        local_db: &Database,
        active_txn: &mut Option<WriteTransaction>,
        request: IpcRequest,
    ) -> Result<IpcResponse> {
        match request {
            IpcRequest::Get { table, key } => {
                let res = run_read_op!(local_db, active_txn, table, |t| {
                    t.get(&*key)?.map(|v| v.value().to_vec())
                })?;
                Ok(IpcResponse::Value(res.flatten()))
            }
            IpcRequest::GetAll { table } => {
                let res = run_read_op!(local_db, active_txn, table, |t| {
                    let mut rows = Vec::new();
                    for item in t.iter()? {
                        let (k, v) = item?;
                        rows.push((k.value().to_string(), v.value().to_vec()));
                    }
                    rows
                })?;

                match res {
                    Some(rows) => Ok(IpcResponse::Rows(rows)),
                    None => Ok(IpcResponse::Rows(Vec::new())),
                }
            }
            IpcRequest::GetMeta { key } => {
                let res = run_read_op!(local_db, active_txn, META_TABLE.name(), |t| {
                    t.get(&*key)?.map(|v| v.value().to_vec())
                })?;
                Ok(IpcResponse::Value(res.flatten()))
            }
            IpcRequest::GetSetting { key } => {
                let res = run_read_op!(local_db, active_txn, SETTINGS_TABLE.name(), |t| {
                    t.get(&*key)?.map(|v| v.value().to_vec())
                })?;
                Ok(IpcResponse::Value(res.flatten()))
            }
            IpcRequest::IsEmpty { table } => {
                let res = run_read_op!(local_db, active_txn, table, |t| t.len()? == 0)?;

                match res {
                    Some(is_empty) => Ok(IpcResponse::Bool(is_empty)),
                    None => Ok(IpcResponse::Bool(true)),
                }
            }
            _ => unreachable!(),
        }
    }

    fn handle_write_op(
        local_db: &Database,
        active_txn: &mut Option<WriteTransaction>,
        request: IpcRequest,
    ) -> Result<IpcResponse> {
        match request {
            IpcRequest::Insert { table, key, value } | IpcRequest::Update { table, key, value } => {
                run_write_op!(local_db, active_txn, table, |t| {
                    t.insert(&*key, value.as_slice())?;
                    IpcResponse::Ok
                })
            }
            IpcRequest::InsertMany { table, items } | IpcRequest::UpdateMany { table, items } => {
                run_write_op!(local_db, active_txn, table, |t| {
                    for (key, value) in items {
                        t.insert(&*key, value.as_slice())?;
                    }
                    IpcResponse::Ok
                })
            }
            IpcRequest::Remove { table, key } => {
                run_write_op!(local_db, active_txn, table, |t| {
                    let val = t.remove(&*key)?.map(|v| v.value().to_vec());
                    IpcResponse::Value(val)
                })
            }
            IpcRequest::RemoveMany { table, keys } => {
                run_write_op!(local_db, active_txn, table, |t| {
                    let mut rows = Vec::new();
                    for key in keys {
                        if let Some(val) = t.remove(&*key)? {
                            rows.push((key, val.value().to_vec()));
                        }
                    }
                    IpcResponse::Rows(rows)
                })
            }
            IpcRequest::SetMeta { key, value } => {
                run_write_op!(local_db, active_txn, META_TABLE.name(), |t| {
                    t.insert(&*key, value.as_slice())?;
                    IpcResponse::Ok
                })
            }
            IpcRequest::SetSetting { key, value } => {
                run_write_op!(local_db, active_txn, SETTINGS_TABLE.name(), |t| {
                    t.insert(&*key, value.as_slice())?;
                    IpcResponse::Ok
                })
            }
            IpcRequest::CreateTable { table } => {
                if let Some(txn) = active_txn.as_ref() {
                    let _ = txn.open_table(get_table(&table))?;
                    Ok(IpcResponse::Ok)
                } else {
                    let txn = local_db.begin_write()?;
                    let _ = txn.open_table(get_table(&table))?;
                    txn.commit()?;
                    Ok(IpcResponse::Ok)
                }
            }
            _ => unreachable!(),
        }
    }

    fn send_response(stream: &mut LocalSocketStream, response: &IpcResponse) -> IoResult<()> {
        let resp_bytes = to_stdvec(response).map_err(IoError::other)?;
        let len = u32::try_from(resp_bytes.len()).unwrap_or(u32::MAX);
        stream.write_all(&len.to_le_bytes())?;
        stream.write_all(&resp_bytes)?;
        stream.flush()?;
        Ok(())
    }
}
