use divan::{Bencher, black_box};
use minidb::{KeySource, MiniDB, TableModel};
use rand::{RngExt, seq::IndexedRandom};
use redb::TableDefinition;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

const KEY: [u8; 32] = [1u8; 32];

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Person {
    id: String,
    name: String,
    age: u8,
}

impl TableModel for Person {
    const TABLE: redb::TableDefinition<'_, &'static str, &[u8]> = TableDefinition::new("people");

    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

fn main() {
    divan::main();
}

#[divan::bench]
fn new(b: Bencher) {
    b.with_inputs(|| NamedTempFile::new().unwrap())
        .bench_values(|temp_file| {
            let db = MiniDB::builder(temp_file.path())
                .table::<Person>()
                .key_source(KeySource::PreDerived(KEY))
                .build()
                .unwrap();
            black_box(db);
        });
}

#[divan::bench(name = "insert (fresh db)")]
fn insert_into_fresh_db(b: Bencher) {
    b.with_inputs(|| {
        let temp_file = NamedTempFile::new().unwrap();
        let db = MiniDB::builder(temp_file.path())
            .table::<Person>()
            .key_source(KeySource::PreDerived(KEY))
            .build()
            .unwrap();

        let mut p = Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        };

        p.id = String::new();

        (temp_file, db, p)
    })
    .bench_refs(|(_temp_file, db, p)| {
        db.insert(p).unwrap();
    });
}

#[divan::bench(name = "insert (existing db)")]
fn insert_into_existing_db(b: Bencher) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    b.with_inputs(|| Person {
        id: String::new(),
        name: "John Doe".to_string(),
        age: 31,
    })
    .bench_refs(|p| {
        db.insert(p).unwrap();
    });
}

#[divan::bench(name = "insert_many (fresh db)", args = [1, 1000, 10000])]
fn insert_many_into_fresh_db(b: Bencher, n: usize) {
    b.with_inputs(|| {
        let temp_file = NamedTempFile::new().unwrap();
        let db = MiniDB::builder(temp_file.path())
            .table::<Person>()
            .key_source(KeySource::PreDerived(KEY))
            .build()
            .unwrap();

        let mut people = Vec::with_capacity(n);
        for _ in 0..n {
            people.push(Person {
                id: String::new(),
                name: "John Doe".to_string(),
                age: 31,
            });
        }
        (temp_file, db, people)
    })
    .bench_refs(|(_temp_file, db, people)| {
        db.insert_many(people).unwrap();
    });
}

#[divan::bench(name = "insert_many (existing db)", args = [1, 1000, 10000])]
fn insert_many_into_existing_db(b: Bencher, n: usize) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    b.with_inputs(|| {
        let mut people = Vec::with_capacity(n);
        for _ in 0..n {
            people.push(Person {
                id: String::new(),
                name: "John Doe".to_string(),
                age: 31,
            });
        }
        people
    })
    .bench_refs(|people| {
        db.insert_many(people).unwrap();
    });
}

#[divan::bench(name = "update (fresh db)")]
fn update_into_fresh_db(b: Bencher) {
    b.with_inputs(|| {
        let temp_file = NamedTempFile::new().unwrap();
        let db = MiniDB::builder(temp_file.path())
            .table::<Person>()
            .key_source(KeySource::PreDerived(KEY))
            .build()
            .unwrap();

        let mut p = Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        };

        db.insert(&mut p).unwrap();

        p.age = 32;

        (temp_file, db, p)
    })
    .bench_refs(|(_temp_file, db, p)| {
        db.update(p).unwrap();
    });
}

#[divan::bench(name = "update (existing db)")]
fn update_into_existing_db(b: Bencher) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    b.with_inputs(|| {
        let mut p = Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        };

        db.insert(&mut p).unwrap();

        p.age = 32;

        p
    })
    .bench_refs(|p| {
        db.update(p).unwrap();
    });
}

#[divan::bench(name = "update_many (fresh db)", args = [1, 1000, 10000])]
fn update_many_into_fresh_db(b: Bencher, n: usize) {
    b.with_inputs(|| {
        let temp_file = NamedTempFile::new().unwrap();
        let db = MiniDB::builder(temp_file.path())
            .table::<Person>()
            .key_source(KeySource::PreDerived(KEY))
            .build()
            .unwrap();

        let mut people = Vec::with_capacity(n);
        for i in 0..n {
            people.push(Person {
                id: String::new(),
                name: format!("John Doe {}", i),
                age: 31,
            });
        }
        db.insert_many(&mut people).unwrap();
        for p in &mut people {
            p.age = 32;
        }
        (temp_file, db, people)
    })
    .bench_refs(|(_temp_file, db, people)| {
        db.update_many(people).unwrap();
    });
}

#[divan::bench(name = "update_many (existing db)", args = [1, 1000, 10000])]
fn update_many_into_existing_db(b: Bencher, n: usize) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    b.with_inputs(|| {
        let mut people = Vec::with_capacity(n);
        for i in 0..n {
            people.push(Person {
                id: String::new(),
                name: format!("John Doe {}", i),
                age: 31,
            });
        }
        db.insert_many(&mut people).unwrap();
        for p in &mut people {
            p.age = 32;
        }
        people
    })
    .bench_refs(|people| {
        db.update_many(people).unwrap();
    });
}

#[divan::bench]
fn get(b: Bencher) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    let mut person = Person {
        id: String::new(),
        name: "John Doe".to_string(),
        age: 31,
    };
    db.insert(&mut person).unwrap();

    let id = person.id.clone();

    b.bench(|| {
        let res = db.get::<Person>(&id).unwrap();
        black_box(res);
    });
}

