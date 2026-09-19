use minidb::{MiniDB, Table, ipc::IpcServer};
use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use std::{
    thread::{sleep, spawn},
    time::Duration,
};
use tempfile::NamedTempFile;

// If you have the `macros` feature enabled, you can use the derive macro like this:

// #[derive(Debug, Table, Serialize, Deserialize)]
// #[minidb(name = "users")]
// struct User {
//     #[key]
//     id: String,
//     name: String,
// }

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
}

impl Table for User {
    const TABLE: TableDefinition<'_, &'static str, &[u8]> = TableDefinition::new("users");

    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

fn main() {
    #[cfg(windows)]
    let socket_path = r"\\.\pipe\minidb_example";
    #[cfg(unix)]
    let socket_path = "/tmp/minidb_example.sock";

    let temp_file = NamedTempFile::new().unwrap();
    let server_db = MiniDB::builder()
        .path(temp_file.path())
        .table::<User>()
        .open()
        .unwrap();

    let server_socket = socket_path.to_string();
    spawn(move || {
        if let Err(err) = IpcServer::listen(&server_db, server_socket) {
            eprintln!("IPC server error: {err}");
        }
    });

    sleep(Duration::from_millis(100));

    // connects to the database entirely over IPC
    let client_db = MiniDB::builder()
        .ipc_path(socket_path)
        .table::<User>()
        .open()
        .unwrap();

    let mut user = User {
        id: String::new(),
        name: "John Doe".to_string(),
    };

    client_db.insert(&mut user).unwrap();
    let retrieved: Option<User> = client_db.get(&user.id).unwrap();
    println!("Retrieved user via IPC: {retrieved:?}");
}
