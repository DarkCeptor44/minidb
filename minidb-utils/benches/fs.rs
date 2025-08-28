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

use divan::{black_box, Bencher};
use minidb_utils as utils;
use serde::{Deserialize, Serialize};
use tempfile::{tempdir, NamedTempFile};
use tokio::runtime::Runtime;

const N: u64 = 1024;

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: u8,
}

fn main() {
    divan::main();
}

#[divan::bench]
fn deserialize_file(b: Bencher) {
    b.with_inputs(|| {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let path = temp_dir.path().join("test");
        let p = Person {
            name: "John Doe".to_string(),
            age: 31,
        };

        utils::serialize_file(&path, &p).expect("Failed to serialize file");

        (temp_dir, path)
    })
    .bench_values(|(_temp_dir, path)| {
        let p2: Person =
            utils::deserialize_file(black_box(path)).expect("Failed to deserialize file");
        black_box(p2);
    });
}

#[divan::bench]
fn deserialize_file_async(b: Bencher) {
    let rt = Runtime::new().expect("Failed to create runtime");

    b.with_inputs(|| {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let path = temp_dir.path().join("test");
        let p = Person {
            name: "John Doe".to_string(),
            age: 31,
        };

        utils::serialize_file(&path, &p).expect("Failed to serialize file");

        (temp_dir, path)
    })
    .bench_values(|(_temp_dir, path)| {
        let p2: Person = rt
            .block_on(utils::deserialize_file_async(black_box(path)))
            .expect("Failed to deserialize file");
        black_box(p2);
    });
}

#[divan::bench]
fn read_from_file(b: Bencher) {
    b.with_inputs(|| {
        let content = padding(N);
        let file = NamedTempFile::new().expect("Failed to create temporary file");
        let path = file.path().to_path_buf();

        std::fs::write(&path, content).expect("Failed to write to file");

        (file, path)
    })
    .bench_values(|(_file, path)| {
        let s = utils::read_from_file(black_box(path)).expect("Failed to read file");
        black_box(s);
    });
}

#[divan::bench]
fn read_from_file_async(b: Bencher) {
    let rt = Runtime::new().expect("Failed to create runtime");

    b.with_inputs(|| {
        let content = padding(N);
        let file = NamedTempFile::new().expect("Failed to create temporary file");
        let path = file.path().to_path_buf();

        std::fs::write(&path, content).expect("Failed to write to file");

        (file, path)
    })
    .bench_values(|(_file, path)| {
        let s = rt
            .block_on(utils::read_from_file_async(black_box(path)))
            .expect("Failed to read file");
        black_box(s)
    });
}

#[divan::bench]
fn serialize_file(b: Bencher) {
    b.with_inputs(|| {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let path = temp_dir.path().join("test");
        let p = Person {
            name: "John Doe".to_string(),
            age: 31,
        };

        (temp_dir, path, p)
    })
    .bench_values(|(_temp_dir, path, p)| {
        utils::serialize_file(black_box(path), black_box(&p)).expect("Failed to serialize file");
        black_box(());
    });
}

#[divan::bench]
fn serialize_file_async(b: Bencher) {
    let rt = Runtime::new().expect("Failed to create runtime");

    b.with_inputs(|| {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let path = temp_dir.path().join("test");
        let p = Person {
            name: "John Doe".to_string(),
            age: 31,
        };

        (temp_dir, path, p)
    })
    .bench_values(|(_temp_dir, path, p)| {
        rt.block_on(utils::serialize_file_async(black_box(path), black_box(&p)))
            .expect("Failed to serialize file");
        black_box(());
    });
}

/// Returns a vector of bytes that is `bytes` long
fn padding(bytes: u64) -> Vec<u8> {
    const CONTENT: &str = "content8adsaasdadasdadlklaklskdklaslkd";

    CONTENT
        .as_bytes()
        .iter()
        .cycle()
        .take(bytes as usize)
        .cloned()
        .collect()
}
