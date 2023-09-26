use axum::{http::StatusCode, response::IntoResponse, BoxError};
use cowstr::ToCowStr;

use super::codes::CodesResponse;
use crate::ssr::features::api::{ResponseGeneric, ResponseStatus};

pub struct Handler {}

impl Handler {
  pub async fn handler_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
      return (
        StatusCode::REQUEST_TIMEOUT,
        ResponseGeneric::new(
          ResponseStatus::Failure,
          Some(super::codes::CodesResponse::Timeout),
          Some("request timed out".to_cowstr()),
        ),
      );
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
      return (
        StatusCode::SERVICE_UNAVAILABLE,
        ResponseGeneric::new(
          ResponseStatus::Failure,
          Some(CodesResponse::Overloaded),
          Some("service is overloaded, try again later".to_cowstr()),
        ),
      );
    }

    (
      StatusCode::INTERNAL_SERVER_ERROR,
      ResponseGeneric::new(
        ResponseStatus::Failure,
        Some(CodesResponse::GenericOrUnknown),
        Some(format!("Unhandled internal error: {}", error)),
      ),
    )
  }
}
