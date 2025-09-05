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

use divan::{Bencher, black_box};
use minidb::{AsTable, Database, Id, Table};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

const T: &[usize] = &[1, 4, 8, 16];

#[derive(Table, Serialize, Deserialize)]
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

#[divan::bench(threads = T)]
fn insert_one(b: Bencher) {
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
        let id = p.insert(black_box(&db)).expect("Failed to insert person");
        black_box(id);
    });
}

#[divan::bench(threads = T)]
fn insert_1000(b: Bencher) {
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
        for _ in 0..1000 {
            let id = p.insert(black_box(&db)).expect("Failed to insert person");
            black_box(id);
        }
        black_box(());
    });
}

#[divan::bench(threads = T)]
fn update(b: Bencher) {
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
        let id = p.insert(&db).expect("Failed to insert person");
        p.id = id;
        p.age += 1;
        p
    })
    .bench_values(|p| {
        p.update(black_box(&db)).expect("Failed to update person");
        black_box(());
    });
}

#[divan::bench(threads = T)]
fn get(b: Bencher) {
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

    let id = p.insert(&db).expect("Failed to insert person");

    b.bench(|| {
        let p2 = Person::get(black_box(&db), black_box(&id)).expect("Failed to get person");
        black_box(p2);
    });
}

#[divan::bench(threads = T)]
fn delete(b: Bencher) {
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

    b.with_inputs(|| p.insert(&db).expect("Failed to insert person"))
        .bench_values(|id| {
            Person::delete(black_box(&db), black_box(&id)).expect("Failed to delete person");
            black_box(());
        });
}