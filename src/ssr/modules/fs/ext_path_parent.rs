use std::path::{Path, PathBuf};

pub trait PathParentExt<'a>: Send + Sync {
  fn path_parent(&'a self) -> PathBuf
  where
    Self: Send + Sync;
}

impl<'a, TInput: AsRef<Path> + Send + Sync + ?Sized> PathParentExt<'a> for TInput {
  fn path_parent(&'a self) -> PathBuf
  where
    Self: Send + Sync,
  {
    let mut path_owned = self.as_ref().to_owned();
    path_owned.pop();

    path_owned
  }
}
