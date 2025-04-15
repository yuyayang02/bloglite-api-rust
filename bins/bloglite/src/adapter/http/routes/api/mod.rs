use std::sync::Arc;

use axum::Router;

use crate::application::AppState;

mod articles;

pub fn setup(state: Arc<AppState>) -> Router {
    Router::new().nest("/articles", articles::setup(state))
}
