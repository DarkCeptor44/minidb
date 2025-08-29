use anyhow::{anyhow, Context, Error, Result};
use argon2::{
    password_hash::{Salt as Argon2Salt, SaltString},
    Argon2, Params, ParamsBuilder, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand::TryRngCore;
use serde::{Deserialize, Serialize};

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

/// [Argon2] parameters wrapper
///
/// ## Example
///
/// ```rust
/// use minidb_utils::Argon2Params;
/// use argon2::Params;
///
/// let my_params = Argon2Params {
///     iterations: 1,
///     memory: 1024,
///     parallelism: 1,
///     output_len: Some(32),
/// };
///
/// let params: Params = my_params.try_into().unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Params {
    /// Memory cost in KiB, between 8 * [`threads`](Self::parallelism) and `(2^32)-1`
    pub memory: u32,

    /// Number of iterations, also known as `t_cost`, between 1 and `(2^32)-1`
    pub iterations: u32,

    /// Number of threads, also known as `p_cost` or `threads`, between 1 and `(2^24)-1`
    pub parallelism: u32,

    /// Length of the output (default: 32 bytes)
    pub output_len: Option<usize>,
}

impl TryFrom<Argon2Params> for Params {
    type Error = argon2::Error;

    fn try_from(value: Argon2Params) -> std::result::Result<Self, Self::Error> {
        Self::new(
            value.memory,
            value.iterations,
            value.parallelism,
            value.output_len,
        )
    }
}

/// Derive a key from a password and a salt using [Argon2id](argon2)
///
/// ## Arguments
///
/// * `ctx` - The [Argon2] context to use, it contains the parameters used to derive the key like the number of iterations, memory, threads and output length. If not provided, the default parameters will be used
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
///
/// ## Example
///
/// ```rust
/// use minidb_utils::derive_key;
///
/// let key = derive_key(None, "password", "somesalt").unwrap();
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
