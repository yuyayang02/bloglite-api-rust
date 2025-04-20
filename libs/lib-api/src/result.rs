use axum::response::IntoResponse;
use serde::Serialize;

use super::error::ErrorResponse;

pub type ApiResult<T, E = ErrorResponse> = core::result::Result<T, E>;
