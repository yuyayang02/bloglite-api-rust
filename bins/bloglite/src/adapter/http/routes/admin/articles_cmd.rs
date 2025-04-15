use axum::{
    extract::{Multipart, Path, State},
    routing::{delete, patch, post, Router},
    Json,
};
use serde::Deserialize;

use std::sync::Arc;

use crate::application::{self, AppState};

use application as app;
use lib_api::{extract::WrapRejection, Result, SuccessResponse};
use lib_cqrs::CommandHandler;

pub fn setup(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create))
        .route("/{slug}", post(update_content))
        .route("/{slug}", delete(remove))
        .route("/{slug}/version", patch(revert_content))
        .route("/{slug}/category", patch(set_category))
        .route("/{slug}/state", patch(set_state))
        .with_state(state)
}

/// 创建文章
async fn create(
    State(handler): State<app::create_article::CommandHandler>,
    WrapRejection(mut multipart): WrapRejection<Multipart>,
) -> Result<()> {
    // 初始化命令
    let mut cmd = app::create_article::Command::default();

    // 提取数据
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| app::Error::InvalidInput)?
    {
        match field.name().unwrap_or_default() {
            // "author" => cmd.user_id = field.text().await.map_err(|_| app::Error::InvalidParams)?,
            "slug" => cmd.slug = field.text().await.map_err(|_| app::Error::InvalidParams)?,
            "category" => {
                cmd.category = field.text().await.map_err(|_| app::Error::InvalidParams)?
            }
            "document" => {
                cmd.markdown_document = field.text().await.map_err(|_| app::Error::InvalidParams)?
            }
            _ => return Err(app::Error::InvalidParams.into()),
        }
    }

    // 处理命令
    handler.handle(cmd).await?;

    Ok(().into())
}

///  更新文章内容
async fn update_content(
    Path(slug): Path<String>,
    State(handler): State<app::update_article_content::CommandHandler>,
    WrapRejection(mut multipart): WrapRejection<Multipart>,
) -> Result<()> {
    let mut cmd = app::update_article_content::Command::default();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| app::Error::InvalidInput)?
    {
        match field.name().unwrap_or_default() {
            "document" => {
                cmd.markdown_document = field.text().await.map_err(|_| app::Error::InvalidParams)?
            }
            _ => return Err(app::Error::InvalidParams.into()),
        }
    }

    cmd.slug = slug;

    handler.handle(cmd).await?;

    Ok(().into())
}

#[derive(Deserialize)]
struct RevertArticleVersionJson {
    version: String,
}

/// 恢复文章内容
async fn revert_content(
    Path(slug): Path<String>,
    State(handler): State<app::revert_article_content::CommandHandler>,
    Json(req): Json<RevertArticleVersionJson>,
) -> Result<()> {
    handler
        .handle(app::revert_article_content::Command {
            slug,
            target_version: req.version,
        })
        .await?;

    SuccessResponse::ok()
}

/// 删除文章
async fn remove(
    Path(slug): Path<String>,
    State(handler): State<app::delete_article::CommandHandler>,
) -> Result<()> {
    handler
        .handle(app::delete_article::Command { slug })
        .await?;
    Ok(().into())
}

#[derive(Deserialize)]
struct SetArticleCategoryJson {
    category: String,
}

/// 设置文章分类
async fn set_category(
    Path(slug): Path<String>,
    State(handler): State<app::set_article_category::CommandHandler>,
    Json(req): Json<SetArticleCategoryJson>,
) -> Result<()> {
    handler
        .handle(app::set_article_category::Command {
            slug,
            new_category: req.category,
        })
        .await?;

    Ok(().into())
}

#[derive(Deserialize)]
struct SetArticleStateJson {
    state: u8,
}

/// 设置文章状态
async fn set_state(
    Path(slug): Path<String>,
    State(handler): State<app::set_article_state::CommandHandler>,
    Json(req): Json<SetArticleStateJson>,
) -> Result<()> {
    handler
        .handle(app::set_article_state::Command {
            slug,
            state: req.state,
        })
        .await?;

    Ok(().into())
}
