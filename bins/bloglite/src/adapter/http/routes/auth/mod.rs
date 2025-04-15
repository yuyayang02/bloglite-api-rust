use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, request::Parts},
    routing::get,
    Router,
};
use lib_api::Result;

use crate::application::{auth, AppState};

pub fn setup(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/refresh", get(refresh_token))
        .with_state(state)
}

async fn refresh_token(parts: Parts, State(jwt): State<auth::JwtState>) -> Result<String> {
    let token = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(auth::AuthError::MissingToken)?;

    // 验证 refresh token
    let _ = jwt.validate_refresh_token(token)?;

    // 生成 access token
    let token = jwt.sign_access_token()?;

    Ok(token.into())
}
