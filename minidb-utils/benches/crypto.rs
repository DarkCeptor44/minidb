use argon2::{Algorithm, Argon2, ParamsBuilder, Version};
use divan::{black_box, Bencher};
use minidb_utils as utils;

fn main() {
    divan::main();
}

#[divan::bench(args = [(1024, 2, 1), (19*1024, 3, 1), (64*1024, 3, 2)])]
fn derive_key(b: Bencher, p: (u32, u32, u32)) {
    b.with_inputs(|| {
        Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            ParamsBuilder::new()
                .m_cost(p.0)
                .t_cost(p.1)
                .p_cost(p.2)
                .build()
                .expect("Failed to create argon2::Params"),
        )
    })
    .bench_values(|ar| {
        let key = utils::derive_key(black_box(ar), black_box("password"), black_box("somesalt"))
            .expect("Failed to derive key");
        black_box(key);
    });
}

#[divan::bench]
fn generate_salt() {
    black_box(utils::generate_salt().expect("Failed to generate salt"));
}

#[divan::bench(args = [(1024, 2, 1), (19*1024, 3, 1), (64*1024, 3, 2)])]
fn hash_password(b: Bencher, p: (u32, u32, u32)) {
    b.with_inputs(|| {
        Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            ParamsBuilder::new()
                .m_cost(p.0)
                .t_cost(p.1)
                .p_cost(p.2)
                .build()
                .expect("Failed to create argon2::Params"),
        )
    })
    .bench_values(|ar| {
        let hash =
            utils::hash_password(black_box(ar), black_box("password"), black_box("somesalt"))
                .expect("Failed to hash password");
        black_box(hash);
    });
}

#[divan::bench(args = [(1024, 2, 1), (19*1024, 3, 1), (64*1024, 3, 2)])]
fn verify_password(b: Bencher, p: (u32, u32, u32)) {
    b.with_inputs(|| {
        let pass = "password";

        (
            pass,
            utils::hash_password(
                Argon2::new(
                    Algorithm::Argon2id,
                    Version::V0x13,
                    ParamsBuilder::new()
                        .m_cost(p.0)
                        .t_cost(p.1)
                        .p_cost(p.2)
                        .build()
                        .expect("Failed to build params"),
                ),
                pass,
                "somesalt",
            )
            .expect("Failed to hash password"),
        )
    })
    .bench_values(|(pass, hash)| {
        black_box(
            utils::verify_password(black_box(pass), black_box(hash))
                .expect("Failed to verify password with hash"),
        );
    });
}
