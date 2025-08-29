use std::collections::HashSet;

use argon2::{Algorithm, Argon2, Params, PasswordHash, Version};
use minidb_utils::{derive_key, generate_salt, hash_password, verify_password};

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

#[test]
fn test_hash_password_default() {
    let hash = hash_password(None, "password", "somesalt").expect("Failed to hash password");
    PasswordHash::new(dbg!(&hash)).expect("Failed to parse password hash");
}

#[test]
fn test_hash_password_19m_2t_1p_32() {
    let ctx = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19 * 1024, 2, 1, Some(32)).expect("Failed to create argon2::Params"),
    );
    let hash = hash_password(ctx, "password", "somesalt").expect("Failed to hash password");
    PasswordHash::new(dbg!(&hash)).expect("Failed to parse password hash");
}

#[test]
fn test_verify_password_default() {
    let password = "password".to_string();
    let hash = hash_password(None, &password, "somesalt").expect("Failed to hash password");
    assert!(!verify_password("wrongpass", &hash).expect("Failed to verify password"));
    assert!(verify_password(password, hash).expect("Failed to verify password"));
}

#[test]
fn test_verify_password_19m_2t_1p_32() {
    let password = "password".to_string();
    let ctx = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19 * 1024, 2, 1, Some(32)).expect("Failed to create argon2::Params"),
    );
    let hash = hash_password(ctx, &password, "somesalt").expect("Failed to hash password");
    assert!(!verify_password("wrongpass", &hash).expect("Failed to verify password"));
    assert!(verify_password(password, hash).expect("Failed to verify password"));
}
