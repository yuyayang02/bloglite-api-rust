use std::sync::Arc;

use crate::{
    application,
    domain::articles::{self, repository::ArticleRepository},
};

pub struct Command {
    pub id: String,
    pub state: u8,
}

pub struct CommandHandler {
    pub(in crate::application) article_repository: Arc<application::ArticleRepository>,
}

impl lib_cqrs::CommandHandler for CommandHandler {
    type Command = Command;
    type Error = application::Error;

    async fn handle(&self, cmd: Self::Command) -> Result<(), Self::Error> {
        let id = articles::ArticleId::try_from(cmd.id)?;

        let article = self
            .article_repository
            .find(&id)
            .await?
            .ok_or(application::Error::ResourceNotFound)?;

        let (article, event) = match cmd.state {
            0 => article.private()?,
            1 => article.public()?,
            _ => return Err(application::Error::InvalidInput),
        };

        self.article_repository
            .save_all(article, [event.into()])
            .await?;

        Ok(())
    }
}
