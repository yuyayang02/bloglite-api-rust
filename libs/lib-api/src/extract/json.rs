use axum::response::IntoResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct SuccessResponse<T: Serialize> {
    code: u32,
    #[serde(skip_serializing_if = "is_unit")]
    data: T,
}

impl<T: Serialize> SuccessResponse<T> {
    const fn new(data: T) -> Self {
        Self { code: 0, data }
    }
}

impl<T: Serialize> From<Json<T>> for SuccessResponse<T> {
    fn from(value: Json<T>) -> Self {
        Self::new(value.0)
    }
}

#[derive(Serialize)]
pub struct Json<T: Serialize>(pub T);

impl<T: Serialize> Json<T> {
    const fn new(data: T) -> Self {
        Self(data)
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> axum::response::Response {
        axum::Json(SuccessResponse::from(self)).into_response()
    }
}

/// 检查值是否是 `()` 的条件函数
///
/// 一般对单元类型也起效（`struct A;`）
fn is_unit<T>(_: &T) -> bool {
    std::mem::size_of::<T>() == 0 // 通过类型大小判断是否是 `()` 类型
}
