use std::collections::HashSet;

use argon2::Params;
use minidb_utils::{generate_salt, Argon2Params};

#[test]
fn test_argon2params() {
    let my_params = Argon2Params {
        iterations: 1,
        memory: 1024,
        parallelism: 1,
        output_len: Some(32),
    };

    let params: Params = my_params
        .clone()
        .try_into()
        .expect("Failed to convert Argon2Params to argon2::Params");

    assert_eq!(params.m_cost(), my_params.memory);
    assert_eq!(params.t_cost(), my_params.iterations);
    assert_eq!(params.p_cost(), my_params.parallelism);
    assert_eq!(params.output_len(), my_params.output_len);
}

#[test]
fn test_generate_salt() {
    const N: usize = 100000;

    let mut salts = HashSet::new();
    for _ in 0..N {
        let salt = generate_salt().expect("Failed to generate salt");
        assert_eq!(dbg!(salt).len(), 16);

        if !salts.contains(&salt) {
            salts.insert(salt);
        }
    }

    assert_eq!(dbg!(salts).len(), N);
}
