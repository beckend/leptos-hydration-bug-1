use serde::{Deserialize, Serialize};
use strum::AsRefStr;

#[derive(AsRefStr, Debug, Clone, Serialize, Deserialize, utoipa::ToSchema, PartialEq, Eq)]
pub enum CodesResponse {
  /**
   * Errors
   */
  #[strum(serialize = "10000")]
  GenericOrUnknown,
  #[strum(serialize = "10001")]
  Overloaded,
  #[strum(serialize = "10003")]
  Timeout,

  #[strum(serialize = "11000")]
  TemplateAskama,
  #[strum(serialize = "11001")]
  DatabaseOperations,
  #[strum(serialize = "11002")]
  ValidationError,

  /**
   * Success
   */
  #[strum(serialize = "200")]
  OK,
}
