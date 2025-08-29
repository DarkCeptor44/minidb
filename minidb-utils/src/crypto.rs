use anyhow::{anyhow, Context, Error, Result};
use argon2::{
    password_hash::{Salt as Argon2Salt, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand::TryRngCore;

/// Extension trait for [`Option<T>`]
pub trait IntoOptional<T> {
    /// Convert T to [`Option<T>`]
    fn into_optional(self) -> Option<T>;
}

impl<T> IntoOptional<T> for T {
    fn into_optional(self) -> Option<T> {
        Some(self)
    }
}

impl<T> IntoOptional<T> for Option<T> {
    fn into_optional(self) -> Option<T> {
        self
    }
}

/// Derive a key from a password and a salt using [Argon2id](argon2)
///
/// ## Arguments
///
/// * `ctx` - The [Argon2] struct to use, it contains the parameters used to derive the key like the number of iterations, memory, threads and output length. If not provided, the default parameters will be used
/// * `pass` - The password to derive the key from
/// * `salt` - The salt to derive the key with
///
/// ## Returns
///
/// The derived key of N bytes (32 by default)
///
/// ## Errors
///
/// Returns an error if the key derivation fails, refer to [`Argon2::hash_password_into`] for why it might fail
///
/// ## Examples
///
/// Derive a key with the default parameters
///
/// ```rust
/// use minidb_utils::derive_key;
///
/// let key = derive_key(None, "password", "somesalt").unwrap();
/// println!("Key: {:?}", key);
/// ```
///
/// Derive a key with custom parameters
///
/// ```rust
/// use minidb_utils::derive_key;
/// use argon2::{Algorithm, Argon2, Params, Version};
///
/// let ctx = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(1024, 1, 1, Some(24)).unwrap());
/// let key = derive_key(ctx, "password", "somesalt").unwrap();
/// println!("Key: {:?}", key);
/// ```
pub fn derive_key<'a, Ctx, Pass, Salt>(ctx: Ctx, pass: Pass, salt: Salt) -> Result<Vec<u8>>
where
    Ctx: IntoOptional<Argon2<'a>>,
    Pass: AsRef<[u8]>,
    Salt: AsRef<[u8]>,
{
    derive_key_impl(ctx.into_optional(), pass.as_ref(), salt.as_ref())
}

fn derive_key_impl(ctx: Option<Argon2>, pass: &[u8], salt: &[u8]) -> Result<Vec<u8>> {
    let ctx = ctx.unwrap_or_default();
    let l = ctx.params().output_len().unwrap_or(32);
    let mut key = vec![0u8; l];

    ctx.hash_password_into(pass, salt, &mut key)
        .map_err(|e| anyhow!(e))
        .context("Failed to derive key")?;

    Ok(key)
}

/// Generate a random salt
///
/// ## Returns
///
/// A 16 byte salt
///
/// ## Errors
///
/// Returns an error if failed to generate salt, refer to [`rand::rngs::OsRng::try_fill_bytes`] for why it might fail
///
/// ## Example
///
/// ```rust
/// use minidb_utils::generate_salt;
///
/// let salt = generate_salt().unwrap();
/// ```
pub fn generate_salt() -> Result<[u8; 16]> {
    let mut rng = rand::rngs::OsRng;
    let mut salt = [0u8; 16];

    rng.try_fill_bytes(&mut salt)
        .context("Failed to generate salt")?;
    Ok(salt)
}
