pub mod content;
mod error;
pub mod events;
pub mod repository;
pub mod version;

use std::sync::OnceLock;

pub use error::{Error, Result};

// -- article fields
macro_rules! article_value_object {
    ($name:ident $(, $role_fn:expr)?) => {
        #[derive(Default, Clone)]
        pub struct $name(String);
        impl $name {
            fn validate(&self) -> Result<()> {
                $( $role_fn(self)?; )?
                Ok(())
            }
        }
        impl TryFrom<String> for $name {
            type Error = Error;
            fn try_from(value: String) -> Result<Self> {
                let a = Self(value);
                a.validate()?;
                Ok(a)
            }
        }

        impl AsRef<str> for $name {
            #[inline]
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
        impl std::ops::Deref for  $name {
            type Target = str;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0.as_str().to_string()
            }
        }

        impl PartialEq<String> for &$name {
            fn eq(&self, other: &String) -> bool {
                self.0 == *other
            }
        }
    };
}

article_value_object!(ArticleSlug, |slug: &ArticleSlug| {
    const ARTICLE_SLUG_MAX_LENGTH: usize = 25;
    static ROLE: OnceLock<regex::Regex> = OnceLock::new();
    let role = ROLE.get_or_init(|| regex::Regex::new(r"^[a-zA-Z0-9-]+$").unwrap());

    if slug.len() > ARTICLE_SLUG_MAX_LENGTH
        || slug.is_empty()
        || slug.contains(' ')
        || !role.is_match(slug)
    {
        return Err(Error::ArticleSlugFormatError);
    };

    Ok(())
});

article_value_object!(ArticleCategory, |category: &ArticleCategory| {
    if category.is_empty() {
        Err(Error::ArticleCategoryFormatError)
    } else {
        Ok(())
    }
});

article_value_object!(ArticleId, |id: &ArticleId| {
    ulid::Ulid::from_string(&id.0)
        .map(|_| ())
        .map_err(|_| Error::ArticleIdFormatError)
});
article_value_object!(ArticleAuthor);

// -- article state
#[derive(Clone)]
pub enum ArticleState {
    Deleted,
    Private,
    Public,
}

// state -> u8
impl From<ArticleState> for i16 {
    fn from(value: ArticleState) -> Self {
        match value {
            ArticleState::Deleted => -1, // -1 已删除
            ArticleState::Private => 0,  // 0 私有
            ArticleState::Public => 1,   // 1 公开
        }
    }
}

// -- article type

pub struct Article {
    /// 文章唯一标识
    pub(self) id: ArticleId,

    /// 文章slug
    pub(self) slug: ArticleSlug,

    /// 文章分类
    pub(self) category: ArticleCategory,

    /// 文章历史版本树
    pub(self) version_history: version::VersionHistory,

    /// 文章状态：公开/私有
    pub(self) state: ArticleState,
}

impl Article {
    // 文章字段
    pub fn id(&self) -> &ArticleId {
        &self.id
    }

    pub fn slug(&self) -> &ArticleSlug {
        &self.slug
    }

    pub fn category(&self) -> &ArticleCategory {
        &self.category
    }
    pub fn state(&self) -> &ArticleState {
        &self.state
    }
    pub fn version_history(&self) -> &version::VersionHistory {
        &self.version_history
    }

    /// 公开文章
    pub fn public(self) -> Result<(Article, events::ArticleStateChanged)> {
        match self.state {
            ArticleState::Public => Err(Error::ArticleStatusNoChanged),
            ArticleState::Deleted => Err(Error::ArticleDeleted),
            ArticleState::Private => Ok((
                Article {
                    id: self.id.clone(),
                    slug: self.slug,
                    version_history: self.version_history,
                    category: self.category,
                    state: ArticleState::Public,
                },
                events::ArticleStateChanged {
                    id: self.id.into(),
                    state: ArticleState::Public.into(),
                },
            )),
        }
    }

    /// 设置文章私有
    pub fn private(self) -> Result<(Article, events::ArticleStateChanged)> {
        match self.state {
            ArticleState::Private => Err(Error::ArticleStatusNoChanged),
            ArticleState::Deleted => Err(Error::ArticleDeleted),
            ArticleState::Public => Ok((
                Article {
                    id: self.id.clone(),
                    slug: self.slug,
                    version_history: self.version_history,
                    category: self.category,
                    state: ArticleState::Private,
                },
                events::ArticleStateChanged {
                    id: self.id.into(),
                    state: ArticleState::Private.into(),
                },
            )),
        }
    }

