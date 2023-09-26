use super::ext_path_create_with_directory::PathCreateWithDirectoryExt;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use tokio::{
  fs::File,
  io::{AsyncWriteExt, BufWriter},
};

#[async_trait]
pub trait ContentsWriteToPathExt<'a>: Send + Sync {
  async fn contents_write_to_path<PathFile>(
    &'a self,
    path_file: &'a PathFile,
    truncate: bool,
    create: bool,
  ) -> Result<BufWriter<File>>
  where
    Self: Send + Sync,
    PathFile: 'a + AsRef<Path> + Send + Sync + ?Sized;
}

#[async_trait]
impl<'a, TInput: AsRef<str> + Send + Sync> ContentsWriteToPathExt<'a> for TInput {
  async fn contents_write_to_path<PathFile>(
    &'a self,
    path_file: &'a PathFile,
    truncate: bool,
    create: bool,
  ) -> Result<BufWriter<File>>
  where
    Self: Send + Sync,
    PathFile: 'a + AsRef<Path> + Send + Sync + ?Sized,
  {
    let mut file = BufWriter::new(
      path_file
        .path_create_with_directory(truncate, create)
        .await?,
    );
    file.write_all(self.as_ref().as_bytes()).await?;
    file.flush().await?;
    Ok(file)
  }
}

#[async_trait]
impl<'a> ContentsWriteToPathExt<'a> for [u8] {
  async fn contents_write_to_path<PathFile>(
    &'a self,
    path_file: &'a PathFile,
    truncate: bool,
    create: bool,
  ) -> Result<BufWriter<File>>
  where
    Self: Send + Sync,
    PathFile: 'a + AsRef<Path> + Send + Sync + ?Sized,
  {
    let mut file = BufWriter::new(
      path_file
        .path_create_with_directory(truncate, create)
        .await?,
    );
    file.write_all(self).await?;
    file.flush().await?;
    Ok(file)
  }
}
