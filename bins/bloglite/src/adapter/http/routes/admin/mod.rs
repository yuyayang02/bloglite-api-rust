mod articles_cmd;
mod articles_query;

use std::sync::Arc;

use axum::Router;

use crate::{adapter::http::middleware, application::AppState};

pub fn setup(state: Arc<AppState>) -> Router {
    Router::new()
        .nest(
            "/articles",
            articles_query::setup(state.clone()).merge(articles_cmd::setup(state.clone())),
        )
        .layer(axum::middleware::from_fn_with_state(
            state,
            middleware::auth_middleware,
        ))
}
