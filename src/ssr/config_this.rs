use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use cowstr::CowStr;
use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use strum::AsRefStr;

use crate::ssr::features::deserialize::to_path;

#[derive(AsRefStr, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CorsSetting {
  Default,
  VeryPermissive,
}

impl Default for CorsSetting {
  fn default() -> Self {
    Self::Default
  }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Server {
  #[serde(deserialize_with = "to_path::from_string_to_path_unresolved")]
  pub path_dir_public: PathBuf,
  #[serde(deserialize_with = "to_path::from_string_to_path_unresolved")]
  pub path_dir_data_session: PathBuf,
  #[serde(default)]
  pub cors: CorsSetting,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DBGeneric {
  pub uri: CowStr,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Databases {
  pub postgres: DBGeneric,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Tls {
  #[serde(deserialize_with = "to_path::from_string_to_path_unresolved")]
  pub cert: PathBuf,
  #[serde(deserialize_with = "to_path::from_string_to_path_unresolved")]
  pub key: PathBuf,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct ConfigThis {
  pub databases: Databases,
  pub server: Server,
  pub run_env: CowStr,
  pub tls: Option<Tls>,
}

impl ConfigThis {
  pub fn new() -> Result<Self, ConfigError> {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

    let s = Config::builder()
      .add_source(File::with_name("./configs/base.toml").required(false))
      .add_source(File::with_name(&format!("./configs/{}.toml", run_mode)).required(false))
      // This file shouldn't be checked in to version control systems, this is gitignored and overrides all files
      .add_source(File::with_name("./configs/local.toml").required(false))
      // Add in settings from the environment (with a prefix of APP)
      // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
      .add_source(Environment::with_prefix("THIS").separator("_"))
      .build()?;

    // You can deserialize (and thus freeze) the entire configuration as
    s.try_deserialize()
  }
}
