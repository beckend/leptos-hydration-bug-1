use axum::{
  response::{IntoResponse, Response},
  Json,
};
use cowstr::{CowStr, ToCowStr};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strum::AsRefStr;

use crate::ssr::errors::codes::CodesResponse;

#[derive(AsRefStr, Debug, Serialize, Deserialize, utoipa::ToSchema, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum ResponseStatus {
  Success,
  Failure,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, PartialEq, Eq)]
pub struct ResponseGeneric {
  pub code: Option<CodesResponse>,
  // utopia does not like lifetime if this is cow
  #[schema(value_type = Option<String>)]
  pub message: Option<CowStr>,
  pub status: ResponseStatus,
}

impl ResponseGeneric {
  pub fn new<'b, TMessage: Into<CowStr>>(
    status: ResponseStatus,
    code: Option<CodesResponse>,
    message: Option<TMessage>,
  ) -> Self {
    let msg = match message {
      Some(x) => x.into(),
      None => "".to_cowstr(),
    };

    Self {
      code,
      status,
      message: Some(msg),
    }
  }
}

impl IntoResponse for ResponseGeneric {
  fn into_response(self) -> Response {
    Json(self).into_response()
  }
}
