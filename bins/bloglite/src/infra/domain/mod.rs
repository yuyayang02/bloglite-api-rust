mod article_content_hasher;
mod article_content_parser;
mod article_content_render;

mod article_repository;
mod category_repository;

// article content factory 依赖
pub use article_content_hasher::ArticleContentHasher;
pub use article_content_parser::ArticleContentParser;

// 通过取消注释决定使用哪个Render
pub use article_content_render::GithubArticleContentRender as ArticleContentRender;
// pub use article_content_render::LocalArticleContentRender as ArticleContentRender;

// article 仓储
pub use article_repository::ArticleRepository;

// category 简易仓储
pub use category_repository::CategoryRepository;
