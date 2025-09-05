// Copyright (c) 2025, DarkCeptor44
//
// This file is licensed under the GNU Lesser General Public License
// (either version 3 or, at your option, any later version).
//
// This software comes without any warranty, express or implied. See the
// GNU Lesser General Public License for details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this software. If not, see <https://www.gnu.org/licenses/>.

use minidb_utils::{deserialize_file, read_from_file, serialize_file};
use serde::{Deserialize, Serialize};
use tempfile::{tempdir, NamedTempFile};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Person {
    name: String,
    age: u8,
}

#[test]
fn test_deserialize_file() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let path = temp_dir.path().join("test");
    let p = Person {
        name: "John Doe".to_string(),
        age: 31,
    };

    serialize_file(&path, &p).expect("Failed to serialize file");
    assert!(path.is_file());

    let p2: Person = deserialize_file(path).expect("Failed to deserialize file");
    assert_eq!(p2, p);
}

#[tokio::test]
#[cfg(feature = "tokio")]
async fn test_deserialize_file_async() {
    use minidb_utils::{deserialize_file_async, serialize_file_async};

    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let path = temp_dir.path().join("test");
    let p = Person {
        name: "John Doe".to_string(),
        age: 31,
    };

    serialize_file_async(&path, &p)
        .await
        .expect("Failed to serialize file");
    assert!(path.is_file());

    let p2: Person = deserialize_file_async(&path)
        .await
        .expect("Failed to deserialize file");
    assert_eq!(p2, p);
}

#[test]
fn test_read_from_file() {
    const CONTENT: &str = "Hello world";

    let file = NamedTempFile::new().expect("Failed to create temporary file");
    let path = file.path();

    std::fs::write(path, CONTENT).expect("Failed to write to file");

    let s = read_from_file(path).expect("Failed to read file");
    assert_eq!(s, CONTENT);
}

#[tokio::test]
#[cfg(feature = "tokio")]
async fn test_read_from_file_async() {
    use minidb_utils::read_from_file_async;

    const CONTENT: &str = "Hello world";

    let file = NamedTempFile::new().expect("Failed to create temporary file");
    let path = file.path();

    tokio::fs::write(path, CONTENT)
        .await
        .expect("Failed to write to file");

    let s = read_from_file_async(path)
        .await
        .expect("Failed to read file");
    assert_eq!(s, CONTENT);
}

#[test]
fn test_serialize_file() {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let path = temp_dir.path().join("test");
    let p = Person {
        name: "John Doe".to_string(),
        age: 31,
    };

    serialize_file(&path, &p).expect("Failed to serialize file");
    assert!(path.is_file());

    let s = read_from_file(&path).expect("Failed to read file");
    assert_eq!(s, "\u{8}John Doe\u{1f}");
}

#[tokio::test]
#[cfg(feature = "tokio")]
async fn test_serialize_file_async() {
    use minidb_utils::{read_from_file_async, serialize_file_async};

    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let path = temp_dir.path().join("test");
    let p = Person {
        name: "John Doe".to_string(),
        age: 31,
    };

    serialize_file_async(&path, &p)
        .await
        .expect("Failed to serialize file");
    assert!(path.is_file());

    let s = read_from_file_async(&path)
        .await
        .expect("Failed to read file");
    assert_eq!(s, "\u{8}John Doe\u{1f}");
}