use std::sync::Arc;

use crate::{
    application,
    domain::articles::{self, repository::ArticleRepository},
};

pub struct Command {
    pub id: String,
    pub target_version: String,
}
pub struct CommandHandler {
    pub(in crate::application) article_repository: Arc<application::ArticleRepository>,
}

impl lib_cqrs::CommandHandler for CommandHandler {
    type Command = Command;
    type Error = application::Error;
    async fn handle(&self, cmd: Self::Command) -> Result<(), Self::Error> {
        let id = articles::ArticleId::try_from(cmd.id)?;

        let mut article = self
            .article_repository
            .find(&id)
            .await?
            .ok_or(application::Error::ResourceNotFound)?;

        let event = article.revert_to_version(&cmd.target_version)?;

        self.article_repository
            .save_all(article, [event.into()])
            .await?;

        Ok(())
    }
}
