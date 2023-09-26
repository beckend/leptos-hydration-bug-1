use std::env;
use std::fmt::{Debug, Display};
use std::io;
use std::path::{Path, PathBuf};

use anyhow::Context;
use anyhow::Result;
use path_clean::PathClean;

pub fn absolute_path_unresolved(path: impl AsRef<Path>) -> io::Result<PathBuf> {
  let path = path.as_ref();

  let absolute_path = if path.is_absolute() {
    path.to_path_buf()
  } else {
    env::current_dir()?.join(path)
  }
  .clean();

  Ok(absolute_path)
}

pub trait PathToAbsoluteUnresolvedExt<'a>: Send + Sync {
  fn absolute_path_unresolved(&'a self) -> Result<PathBuf>
  where
    Self: Send + Sync + Display + Debug;
}

impl<'a, TInput: AsRef<Path> + Send + Sync + ?Sized> PathToAbsoluteUnresolvedExt<'a> for TInput {
  fn absolute_path_unresolved(&'a self) -> Result<PathBuf>
  where
    Self: Send + Sync + Display + Debug,
  {
    absolute_path_unresolved(self.as_ref()).with_context(|| {
      format!(
        "failed to get absolute path: {}",
        self.as_ref().to_string_lossy()
      )
    })
  }
}
