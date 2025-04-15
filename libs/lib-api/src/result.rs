use super::error::ErrorResponse;
use super::success::SuccessResponse;

pub type Result<T, E = ErrorResponse> = core::result::Result<SuccessResponse<T>, E>;
