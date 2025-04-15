use super::auth;
use crate::domain::articles;

use lib_api::ErrorCode as EC;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ArticleDomain(#[from] articles::Error),

    #[error("内部服务错误")]
    Database(#[from] lib_db::Error),

    #[error("{0}")]
    Auth(#[from] auth::AuthError),

    #[error("资源已存在")]
    ResourceAlreadyExists,

    #[error("资源不存在")]
    ResourceNotFound,

    #[error("无效输入")]
    InvalidInput,

    #[error("无效参数")]
    InvalidParams,
}

// 将 article::error 直接转为 app error
impl From<articles::content::Error> for Error {
    fn from(value: articles::content::Error) -> Self {
        Error::ArticleDomain(value.into())
    }
}

// 为 app::error 实现 api error trait
impl lib_api::ApiError for Error {
    fn as_error_code(&self) -> EC {
        match self {
            Error::ArticleDomain(error) => match error {
                articles::Error::ContentError(error) => error.into(),
                articles::Error::VersionError(error) => error.into(),
                articles::Error::ArticleDeleted
                | articles::Error::DuplicateArticleCategory
                | articles::Error::ArticleStatusNoChanged => EC::OperationNotAllowed,
                articles::Error::InvalidCategory => EC::DependencyNotSatisfied,
                articles::Error::ArticleCategoryFormatError
                | articles::Error::ArticleSlugFormatError => EC::InvalidInput,
            },
            Error::Database(_) => EC::DatabaseError,
            Error::ResourceAlreadyExists => EC::ResourceAlreadyExists,
            Error::ResourceNotFound => EC::ResourceNotFound,
            Error::InvalidInput | Error::InvalidParams => EC::InvalidInput,
            Error::Auth(_) => EC::InvalidToken,
        }
    }
}

// 为 article::content::error 实现 api error trait
impl From<&articles::content::Error> for EC {
    fn from(error: &articles::content::Error) -> Self {
        match error {
            articles::content::Error::MissingField(_)
            | articles::content::Error::EmptyField(_)
            | articles::content::Error::ParseError(_)
            | articles::content::Error::HashingError(_)
            | articles::content::Error::RenderError(_) => EC::InvalidInput,

            articles::content::Error::BodyTooLong
            | articles::content::Error::SummaryTooLong
            | articles::content::Error::TitleTooLong
            | articles::content::Error::TagTooLong
            | articles::content::Error::TagTooMany
            | articles::content::Error::InvalidTagFormat => EC::DataValidationFailed,
        }
    }
}

// 为 article::version::error 实现 api error trait
impl From<&articles::version::Error> for EC {
    fn from(_: &articles::version::Error) -> Self {
        // 该错误应返回的错误码唯一，所以省略该代码片段
        // match error {
        //     articles::version::Error::EmptyHashValue
        //     | articles::version::Error::DuplicateVersion(_)
        //     | articles::version::Error::VersionNotFound(_) => EC::InvalidInput,
        // }
        EC::InvalidInput
    }
}
