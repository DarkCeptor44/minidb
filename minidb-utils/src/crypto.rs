use crate::{IntoOptional, UtilsError};
use anyhow::{anyhow, Context, Result};
use argon2::{
    password_hash::{Salt as Argon2Salt, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand::TryRngCore;

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
/// * [`UtilsError::FailedToDeriveKey`]: Key derivation failed, refer to [`Argon2::hash_password_into`] for why it might fail
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
/// use argon2::{Algorithm, Argon2, Params, Version};
/// use minidb_utils::derive_key;
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
        .context(UtilsError::FailedToDeriveKey)?;

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
/// * [`UtilsError::FailedToGenerateSalt`]: salt generation failed, refer to [`rand::rngs::OsRng::try_fill_bytes`] for why it might fail
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
        .context(UtilsError::FailedToGenerateSalt)?;
    Ok(salt)
}

/// Hash a password using [Argon2id](argon2)
///
/// ## Arguments
///
/// * `ctx` - The [Argon2] struct to use, it contains the parameters used to hash the password like the number of iterations, memory, threads and output length. If not provided, the default parameters will be used
/// * `pass` - The password to hash
/// * `salt` - The salt to hash with
///
/// ## Returns
///
/// A PHC string representing the hashed password
///
/// ## Errors
///
/// * [`UtilsError::FailedToEncodeSalt`]: could not encode salt to Base64
/// * [`UtilsError::FailedToHashPassword`]: password hashing failed, refer to [`PasswordHasher::hash_password`] for why it might fail
///
/// ## Examples
///
/// Hash a password with the default parameters
///
/// ```rust
/// use argon2::PasswordHash;
/// use minidb_utils::hash_password;
///
/// let hash = hash_password(None, "password", "somesalt").unwrap();
/// println!("Hash: {}", hash); // $argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$PL01amPyeUuxG7H0vIr5X+qHkZvWnHmGBGXFYvh8z2E
/// ```
///
/// Hash a password with custom parameters
///
/// ```rust
/// use argon2::{Algorithm, Argon2, Params, Version};
/// use minidb_utils::hash_password;
///
/// let ctx = Argon2::new(
///         Algorithm::Argon2id,
///         Version::V0x13,
///         Params::new(19 * 1024, 2, 1, Some(32)).unwrap(),
///     );
/// let hash = hash_password(ctx, "password", "somesalt").unwrap();
/// println!("Hash: {}", hash); // $argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$PL01amPyeUuxG7H0vIr5X+qHkZvWnHmGBGXFYvh8z2E
/// ```
pub fn hash_password<'a, Ctx, Pass, Salt>(ctx: Ctx, pass: Pass, salt: Salt) -> Result<String>
where
    Ctx: IntoOptional<Argon2<'a>>,
    Pass: AsRef<[u8]>,
    Salt: AsRef<[u8]>,
{
    hash_password_impl(ctx.into_optional(), pass.as_ref(), salt.as_ref())
}

fn hash_password_impl(ctx: Option<Argon2>, pass: &[u8], salt: &[u8]) -> Result<String> {
    let ctx = ctx.unwrap_or_default();
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| anyhow!(e))
        .context(UtilsError::FailedToEncodeSalt(salt.to_vec()))?;
    let salt: Argon2Salt = (&salt_string).into();
    let hash = ctx
        .hash_password(pass, salt)
        .map_err(|e| anyhow!(e))
        .context(UtilsError::FailedToHashPassword)?;
    Ok(hash.to_string())
}

/// Verify a password using [Argon2id](argon2)
///
/// ## Arguments
///
/// * `pass` - The password to verify
/// * `hash` - The PHC string representing the hashed password
///
/// ## Returns
///
/// `true` if the password is correct, `false` otherwise
///
/// ## Errors
///
/// * [`UtilsError::FailedToParsePHCString`]: could not parse the PHC string, refer to [`PasswordVerifier::verify_password`] for why it might fail
///
/// ## Example
///
/// ```rust
/// use minidb_utils::{hash_password, verify_password};
///
/// let password = "password";
/// let hash = hash_password(None, password, "somesalt").unwrap();
///
/// if verify_password(password, hash).unwrap() {
///     println!("Password is correct");
/// } else {
///     println!("Password is incorrect");
/// }
/// ```
pub fn verify_password<Pass, Hash>(pass: Pass, hash: Hash) -> Result<bool>
where
    Pass: AsRef<[u8]>,
    Hash: AsRef<str>,
{
    verify_password_impl(pass.as_ref(), hash.as_ref())
}

fn verify_password_impl(pass: &[u8], hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow!(e))
        .context(UtilsError::FailedToParsePHCString(hash.to_string()))?;
    let ctx = Argon2::default();
    Ok(ctx.verify_password(pass, &parsed_hash).is_ok())
}
