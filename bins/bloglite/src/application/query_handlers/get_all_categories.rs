use std::sync::Arc;

use crate::application;

pub struct QueryHandler {
    pub(in crate::application) category_repository: Arc<application::CategoryRepository>,
}

impl lib_cqrs::QueryHandler for QueryHandler {
    type Query = ();
    type Result = super::ItemsResult<super::CategoryResult>;
    type Error = application::Error;
    async fn handle(&self, _: Self::Query) -> Result<Self::Result, Self::Error> {
        Ok(self
            .category_repository
            .get_all()
            .await?
            .into_iter()
            .map(|category| super::CategoryResult {
                id: category.id().to_owned(),
                name: category.name().to_owned(),
            })
            .into())
    }
}
