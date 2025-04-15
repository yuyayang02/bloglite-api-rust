// #![cfg_attr(
//     debug_assertions,
//     allow(dead_code, unused_imports, unused_variables, unused_mut)
// )]
pub(crate) mod _dev_utils;
pub(crate) mod adapter;
pub(crate) mod application;
pub(crate) mod config;
pub(crate) mod domain;
pub(crate) mod infra;

pub use application::auth;
use infra::outbox;
use std::sync::Arc;
use tracing_subscriber::{fmt::time::ChronoLocal, EnvFilter};

use domain::articles;

pub async fn run() {
    init_log();

    #[cfg(debug_assertions)]
    let db = _dev_utils::init_db().await;
    #[cfg(not(debug_assertions))]
    let db = init_db().await;

    let jwt = auth::JwtState::new();

    // 生成 refresh token 并写入 auth config
    jwt.generate_and_write_auth_config();

    let content_render = infra::domain::ArticleContentRender::new(
        std::env::var("MARKDOWN_RENDER_GITHUB_KEY").unwrap(),
    );

    let state = Arc::new(application::AppState::new(
        db.clone(),
        articles::content::ContentFactory::new(
            infra::domain::ArticleContentParser,
            infra::domain::ArticleContentHasher,
            content_render.clone(),
        ),
        jwt,
    ));

    let (shutdown_send, shutdown_recv) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        shutdown_send.send(()).ok();
    });

    tokio::select! {
        _ = async {
            tokio::join!(
                adapter::http::run_server(state, "0.0.0.0:3000"),
                outbox::init_outbox(content_render, db)
            );
        } => {},
        _ = shutdown_recv => {
            tracing::info!("Shutting down gracefully...");
        }
    };
}

fn init_log() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S%.3f".to_string()))
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

#[allow(unused)]
async fn init_db() -> lib_db::Db {
    let db = lib_db::init_db_from_env().await;
    lib_db::migrate(&db, "sql/prod_initial/create-schema.sql")
        .await
        .unwrap();
    db
}
