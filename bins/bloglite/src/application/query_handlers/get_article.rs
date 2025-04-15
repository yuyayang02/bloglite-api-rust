use crate::{application, infra::readmodel};

use super::{ArticleMetaResult, ArticleWithContentResult};

pub struct Query {
    pub slug: String,
}

pub struct QueryHandler<R = super::role::Api> {
    pub(in crate::application) db: lib_db::Db,
    pub(in crate::application) _type: std::marker::PhantomData<R>,
}

// pub type QueryHandlerForAdmin = QueryHandler<super::role::Admin>;

impl lib_cqrs::QueryHandler for QueryHandler {
    type Query = Query;
    type Result = ArticleWithContentResult;
    type Error = application::Error;
    async fn handle(&self, query: Self::Query) -> Result<Self::Result, Self::Error> {
        let row = readmodel::ArticleQueryBuilder::get_one(&self.db, &query.slug)
            .await?
            .ok_or(application::Error::ResourceNotFound)?;

        Ok(Self::Result {
            parent: ArticleMetaResult {
                slug: row.slug,
                title: row.title,
                summary: row.rendered_summary,
                tags: row.tags,
                author: row.author,
                category: super::CategoryResult {
                    id: row.category_id,
                    name: row.category_name,
                },
                created_at: row.created_at.timestamp_millis(),
                updated_at: row.updated_at.timestamp_millis(),
            },
            content: row.rendered_content,
            version: row.current_version,
        })
    }
}

// impl lib_cqrs::QueryHandler for QueryHandlerForAdmin {
//     type Query = Query;
//     type Result = ArticleForAdminResult;
//     type Error = application::Error;
//     async fn handle(&self, query: Self::Query) -> Result<Self::Result, Self::Error> {
//         let row = readmodel::ArticleQueryBuilder::get_one(&self.db, &query.slug)
//             .await?
//             .ok_or(application::Error::ResourceNotFound)?;

//         Ok(Self::Result {
//             parent: ArticleMetaResult {
//                 slug: row.slug,
//                 title: row.title,
//                 summary: row.rendered_summary,
//                 tags: row.tags,
//                 author: row.author,
//                 category: super::CategoryResult {
//                     id: row.category_id,
//                     name: row.category_name,
//                 },
//                 created_at: row.created_at.timestamp_millis(),
//                 updated_at: row.updated_at.timestamp_millis(),
//             },
//             // content: row.rendered_content,
//             version: row.current_version,
//             state: row.state,
//         })
//     }
// }
