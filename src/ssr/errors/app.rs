use axum::{
  extract::rejection::FormRejection,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use cowstr::{CowStr, ToCowStr};
use sea_orm::DbErr;
use serde::Serialize;
use tokio::task::JoinError;

use crate::ssr::features::api::{ResponseGeneric, ResponseStatus};

use super::{codes::CodesResponse, types::payload::ErrorPayload};

#[derive(thiserror::Error, Debug, utoipa::ToSchema)]
pub enum ErrorsAll {
  #[error(transparent)]
  AxumFormRejection(#[from] FormRejection),
  #[error(transparent)]
  ChannelsKanal(#[from] kanal::SendError),
  #[error(transparent)]
  DatabaseOperations(#[from] DbErr),
  #[error(transparent)]
  Generic(#[from] anyhow::Error),
  #[error(transparent)]
  Payload(#[from] ErrorPayload),
  #[error(transparent)]
  ValidationError(#[from] validator::ValidationErrors),
  #[error(transparent)]
  Threading(#[from] JoinError),
}

#[derive(Serialize, utoipa::ToSchema)]
struct ResponseError {
  #[schema(value_type = String)]
  pub error_message: CowStr,
}

impl IntoResponse for ErrorsAll {
  fn into_response(self) -> Response {
    let (status, payload) = match self {
      Self::AxumFormRejection(_) => {
        let message = format!("Input validation error: [{}]", self).replace('\n', ", ");
        (
          StatusCode::BAD_REQUEST,
          ResponseGeneric::new(
            ResponseStatus::Failure,
            Some(CodesResponse::TemplateAskama),
            Some(message),
          ),
        )
      }
      Self::ChannelsKanal(err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ResponseGeneric::new(
          ResponseStatus::Failure,
          Some(CodesResponse::TemplateAskama),
          Some(err.to_cowstr()),
        ),
      ),
      Self::DatabaseOperations(err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ResponseGeneric::new(
          ResponseStatus::Failure,
          Some(CodesResponse::DatabaseOperations),
          Some(err.to_cowstr()),
        ),
      ),
      Self::Generic(err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ResponseGeneric::new(
          ResponseStatus::Failure,
          Some(CodesResponse::GenericOrUnknown),
          Some(err.to_cowstr()),
        ),
      ),
      Self::Payload(err) => (
        StatusCode::BAD_REQUEST,
        ResponseGeneric::new(
          ResponseStatus::Failure,
          Some(CodesResponse::GenericOrUnknown),
          Some(err.to_cowstr()),
        ),
      ),
      Self::Threading(err) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        ResponseGeneric::new(
          ResponseStatus::Failure,
          Some(CodesResponse::GenericOrUnknown),
          Some(err.to_cowstr()),
        ),
      ),
      Self::ValidationError(_) => {
        let message = format!("Input validation error: [{}]", self).replace('\n', ", ");
        (
          StatusCode::BAD_REQUEST,
          ResponseGeneric::new(
            ResponseStatus::Failure,
            Some(CodesResponse::TemplateAskama),
            Some(message),
          ),
        )
      }
    };

    (status, payload).into_response()
  }
}