    /// 更新文章内容
    pub fn update_content(
        &mut self,
        content: content::Content,
    ) -> Result<events::ArticleContentUpdated> {
        let prev_version = self.version_history.current_version_hash.to_string();

        self.version_history.add_version(&content.hash)?;

        Ok(events::ArticleContentUpdated {
            id: self.id.clone().to_string(),
            parent_version: prev_version,
            current_version: self.version_history.current_version_hash.to_string(),

            title: content.frontmatter.title.into(),
            tags: content.frontmatter.tags.into(),
            summary: content.frontmatter.summary.into(),
            body: content.body.into(),
            rendered_body: content.rendered_body.into(),
            rendered_summary: content.rendered_summary.into(),
        })
    }

    /// 恢复文章内容到某版本
    pub fn revert_to_version<T: AsRef<str>>(
        &mut self,
        hash: T,
    ) -> Result<events::ArticleContentReverted> {
        let prev_version = self.version_history.current_version_hash.to_string();

        self.version_history.rollback_to_version(&hash)?;

        Ok(events::ArticleContentReverted {
            id: self.id.clone().into(),
            prev_version,
            current_version: self.version_history.current_version_hash.to_string(),
        })
    }

    /// 修改文章分类
    pub fn change_article_category<T: Into<String>>(
        &mut self,
        categroy_id: T,
        is_valid: bool,
    ) -> Result<events::ArticleCategoryChanged> {
        if !is_valid {
            return Err(Error::InvalidCategory);
        };

        let category = ArticleCategory::try_from(categroy_id.into())?;

        self.check_duplicate_category(&category)?;

        self.category = category.clone();

        Ok(events::ArticleCategoryChanged {
            id: self.id.clone().into(),
            old_category_id: self.category.clone().into(),
            new_category_id: category.into(),
        })
    }

    /// 标记删除文章
    pub fn delete(&mut self) -> Result<events::ArticleDeleted> {
        self.state = ArticleState::Deleted;
        Ok(events::ArticleDeleted {
            id: self.id.clone().into(),
        })
    }

    /// 检查分类是否重复
    fn check_duplicate_category(&self, category: &ArticleCategory) -> Result<()> {
        match self.category.as_ref() == category.as_ref() {
            true => Err(Error::DuplicateArticleCategory),
            false => Ok(()),
        }
    }
}

// -- article builder ( factory )
#[derive(Default, Clone)]
pub struct NoSlug;

#[derive(Default, Clone)]
pub struct NoId;

#[derive(Default, Clone)]
pub struct NoAuthor;

#[derive(Default, Clone)]
pub struct NoCategory;

#[derive(Default, Clone)]
pub struct NoContent;

#[derive(Default, Clone)]
pub struct ArticleBuilder<S, A, CA, CO> {
    id: ulid::Ulid,
    slug: S,
    // 当slug冲突时启用
    slug_conflict: Option<u8>,
    author: A,
    category: CA,
    is_valid_category: bool, // 默认值为false
    content: CO,
}

impl ArticleBuilder<NoSlug, NoAuthor, NoCategory, NoContent> {
    pub fn new() -> Self {
        Self {
            id: ulid::Ulid::new(),
            ..Self::default()
        }
    }
}

impl<S, A, CA, CO> ArticleBuilder<S, A, CA, CO> {
    pub fn slug<T: Into<String>>(self, slug: T) -> ArticleBuilder<ArticleSlug, A, CA, CO> {
        ArticleBuilder {
            id: self.id,
            slug: ArticleSlug(slug.into()),
            author: self.author,
            category: self.category,
            content: self.content,
            is_valid_category: self.is_valid_category,
            slug_conflict: self.slug_conflict,
        }
    }

    pub fn author<T: Into<String>>(self, author: T) -> ArticleBuilder<S, ArticleAuthor, CA, CO> {
        ArticleBuilder {
            id: self.id,
            slug: self.slug,
            author: ArticleAuthor(author.into()),
            category: self.category,
            content: self.content,
            is_valid_category: self.is_valid_category,
            slug_conflict: self.slug_conflict,
        }
    }

    pub fn category<T: Into<String>>(
        self,
        category: T,
        is_valid: bool,
    ) -> ArticleBuilder<S, A, ArticleCategory, CO> {
        // if !is_valid {
        //     return Err(Error::InvalidCategory);
        // }

        ArticleBuilder {
            id: self.id,
            slug: self.slug,
            author: self.author,
            category: ArticleCategory(category.into()),
            content: self.content,
            is_valid_category: is_valid,
            slug_conflict: self.slug_conflict,
        }
    }

    pub fn content(self, content: content::Content) -> ArticleBuilder<S, A, CA, content::Content> {
        ArticleBuilder {
            id: self.id,
            slug: self.slug,
            author: self.author,
            category: self.category,
            content: content,
            is_valid_category: self.is_valid_category,
            slug_conflict: self.slug_conflict,
        }
    }
}

