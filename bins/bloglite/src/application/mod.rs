pub mod auth;
mod command_handlers;
mod error;
pub mod query_handlers;

use axum::extract::FromRef;
use std::sync::Arc;

use crate::domain::articles;
use crate::{
    infra,
    infra::domain::{ArticleContentHasher, ArticleContentParser, ArticleContentRender},
};

pub use command_handlers::*;
pub use error::Error;
pub use query_handlers::*;

// 在application模块阻止泛型参数传播
//
// ArticleContentFactory
pub type ArticleContentFactory = articles::content::ContentFactory<
    ArticleContentParser,
    ArticleContentRender,
    ArticleContentHasher,
>;

// ArticleRepository
type ArticleRepository = infra::domain::ArticleRepository;

// CategoryRepository
type CategoryRepository = infra::domain::CategoryRepository;

pub struct AppState {
    db: lib_db::Db,
    content_factory: Arc<ArticleContentFactory>,
    article_repository: Arc<ArticleRepository>,
    category_repository: Arc<CategoryRepository>,
    jwt: auth::JwtState,
}

impl AppState {
    pub fn new(
        db: lib_db::Db,
        content_factory: ArticleContentFactory,
        jwt: auth::JwtState,
    ) -> Self {
        AppState {
            content_factory: Arc::new(content_factory),
            article_repository: Arc::new(ArticleRepository::new(db.clone())),
            db: db.clone(),
            category_repository: Arc::from(CategoryRepository::new(db.clone())),
            jwt,
        }
    }
}

// app to article command handler
impl FromRef<Arc<AppState>> for create_article::CommandHandler {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            content_factory: input.content_factory.clone(),
            article_repository: input.article_repository.clone(),
            category_repository: input.category_repository.clone(),
        }
    }
}

impl FromRef<Arc<AppState>> for update_article_content::CommandHandler {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            content_factory: input.content_factory.clone(),
            article_repository: input.article_repository.clone(),
        }
    }
}
impl FromRef<Arc<AppState>> for delete_article::CommandHandler {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            article_repository: input.article_repository.clone(),
        }
    }
}

impl FromRef<Arc<AppState>> for set_article_category::CommandHandler {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            article_repository: input.article_repository.clone(),
            category_repository: input.category_repository.clone(),
        }
    }
}

impl FromRef<Arc<AppState>> for set_article_state::CommandHandler {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            article_repository: input.article_repository.clone(),
        }
    }
}

impl FromRef<Arc<AppState>> for revert_article_content::CommandHandler {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            article_repository: input.article_repository.clone(),
        }
    }
}

// app to auth/jwt
impl FromRef<Arc<AppState>> for auth::JwtState {
    fn from_ref(input: &Arc<AppState>) -> Self {
        input.jwt.clone()
    }
}

// app to readmodel
impl<R> FromRef<Arc<AppState>> for get_article::QueryHandler<R> {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            db: input.db.clone(),
            _type: std::marker::PhantomData,
        }
    }
}

impl<R> FromRef<Arc<AppState>> for get_all_tags::QueryHandler<R> {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            db: input.db.clone(),
            _type: std::marker::PhantomData,
        }
    }
}

impl<R> FromRef<Arc<AppState>> for search_articles::QueryHandler<R> {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            db: input.db.clone(),
            _type: std::marker::PhantomData,
        }
    }
}

impl FromRef<Arc<AppState>> for get_all_categories::QueryHandler {
    fn from_ref(input: &Arc<AppState>) -> Self {
        Self {
            category_repository: input.category_repository.clone(),
        }
    }
}
