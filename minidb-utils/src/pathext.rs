use std::path::Path;

use anyhow::{Context, Result};
use minidb_shared::MiniDBError;

/// Extension trait for [`Path`], [`PathBuf`](std::path::PathBuf) or anything that implements [`AsRef<Path>`].
pub trait PathExt {
    /// Returns `true` if the path is a directory and is empty
    ///
    /// ## Errors
    ///
    /// Returns an error if the directory could not be read
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use minidb_utils::PathExt;
    ///
    /// let path = Path::new("path/to/dir");
    ///
    /// if path.is_empty().unwrap() {
    ///     println!("Directory is safe to delete or etc");
    /// }
    /// ```
    fn is_empty(&self) -> Result<bool>;
}

impl<P> PathExt for P
where
    P: AsRef<Path>,
{
    fn is_empty(&self) -> Result<bool> {
        let path = self.as_ref();

        if !path.is_dir() {
            return Ok(false);
        }

        let mut dir = path
            .read_dir()
            .context(MiniDBError::FailedToReadDir(path.to_path_buf()))?;
        Ok(dir.next().is_none())
    }
}