// 当所有泛型参数设置完后，就可以`build`
impl ArticleBuilder<ArticleSlug, ArticleAuthor, ArticleCategory, content::Content> {
    // 从仓储创建聚合
    // 可能有创建未经验证的聚合的风险，谨慎使用
    pub(crate) fn only_from_repository<T: Into<String>>(
        id: T,
        slug: T,
        category: T,
        state: ArticleState,
        history: version::VersionHistory,
    ) -> Article {
        Article {
            id: ArticleId(id.into()),
            slug: ArticleSlug(slug.into()),
            category: ArticleCategory(category.into()),
            version_history: history,
            state,
        }
    }

    // 构建聚合
    pub fn build(mut self) -> Result<(Article, events::ArticleCreated)> {
        // 检查分类有效性（通过注入flag参数分离具体行为和业务逻辑）
        if !self.is_valid_category {
            return Err(Error::InvalidCategory);
        };

        // 历史版本树
        let history = version::VersionHistory::new(self.content.hash)?;

        // 当前版本号
        let current_version = history.current_version_hash.to_string();

        if let Some(i) = self.slug_conflict {
            self.slug = ArticleSlug(format!("{}-{}", self.slug.as_ref(), i))
        }

        // 校验参数
        self.slug.validate()?;
        self.category.validate()?;
        // self.author.validate()?; // 文章领域不关心作者

        // 返回实体和创建事件（events::ArticleCreated）
        Ok((
            Article {
                id: ArticleId(self.id.to_string()),
                slug: self.slug.clone(),
                category: self.category.clone(),
                version_history: history,
                state: ArticleState::Private,
            },
            events::ArticleCreated {
                id: self.id.to_string(),
                slug: self.slug.to_string(),
                author: self.author.to_string(),
                category_id: self.category.to_string(),
                current_version,
                state: ArticleState::Private.into(),
                title: self.content.frontmatter.title.into(),
                summary: self.content.frontmatter.summary.into(),
                body: self.content.body.into(),
                tags: self.content.frontmatter.tags.into(),
                rendered_body: self.content.rendered_body.into(),
                rendered_summary: self.content.rendered_summary.into(),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use content::tests::create_content;
    fn default_mock_content() -> content::Content {
        create_content("title", "summary", "body", "hash")
    }
    #[test]
    fn test_new_article() {
        let content = default_mock_content();

        let (article, event) = ArticleBuilder::new()
            .slug("slug")
            .author("author")
            .category("category", true)
            .content(content)
            .build()
            .unwrap();

        assert_eq!(article.category.as_ref(), "category");

        assert_eq!(event.author.as_str(), "author");
        assert_eq!(event.slug.as_str(), "slug");
        assert_eq!(event.current_version.as_str(), "hash");
        assert_eq!(event.category_id.as_str(), "category");
        assert_eq!(event.state, 0);
    }

    #[test]
    fn test_article_update_content() {
        let content = default_mock_content();

        let (mut article, _) = ArticleBuilder::new()
            .slug("slug")
            .author("author")
            .category("category", true)
            .content(content)
            .build()
            .unwrap();

        assert!(matches!(
            article.update_content(default_mock_content()),
            Err(Error::VersionError(version::Error::DuplicateVersion(_)))
        ));

        let content2 = create_content("title2", "summary2", "body2", "hash2");

        let event = article.update_content(content2).unwrap();

        assert_eq!(article.category.as_ref(), "category");

        assert_eq!(event.parent_version.as_str(), "hash");
        assert_eq!(event.current_version.as_str(), "hash2");
    }

    #[test]
    fn test_article_restore_content() {
        let content = default_mock_content();

        let (mut article, _) = ArticleBuilder::new()
            .slug("slug")
            .author("author")
            .category("category", true)
            .content(content)
            .build()
            .unwrap();

        let content2 = create_content("title2", "summary2", "body2", "hash2");

        article.update_content(content2).unwrap();

        assert_eq!(
            article.version_history.current_version_hash.as_ref(),
            "hash2"
        );

        assert!(matches!(
            article.revert_to_version("not_exist_hash"),
            Err(Error::VersionError(version::Error::VersionNotFound(x))) if x == "not_exist_hash"
        ));

        let event = article.revert_to_version("hash").unwrap();

        assert_eq!(
            article.version_history.current_version_hash.as_ref(),
            "hash"
        );

        assert_eq!(article.category.as_ref(), "category");

        assert_eq!(event.current_version.as_str(), "hash");
    }

    #[test]
    fn test_only_from_repository() {
        let history = version::VersionHistory::new(version::Version::new("hash").unwrap()).unwrap();

        let article = ArticleBuilder::only_from_repository(
            ulid::Ulid::new().to_string().as_str(),
            "slug",
            "category",
            ArticleState::Private,
            history,
        );

        assert_eq!(article.category.as_ref(), "category");
    }
}
