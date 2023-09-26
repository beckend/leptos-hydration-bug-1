use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;

use crate::ssr::modules::fs::ext_path_parent::PathParentExt;

#[async_trait]
pub trait PathCreateWithDirectoryExt<'a>: Send + Sync {
  async fn path_create_with_directory(&'a self, truncate: bool, create: bool) -> Result<fs::File>
  where
    Self: Send + Sync;
}

#[async_trait]
impl<'a, TInput: AsRef<Path> + Send + Sync + ?Sized> PathCreateWithDirectoryExt<'a> for TInput {
  async fn path_create_with_directory(&'a self, truncate: bool, create: bool) -> Result<fs::File>
  where
    Self: Send + Sync,
  {
    if create {
      fs::create_dir_all(self.path_parent()).await?;
    }

    match fs::OpenOptions::new()
      .write(true)
      .truncate(truncate)
      .create(create)
      .open(&self)
      .await
    {
      Ok(x) => Ok(x),
      Err(err) => Err(anyhow::anyhow!("{}", err)),
    }
  }
}
