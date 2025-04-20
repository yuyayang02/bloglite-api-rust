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
    pub category: String,
    pub user_id: String,
    pub markdown_document: String,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            user_id: "于野".to_string(),
            category: Default::default(),
            slug: Default::default(),
            markdown_document: Default::default(),
        }
    }
}

pub struct CommandHandler {
    pub(in crate::application) content_factory: Arc<application::ArticleContentFactory>,
    pub(in crate::application) article_repository: Arc<application::ArticleRepository>,
    pub(in crate::application) category_repository: Arc<application::CategoryRepository>,
}

impl lib_cqrs::CommandHandler<(String,)> for CommandHandler {
    type Command = Command;
    type Error = application::Error;
    async fn handle(&self, cmd: Self::Command) -> Result<(String,), Self::Error> {
        // 校验slug格式
        let slug = articles::ArticleSlug::try_from(cmd.slug)?;

        // 检查是否已存在
        self.article_repository
            .find_by_slug(&slug)
            .await? // 返回 Option<Article>
            .map(|_| Err(application::error::Error::ResourceAlreadyExists)) // 返回 Option<Result(Err)>
            .unwrap_or(Ok(()))?; // 如果option is none返回Ok(())，否则返回内部值Result，然后`?`解包固定得到Err

        let is_valid = self
            .category_repository
            .find(&cmd.category)
            .await?
            .is_some();

        // 处理文章内容
        let content = self.content_factory.process(cmd.markdown_document).await?;

        // 创建文章聚合
        let builder = articles::ArticleBuilder::new()
            .slug(slug)
            .author(cmd.user_id)
            .category(cmd.category, is_valid)
            .content(content);

        let (article, event) = builder.build()?;
        let id = article.id().to_string();

        self.article_repository
            .save_all(article, [event.into()])
            .await?;

        Ok((id,))
    }
}
