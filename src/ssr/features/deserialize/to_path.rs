use anyhow::Result;
use serde::{de, Deserialize, Deserializer};
use std::path::PathBuf;

use crate::ssr::modules::fs::absolute::PathToAbsoluteUnresolvedExt;

pub fn from_string_to_path_unresolved<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
  D: Deserializer<'de>,
{
  let x: String = Deserialize::deserialize(deserializer)?;
  x.absolute_path_unresolved().map_err(de::Error::custom)
}
