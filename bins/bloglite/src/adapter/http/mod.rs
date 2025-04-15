mod middleware;
mod routes;

use axum::{extract::DefaultBodyLimit, Router};
use std::{sync::Arc, time::Duration};
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::instrument;

use crate::application::AppState;
use routes::{admin, api, auth};

fn setup_route_v1(state: Arc<AppState>) -> Router {
    Router::new().nest("/v1", {
        Router::new()
            .nest("/admin", admin::setup(state.clone()))
            .nest("/api", api::setup(state.clone()))
            .nest("/auth", auth::setup(state.clone()))
    })
}

fn setup_middleware(app: Router) -> Router {
    let logger_middleware = TraceLayer::new_for_http().on_failure(());

    app.layer(
        tower::ServiceBuilder::new()
            .layer(DefaultBodyLimit::disable())
            .layer(RequestBodyLimitLayer::new(lib_utils::consts::mb(5))) // 5 mb
            .layer(logger_middleware)
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            ),
    )
}
#[instrument(name = "http server", skip_all)]
pub async fn run_server(state: Arc<AppState>, addr: &'static str) {
    tracing::info!(addr, "start listening");

    let app = setup_route_v1(state);
    let app = setup_middleware(app);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap_or_else(|e| {
        eprint!("{}", e.to_string());
        std::process::exit(1)
    });
}
