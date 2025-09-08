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

use std::collections::HashSet;

use divan::{Bencher, black_box};
use minidb::{Database, Id, Table};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

const T: &[usize] = &[1, 4, 8, 16];

#[derive(Table, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Person {
    #[key]
    id: Id<Self>,
    name: String,
    age: u8,
}

fn main() {
    divan::main();
}

#[divan::bench(threads = T)]
fn new(b: Bencher) {
    b.with_inputs(|| tempdir().expect("Failed to create temp dir"))
        .bench_values(|temp_dir| {
            let db = Database::builder()
                .path(black_box(temp_dir.path()))
                .table::<Person>()
                .build()
                .expect("Failed to build database");
            black_box(db);
        });
}

#[divan::bench(threads = T, args = [1, 1000])]
fn insert(b: Bencher, n: usize) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to create database");

    let p = Person {
        id: Id::new(),
        name: "John Doe".into(),
        age: 31,
    };

    b.bench(|| {
        for _ in 0..n {
            let id = db.insert(black_box(&p)).expect("Failed to insert person");
            black_box(id);
        }
        black_box(());
    });
}

#[divan::bench(threads = T, args = [1, 1000])]
fn update(b: Bencher, n: usize) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to create database");

    b.with_inputs(|| {
        let mut p = Person {
            id: Id::new(),
            name: "John Doe".into(),
            age: 31,
        };
        let id = db.insert(&p).expect("Failed to insert person");
        p.id = id;
        p
    })
    .bench_values(|mut p| {
        for _ in 0..n {
            p.age += 1;
            db.update(black_box(&p)).expect("Failed to update person");
            black_box(());
        }
        black_box(());
    });
}

#[divan::bench(threads = T, args = [1, 1000])]
fn get(b: Bencher, n: usize) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to create database");

    let p = Person {
        id: Id::new(),
        name: "John Doe".into(),
        age: 31,
    };

    let id = db.insert(&p).expect("Failed to insert person");

    b.bench(|| {
        for _ in 0..n {
            let p2 = db.get(black_box(&id)).expect("Failed to get person");
            black_box(p2);
        }
        black_box(());
    });
}

#[divan::bench(threads = T, args = [1, 1000])]
fn delete(b: Bencher, n: usize) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to create database");

    let p = Person {
        id: Id::new(),
        name: "John Doe".into(),
        age: 31,
    };

    b.with_inputs(|| {
        let mut ids = HashSet::new();
        for _ in 0..n {
            ids.insert(db.insert(&p).expect("Failed to insert person"));
        }
        ids
    })
    .bench_values(|ids| {
        for id in &ids {
            db.delete(black_box(id)).expect("Failed to delete person");
            black_box(());
        }
        black_box(());
    });
}

#[divan::bench(threads = T, args = [1, 1000])]
fn exists(b: Bencher, n: usize) {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();
    let db = Database::builder()
        .path(temp_path)
        .table::<Person>()
        .build()
        .expect("Failed to create database");

    let p = Person {
        id: Id::new(),
        name: String::from("John Doe"),
        age: 31,
    };

    let id = db.insert(&p).expect("Failed to insert person");
    b.bench(|| {
        for _ in 0..n {
            black_box(db.exists(black_box(&id)));
        }
        black_box(());
    });
}
