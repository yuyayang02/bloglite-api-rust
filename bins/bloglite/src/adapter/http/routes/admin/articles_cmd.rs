use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue},
    routing::{delete, patch, post, Router},
};

use serde::Deserialize;

use std::sync::Arc;

use crate::application::{self, AppState};

use application as app;
use lib_api::{extract::WrapRejection, ApiResult, Json};
use lib_cqrs::CommandHandler;

pub fn setup(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(create))
        .route("/{id}", post(update_content))
        .route("/{id}", delete(remove))
        .route("/{id}/version", patch(revert_content))
        .route("/{id}/category", patch(set_category))
        .route("/{id}/state", patch(set_state))
        .with_state(state)
}

/// 创建文章
async fn create(
    State(handler): State<app::create_article::CommandHandler>,
    WrapRejection(mut multipart): WrapRejection<Multipart>,
) -> ApiResult<(HeaderMap, Json<()>)> {
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
    let (id,) = handler.handle(cmd).await?;

    Ok((
        {
            let mut header = HeaderMap::new();
            header.insert("Resource-Id", HeaderValue::from_str(&id).unwrap());
            header
        },
        Json(()),
    ))
}

///  更新文章内容
async fn update_content(
    Path(slug): Path<String>,
    State(handler): State<app::update_article_content::CommandHandler>,
    WrapRejection(mut multipart): WrapRejection<Multipart>,
) -> ApiResult<Json<()>> {
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

    cmd.id = slug;

    handler.handle(cmd).await?;

    Ok(Json(()))
}

#[derive(Deserialize)]
struct RevertArticleVersionJson {
    version: String,
}

/// 恢复文章内容
async fn revert_content(
    Path(id): Path<String>,
    State(handler): State<app::revert_article_content::CommandHandler>,
    axum::Json(req): axum::Json<RevertArticleVersionJson>,
) -> ApiResult<Json<()>> {
    handler
        .handle(app::revert_article_content::Command {
            id,
            target_version: req.version,
        })
        .await?;

    Ok(Json(()))
}

/// 删除文章
async fn remove(
    Path(id): Path<String>,
    State(handler): State<app::delete_article::CommandHandler>,
) -> ApiResult<Json<()>> {
    handler.handle(app::delete_article::Command { id }).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
struct SetArticleCategoryJson {
    category: String,
}

/// 设置文章分类
async fn set_category(
    Path(slug): Path<String>,
    State(handler): State<app::set_article_category::CommandHandler>,
    axum::Json(req): axum::Json<SetArticleCategoryJson>,
) -> ApiResult<Json<()>> {
    handler
        .handle(app::set_article_category::Command {
            id: slug,
            new_category: req.category,
        })
        .await?;

    Ok(Json(()))
}

#[derive(Deserialize)]
struct SetArticleStateJson {
    state: u8,
}

/// 设置文章状态
async fn set_state(
    Path(slug): Path<String>,
    State(handler): State<app::set_article_state::CommandHandler>,
    axum::Json(req): axum::Json<SetArticleStateJson>,
) -> ApiResult<Json<()>> {
    handler
        .handle(app::set_article_state::Command {
            id: slug,
            state: req.state,
        })
        .await?;

    Ok(Json(()))
}
