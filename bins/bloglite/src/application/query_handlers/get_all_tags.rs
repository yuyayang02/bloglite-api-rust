use crate::{application, infra::readmodel};

pub struct QueryHandler<R = super::role::Api> {
    pub(in crate::application) db: lib_db::Db,
    pub(in crate::application) _type: std::marker::PhantomData<R>,
}

pub type QueryHandlerForAdmin = QueryHandler<super::role::Admin>;

impl lib_cqrs::QueryHandler for QueryHandler {
    type Query = ();
    type Result = super::ItemsResult<String>;
    type Error = application::Error;
    async fn handle(&self, _: Self::Query) -> Result<Self::Result, Self::Error> {
        Ok(readmodel::TagsQuery::get_tags(&self.db, false)
            .await?
            .into())
    }
}

impl lib_cqrs::QueryHandler for QueryHandlerForAdmin {
    type Query = ();
    type Result = super::ItemsResult<String>;
    type Error = application::Error;
    async fn handle(&self, _: Self::Query) -> Result<Self::Result, Self::Error> {
        Ok(readmodel::TagsQuery::get_tags(&self.db, true).await?.into())
    }
}
