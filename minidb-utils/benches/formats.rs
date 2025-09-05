use divan::{Bencher, black_box};
use rand::Rng;
use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};

const NUM_PEOPLE: usize = 10000;

#[derive(Serialize, Deserialize, Writable, Readable)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

fn main() {
    divan::main();
}

/// Generate a vector of fake people
fn fake_people(n: Option<usize>) -> Vec<Person> {
    let mut rng = rand::rng();

    (1..=n.unwrap_or(NUM_PEOPLE))
        .map(|i| Person {
            name: format!("Person {i}"),
            age: rng.random_range(18..100),
        })
        .collect()
}

macro_rules! generate_bench {
    ($name:ident,$se:expr,$de:expr) => {
        mod $name {
            use super::*;

            #[divan::bench]
            fn serialize(b: Bencher) {
                let people = fake_people(None);

                b.bench(|| {
                    let result = $se(&people).expect("Failed to serialize");
                    black_box(result);
                });
            }

            #[divan::bench]
            fn deserialize(b: Bencher) {
                let people = fake_people(None);
                let people_vec = $se(&people).expect("Failed to serialize");

                b.bench(|| {
                    let result: Vec<Person> = $de(&people_vec).expect("Failed to deserialize");
                    black_box(result);
                });
            }
        }
    };
}

generate_bench!(postcard, ::postcard::to_stdvec, ::postcard::from_bytes);
generate_bench!(rmp_serde, ::rmp_serde::to_vec, ::rmp_serde::from_slice);
generate_bench!(minicbor, minicbor_serde::to_vec, minicbor_serde::from_slice);
generate_bench!(bitcode, ::bitcode::serialize, ::bitcode::deserialize);
generate_bench!(speedy_vec, |v: &Vec<Person>| v.write_to_vec(), |v: &Vec<
    u8,
>| {
    Vec::read_from_buffer(v)
});
