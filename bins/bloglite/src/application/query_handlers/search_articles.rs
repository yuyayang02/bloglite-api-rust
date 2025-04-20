use crate::{application, infra::readmodel};

use super::{ArticleForAdminResult, ArticleListResult, ArticleMetaResult, CategoryResult};

pub struct Query {
    pub page: i32,
    pub limit: i32,
    pub category: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
}

pub struct QueryHandler<R = super::role::Api> {
    pub(in crate::application) db: lib_db::Db,
    pub(in crate::application) _type: std::marker::PhantomData<R>,
}

pub type QueryHandlerForAdmin = QueryHandler<super::role::Admin>;

impl lib_cqrs::QueryHandler for QueryHandler {
    type Query = Query;
    type Result = ArticleListResult;
    type Error = application::Error;
    async fn handle(&self, query: Self::Query) -> Result<Self::Result, Self::Error> {
        let (rows, total) = readmodel::ArticleQueryBuilder::get_with_filter(
            &self.db,
            query.page,
            query.limit,
            query.category,
            &query.tags,
            query.author,
            false,
        )
        .await?;

        Ok(Self::Result {
            total: total as usize,
            limit: query.page as usize,
            page: query.limit as usize,
            count: rows.len(),
            items: rows
                .into_iter()
                .map(|a| ArticleMetaResult {
                    slug: a.slug,
                    title: a.title,
                    summary: a.rendered_summary,
                    tags: a.tags,
                    author: a.author,
                    category: CategoryResult {
                        id: a.category_id,
                        name: a.category_name,
                    },
                    created_at: a.created_at.timestamp_millis(),
                    updated_at: a.updated_at.timestamp_millis(),
                })
                .collect(),
        })
    }
}

impl lib_cqrs::QueryHandler for QueryHandlerForAdmin {
    type Query = Query;
    type Result = ArticleListResult<ArticleForAdminResult>;
    type Error = application::Error;
    async fn handle(&self, query: Self::Query) -> Result<Self::Result, Self::Error> {
        let (rows, total) = readmodel::ArticleQueryBuilder::get_with_filter(
            &self.db,
            query.page,
            query.limit,
            query.category,
            &query.tags,
            query.author,
            true,
        )
        .await?;

        Ok(Self::Result {
            total: total as usize,
            limit: query.limit as usize,
            page: query.page as usize,
            count: rows.len(),
            items: rows
                .into_iter()
                .map(|a| ArticleForAdminResult {
                    parent: ArticleMetaResult {
                        slug: a.slug,
                        title: a.title,
                        summary: a.rendered_summary,
                        tags: a.tags,
                        author: a.author,
                        category: CategoryResult {
                            id: a.category_id,
                            name: a.category_name,
                        },
                        created_at: a.created_at.timestamp_millis(),
                        updated_at: a.updated_at.timestamp_millis(),
                    },
                    // content: a.rendered_content,
                    state: a.state,
                    version: a.current_version,
                    id: a.id,
                })
                .collect(),
        })
    }
}