#[divan::bench(name = "get (one from large db)", args = [100, 1000])]
fn get_one_from_large_db(b: Bencher, n: usize) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    let mut people = Vec::with_capacity(n);
    for _ in 0..n {
        people.push(Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        });
    }
    db.insert_many(&mut people).unwrap();

    let mut rng = rand::rng();
    let random_person = people.choose(&mut rng).unwrap();
    let id = random_person.id.clone();

    b.bench(|| {
        let res = db.get::<Person>(&id).unwrap();
        black_box(res);
    });
}

#[divan::bench(args = [1000, 10000, 100000])]
fn all(b: Bencher, n: usize) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    let mut people = Vec::with_capacity(n);
    for _ in 0..n {
        people.push(Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        });
    }
    db.insert_many(&mut people).unwrap();

    b.bench(|| {
        let res = db.all::<Person>().unwrap();
        black_box(res);
    });
}

#[divan::bench(name = "remove (fresh db)")]
fn remove_from_fresh_db(b: Bencher) {
    b.with_inputs(|| {
        let temp_file = NamedTempFile::new().unwrap();
        let db = MiniDB::builder(temp_file.path())
            .table::<Person>()
            .key_source(KeySource::PreDerived(KEY))
            .build()
            .unwrap();

        let mut p = Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        };

        db.insert(&mut p).unwrap();
        (temp_file, db, p.id)
    })
    .bench_refs(|(_temp_file, db, p)| {
        let res = db.remove::<Person>(p).unwrap();
        black_box(res);
    });
}

#[divan::bench(name = "remove (existing db)")]
fn remove_from_existing_db(b: Bencher) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    b.with_inputs(|| {
        let mut p = Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        };

        db.insert(&mut p).unwrap();
        p.id
    })
    .bench_refs(|p| {
        let res = db.remove::<Person>(p).unwrap();
        black_box(res);
    });
}

#[divan::bench(name = "remove_many (fresh db)",args = [1, 1000, 10000])]
fn remove_many_from_fresh_db(b: Bencher, n: usize) {
    b.with_inputs(|| {
        let temp_file = NamedTempFile::new().unwrap();
        let db = MiniDB::builder(temp_file.path())
            .table::<Person>()
            .key_source(KeySource::PreDerived(KEY))
            .build()
            .unwrap();

        let mut people = Vec::with_capacity(n);
        for _ in 0..n {
            people.push(Person {
                id: String::new(),
                name: "John Doe".to_string(),
                age: 31,
            });
        }

        db.insert_many(&mut people).unwrap();
        let ids: Vec<String> = people.iter().map(|p| p.id.clone()).collect();

        (temp_file, db, ids)
    })
    .bench_values(|(_temp_file, db, ids)| {
        let ids_str: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let res = db.remove_many::<Person>(&ids_str).unwrap();
        black_box(res);
    });
}

#[divan::bench(name = "remove_many (existing db)", args = [1, 1000, 10000])]
fn remove_many_from_existing_db(b: Bencher, n: usize) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    b.with_inputs(|| {
        let mut people = Vec::with_capacity(n);
        for _ in 0..n {
            people.push(Person {
                id: String::new(),
                name: "John Doe".to_string(),
                age: 31,
            });
        }

        db.insert_many(&mut people).unwrap();
        let ids: Vec<String> = people.iter().map(|p| p.id.clone()).collect();

        ids
    })
    .bench_values(|ids| {
        let ids_str: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let res = db.remove_many::<Person>(&ids_str).unwrap();
        black_box(res);
    });
}

#[divan::bench(name = "for_each", args = [1, 1000, 10000])]
fn for_each(b: Bencher, n: usize) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    let mut people = Vec::with_capacity(n);
    for _ in 0..n {
        people.push(Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        });
    }

    db.insert_many(&mut people).unwrap();

    b.bench(|| {
        db.for_each(|p: &Person| {
            black_box(p);
        })
        .unwrap();
    });
}

#[divan::bench(args = [(1000, false), (1000, true), (10000, false), (10000, true)])]
fn export_table(b: Bencher, args: (usize, bool)) {
    let (n, pretty) = args;
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    let mut people = Vec::with_capacity(n);
    for _ in 0..n {
        people.push(Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        });
    }

    db.insert_many(&mut people).unwrap();

    b.bench(|| {
        let res = db.export_table::<Person>(pretty).unwrap();
        black_box(res);
    });
}

#[divan::bench]
fn is_empty(b: Bencher) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    let mut people = Vec::with_capacity(1000);
    for _ in 0..1000 {
        people.push(Person {
            id: String::new(),
            name: "John Doe".to_string(),
            age: 31,
        });
    }
    db.insert_many(&mut people).unwrap();

    b.bench(|| {
        let res = db.is_empty::<Person>().unwrap();
        black_box(res);
    });
}

#[divan::bench]
fn create_table(b: Bencher) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    b.bench(|| db.create_table::<Person>().unwrap());
}

#[divan::bench]
fn get_setting(b: Bencher) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    db.set_setting("test", &1234).unwrap();

    b.bench(|| {
        let res = db.get_setting::<i32>("test").unwrap();
        black_box(res);
    });
}

#[divan::bench]
fn set_setting(b: Bencher) {
    let temp_file = NamedTempFile::new().unwrap();
    let db = MiniDB::builder(temp_file.path())
        .table::<Person>()
        .key_source(KeySource::PreDerived(KEY))
        .build()
        .unwrap();

    b.with_inputs(|| {
        let mut rng = rand::rng();
        format!("test{}", rng.random_range(0..10000000))
    })
    .bench_refs(|key| {
        db.set_setting(key, &1234).unwrap();
    });
}
