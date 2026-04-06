// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed
// with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{ArgonKey, Error, error::Result};
use argon2::{
    Algorithm, Argon2, Params as ArgonParams, PasswordHasher, Version,
    password_hash::{SaltString, rand_core::RngCore},
};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, OsRng},
};

/// Decrypt bytes using a cipher and ciphertext
///
/// ## Arguments
///
/// * `cipher` - The cipher to use for decryption
/// * `ciphertext` - The ciphertext to decrypt
///
/// ## Returns
///
/// A `Result` containing the decrypted bytes
///
/// ## Errors
///
/// Returns an error if the decryption fails or if the ciphertext is too short
pub fn decrypt_bytes<C>(cipher: &XChaCha20Poly1305, ciphertext: C) -> Result<Vec<u8>>
where
    C: AsRef<[u8]>,
{
    decrypt_bytes_impl(cipher, ciphertext.as_ref())
}

fn decrypt_bytes_impl(cipher: &XChaCha20Poly1305, ciphertext: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < 24 {
        return Err(Error::CipherTextTooShort(ciphertext.len()));
    }

    let (nonce_bytes, ciphertext) = ciphertext.split_at(24);
    let nonce_array: [u8; 24] = nonce_bytes.try_into().unwrap();
    let nonce = XNonce::from(nonce_array);
    let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref())?;

    Ok(plaintext)
}

/// Derive a key from a password and salt
///
/// ## Arguments
///
/// * `password` - The password to derive the key from
/// * `salt` - The salt to use for the key derivation
/// * `params` - The parameters to use for the key derivation
///
/// ## Returns
///
/// A `Result` containing the derived key
///
/// ## Errors
///
/// Returns an error if the key derivation fails
pub fn derive_key_from_password<Pass, Salt, Params>(
    password: Pass,
    salt: Salt,
    params: Params,
) -> Result<ArgonKey>
where
    Pass: AsRef<str>,
    Salt: Into<Option<String>>,
    Params: Into<Option<ArgonParams>>,
{
    derive_key_from_password_impl(password.as_ref(), salt.into(), params.into())
}

fn derive_key_from_password_impl(
    password: &str,
    salt: Option<String>,
    params: Option<ArgonParams>,
) -> Result<ArgonKey> {
    let salt_string = salt.unwrap_or_else(|| {
        let salt_str = SaltString::generate(&mut OsRng);
        salt_str.to_string()
    });
    let salt = SaltString::from_b64(&salt_string)?;

    let params = params.unwrap_or_default();
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let bytes = argon2
        .hash_password(password.as_bytes(), &salt)?
        .hash
        .ok_or(Error::MissingHashOutput)?;

    if bytes.len() != 32 {
        return Err(Error::KeyLengthMismatch(bytes.len()));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(bytes.as_bytes());
    Ok(key)
}

/// Encrypt bytes using a cipher
///
/// ## Arguments
///
/// * `cipher` - The cipher to use for encryption
/// * `plaintext` - The plaintext to encrypt
///
/// ## Returns
///
/// A `Result` containing the encrypted bytes
///
/// ## Errors
///
/// Returns an error if the encryption fails
pub fn encrypt_bytes<P>(cipher: &XChaCha20Poly1305, plaintext: P) -> Result<Vec<u8>>
where
    P: AsRef<[u8]>,
{
    encrypt_bytes_impl(cipher, plaintext.as_ref())
}

fn encrypt_bytes_impl(cipher: &XChaCha20Poly1305, plaintext: &[u8]) -> Result<Vec<u8>> {
    let mut rng = OsRng;
    let mut nonce_bytes = [0u8; 24];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from(nonce_bytes);

    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref())?;
    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::time_function;
    use chacha20poly1305::KeyInit;

    #[test]
    fn test_derive_key_from_password() {
        let password = "abcdef123";
        let salt = SaltString::generate(&mut OsRng);
        let key1 =
            time_function!(derive_key_from_password(password, salt.to_string(), None)).unwrap();
        let key2 =
            time_function!(derive_key_from_password(password, salt.to_string(), None)).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_key_differs_for_different_passwords() {
        let password1 = "asdjaskdkas";
        let password2 = "dkdkklsakll";
        let salt = SaltString::generate(&mut OsRng);

        let key1 =
            time_function!(derive_key_from_password(password1, salt.to_string(), None)).unwrap();
        let key2 =
            time_function!(derive_key_from_password(password2, salt.to_string(), None)).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_and_consistency() {
        const N: u64 = 10;

        let password = "abcdef123";
        let salt = time_function!(SaltString::generate(&mut OsRng));
        let mut keys_map: HashMap<[u8; 32], u64> = HashMap::new();

        for _ in 0..N {
            let key =
                time_function!(derive_key_from_password(password, salt.to_string(), None)).unwrap();
            *keys_map.entry(key).or_insert(0) += 1;
        }

        assert_eq!(keys_map.len(), 1);
        let (_, count) = keys_map.iter().next().unwrap();
        assert_eq!(*count, N);
    }

    #[test]
    fn test_encryption_and_decryption() {
        let key = [1u8; 32];
        let cipher = XChaCha20Poly1305::new(&key.into());
        let plaintext = b"hello world";
        let ciphertext = time_function!(encrypt_bytes(&cipher, plaintext)).unwrap();
        let decrypted = time_function!(decrypt_bytes(&cipher, &ciphertext)).unwrap();

        assert_eq!(&plaintext[..], &decrypted);
    }
}
