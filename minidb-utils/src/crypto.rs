use crate::{IntoOptional, UtilsError};
use anyhow::{Context, Result, anyhow};
use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{Salt as Argon2Salt, SaltString},
};
use rand::TryRngCore;
use serde::{Deserialize, Serialize};

/// Argon2 parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArgonParams {
    /// Memory size, expressed in kibibytes, between 8*`p_cost` and (2^32)-1.
    ///
    /// Value is an integer in decimal (1 to 10 digits).
    pub memory: u32,

    /// Number of iterations, between 1 and (2^32)-1.
    ///
    /// Value is an integer in decimal (1 to 10 digits).
    pub iterations: u32,

    /// Degree of parallelism, between 1 and (2^24)-1.
    ///
    /// Value is an integer in decimal (1 to 8 digits).
    pub parallelism: u32,

    /// Output length, in bytes.
    pub output_len: usize,
}

impl Default for ArgonParams {
    fn default() -> Self {
        Self {
            memory: Params::DEFAULT_M_COST,
            iterations: Params::DEFAULT_T_COST,
            parallelism: Params::DEFAULT_P_COST,
            output_len: Params::DEFAULT_OUTPUT_LEN,
        }
    }
}

impl TryFrom<ArgonParams> for Params {
    type Error = anyhow::Error;

    fn try_from(value: ArgonParams) -> std::result::Result<Self, Self::Error> {
        Params::new(
            value.memory,
            value.iterations,
            value.parallelism,
            Some(value.output_len),
        )
        .map_err(|e| anyhow!(e))
        .context(UtilsError::FailedToCreateParams)
    }
}

impl ArgonParams {
    /// Creates a new Argon2 parameters struct
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the memory size
    #[must_use]
    pub fn m_cost(mut self, memory: u32) -> Self {
        self.memory = memory;
        self
    }

    /// Sets the number of iterations
    #[must_use]
    pub fn t_cost(mut self, iterations: u32) -> Self {
        self.iterations = iterations;
        self
    }

    /// Sets the degree of parallelism
    #[must_use]
    pub fn p_cost(mut self, parallelism: u32) -> Self {
        self.parallelism = parallelism;
        self
    }

    /// Sets the output length
    #[must_use]
    pub fn output_len(mut self, output_len: usize) -> Self {
        self.output_len = output_len;
        self
    }
}

/// Derive a key from a password and a salt using [Argon2id](argon2)
///
/// ## Arguments
///
/// * `params` - The parameters used to derive the key like the number of iterations, memory, threads and output length. If not provided, the default parameters will be used
/// * `pass` - The password to derive the key from
/// * `salt` - The salt to derive the key with
///
/// ## Returns
///
/// The derived key of N bytes (32 by default)
///
/// ## Errors
///
/// * [`UtilsError::FailedToCreateParams`]: Could not create Argon2 parameters, refer to [`Params::new`](argon2::Params::new) for why it might fail
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
/// use minidb_utils::{ArgonParams, derive_key};
///
/// let p = ArgonParams::new().m_cost(1024).t_cost(2).p_cost(1).output_len(24);
/// let key = derive_key(p, "password", "somesalt").unwrap();
/// println!("Key: {:?}", key);
/// ```
pub fn derive_key<Params, Pass, Salt>(params: Params, pass: Pass, salt: Salt) -> Result<Vec<u8>>
where
    Params: IntoOptional<ArgonParams>,
    Pass: AsRef<[u8]>,
    Salt: AsRef<[u8]>,
{
    derive_key_impl(params.into_optional(), pass.as_ref(), salt.as_ref())
}

fn derive_key_impl(params: Option<ArgonParams>, pass: &[u8], salt: &[u8]) -> Result<Vec<u8>> {
    let params = params.unwrap_or_default();
    let l = params.output_len;
    let ctx = Argon2::new(Algorithm::Argon2id, Version::V0x13, params.try_into()?);
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
/// * `params` - The parameters used to hash the password like the number of iterations, memory, threads and output length. If not provided, the default parameters will be used
/// * `pass` - The password to hash
/// * `salt` - The salt to hash with
///
/// ## Returns
///
/// A PHC string representing the hashed password
///
/// ## Errors
///
/// * [`UtilsError::FailedToCreateParams`]: Could not create Argon2 parameters, refer to [`Params::new`](argon2::Params::new) for why it might fail
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
/// use minidb_utils::{ArgonParams, hash_password};
///
/// let p = ArgonParams::new().m_cost(1024).t_cost(2).p_cost(1).output_len(24);
/// let hash = hash_password(p, "password", "somesalt").unwrap();
/// println!("Hash: {}", hash); // $argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$PL01amPyeUuxG7H0vIr5X+qHkZvWnHmGBGXFYvh8z2E
/// ```
pub fn hash_password<Params, Pass, Salt>(params: Params, pass: Pass, salt: Salt) -> Result<String>
where
    Params: IntoOptional<ArgonParams>,
    Pass: AsRef<[u8]>,
    Salt: AsRef<[u8]>,
{
    hash_password_impl(params.into_optional(), pass.as_ref(), salt.as_ref())
}

fn hash_password_impl(params: Option<ArgonParams>, pass: &[u8], salt: &[u8]) -> Result<String> {
    let params = params.unwrap_or_default();
    let ctx = Argon2::new(Algorithm::Argon2id, Version::V0x13, params.try_into()?);
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
