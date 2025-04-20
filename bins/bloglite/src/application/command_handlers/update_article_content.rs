use std::sync::Arc;

use crate::{
    application,
    domain::articles::{self, repository::ArticleRepository},
};

#[derive(Default)]
pub struct Command {
    pub id: String,
    pub markdown_document: String,
}

pub struct CommandHandler {
    pub(in crate::application) content_factory: Arc<application::ArticleContentFactory>,
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

        let content = self.content_factory.process(cmd.markdown_document).await?;

        let event = article.update_content(content)?;

        self.article_repository
            .save_all(article, [event.into()])
            .await?;
        Ok(())
    }
}
