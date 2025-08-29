use std::collections::HashSet;

use argon2::{Algorithm, Argon2, Params, PasswordHash, Version};
use minidb_utils::{derive_key, generate_salt, hash_password};

#[test]
fn test_derive_key_simple() {
    let key = derive_key(None, "password", "somesalt").expect("Failed to derive key");
    assert_eq!(dbg!(key).len(), 32);
}

#[test]
fn test_derive_key_complex() {
    let ctx = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(1024, 1, 1, Some(24)).expect("Failed to create argon2::Params"),
    );
    let key = derive_key(ctx, "password", "somesalt").expect("Failed to derive key");
    assert_eq!(dbg!(key).len(), 24);
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
