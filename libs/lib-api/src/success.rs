use axum::{response::IntoResponse, Json};
use serde::Serialize;

use crate::ErrorResponse;

#[derive(Serialize)]
pub struct SuccessResponse<T: Serialize> {
    code: u32,
    #[serde(skip_serializing_if = "is_unit")]
    data: T,
}

impl<T: Serialize> IntoResponse for SuccessResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

impl<T: Serialize> SuccessResponse<T> {
    const fn new(data: T) -> Self {
        Self { code: 0, data }
    }

    pub const fn data(data: T) -> Result<Self, ErrorResponse> {
        Ok(Self::new(data))
    }
}

impl SuccessResponse<()> {
    pub const fn ok() -> Result<Self, ErrorResponse> {
        Ok(Self { code: 0, data: () })
    }
}

impl<T: Serialize> From<T> for SuccessResponse<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// 检查值是否是 `()` 的条件函数
///
/// 一般对单元类型也起效（`struct A;`）
fn is_unit<T>(_: &T) -> bool {
    std::mem::size_of::<T>() == 0 // 通过类型大小判断是否是 `()` 类型
}
