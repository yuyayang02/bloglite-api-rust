use std::sync::Arc;

use crate::{
    application,
    domain::{
        articles::{self, repository::ArticleRepository},
        categories::CategoryRepository,
    },
};

pub struct Command {
    pub slug: String,
    pub new_category: String,
}

pub struct CommandHandler {
    pub(in crate::application) article_repository: Arc<application::ArticleRepository>,
    pub(in crate::application) category_repository: Arc<application::CategoryRepository>,
}

impl lib_cqrs::CommandHandler for CommandHandler {
    type Command = Command;
    type Error = application::Error;

    async fn handle(&self, cmd: Self::Command) -> Result<(), Self::Error> {
        let slug = articles::ArticleSlug::try_from(cmd.slug)?;

        let mut article = self
            .article_repository
            .find(&slug)
            .await?
            .ok_or(application::Error::ResourceNotFound)?;

        let is_valid = self
            .category_repository
            .find(&cmd.new_category)
            .await?
            .is_some();

        let event = article.change_article_category(cmd.new_category, is_valid)?;

        self.article_repository
            .save_all(article, [event.into()])
            .await?;

        Ok(())
    }
}
