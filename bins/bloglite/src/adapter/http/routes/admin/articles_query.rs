use std::sync::Arc;

use axum::{extract::State, routing::get, Router};

use axum_extra::extract::Query;
use lib_api::{ApiResult, Json};
use lib_cqrs::QueryHandler;

use crate::application::{
    get_all_categories, get_all_tags, query_handlers, search_articles, AppState,
};

const fn admin_default_page() -> i32 {
    1
}
const fn admin_default_limit() -> i32 {
    20
}

pub fn setup(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list))
        // .route("/{slug}", get(article))
        .route("/tags", get(tag_list))
        .route("/categories", get(category_list))
        .with_state(state)
}

// async fn article(
//     Path(slug): Path<String>,
//     State(handler): State<get_article::QueryHandlerForAdmin>,
// ) -> Result<query_handlers::ArticleForAdminResult> {
//     Ok(handler.handle(get_article::Query { slug }).await?.into())
// }

#[derive(serde::Deserialize)]
struct GetListQuery {
    #[serde(default = "admin_default_page")]
    page: i32,
    #[serde(default = "admin_default_limit")]
    limit: i32,
    category: Option<String>,
    author: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

async fn list(
    Query(query): Query<GetListQuery>,
    State(handler): State<search_articles::QueryHandlerForAdmin>,
) -> ApiResult<Json<query_handlers::ArticleListResult<query_handlers::ArticleForAdminResult>>> {
    Ok(Json(
        handler
            .handle(search_articles::Query {
                page: query.page,
                limit: query.limit,
                category: query.category,
                author: query.author,
                tags: query.tags,
            })
            .await?,
    ))
}

async fn tag_list(
    State(handler): State<get_all_tags::QueryHandlerForAdmin>,
) -> ApiResult<Json<query_handlers::ItemsResult<String>>> {
    Ok(Json(handler.handle(()).await?))
}

async fn category_list(
    State(handler): State<get_all_categories::QueryHandler>,
) -> ApiResult<Json<query_handlers::ItemsResult<query_handlers::CategoryResult>>> {
    Ok(Json(handler.handle(()).await?))
}
