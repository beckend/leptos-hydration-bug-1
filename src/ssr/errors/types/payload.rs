use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub struct ErrorPayload {
  message: String,
}

impl ErrorPayload {
  pub fn new(message: impl AsRef<str>) -> Self {
    Self {
      message: message.as_ref().to_string(),
    }
  }
}

impl fmt::Display for ErrorPayload {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Invalid payload: {}", self.message)
  }
}
