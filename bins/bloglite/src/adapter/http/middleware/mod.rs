use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

use crate::application::auth::{self, AuthError};

pub async fn auth_middleware(
    State(jwt): State<auth::JwtState>, // 从应用状态获取
    req: Request,
    next: Next,
) -> Result<Response, lib_api::ErrorResponse> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(AuthError::MissingToken)?;

    let _ = jwt.validate_access_token(token)?;

    Ok(next.run(req).await)
}
