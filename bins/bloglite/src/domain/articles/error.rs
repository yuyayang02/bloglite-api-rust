pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ContentError(#[from] super::content::Error),

    #[error(transparent)]
    VersionError(#[from] super::version::Error),

    #[error("文章slug格式无效")]
    ArticleSlugFormatError,

    #[error("文章category格式无效")]
    ArticleCategoryFormatError,

    #[error("无法重复分配相同的文章分类")]
    DuplicateArticleCategory,

    #[error("文章状态未发生变更")]
    ArticleStatusNoChanged,

    #[error("未注册的分类")]
    InvalidCategory,

    #[error("文章已删除，不可操作")]
    ArticleDeleted,
}
