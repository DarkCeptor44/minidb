use anyhow::{anyhow, Error};
use argon2::{
    password_hash::{Salt as Argon2Salt, SaltString},
    Argon2, Params, ParamsBuilder, PasswordHash, PasswordHasher, PasswordVerifier,
};
use rand::TryRngCore;
use serde::{Deserialize, Serialize};

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

impl TryInto<Params> for Argon2Params {
    type Error = Error;

    fn try_into(self) -> std::result::Result<Params, Self::Error> {
        ParamsBuilder::new()
            .m_cost(self.memory)
            .t_cost(self.iterations)
            .p_cost(self.parallelism)
            .output_len(self.output_len.unwrap_or(32))
            .build()
            .map_err(|e| anyhow!("Failed to build Argon2 parameters: {e}"))
    }
}
