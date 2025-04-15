pub mod get_all_categories;
pub mod get_all_tags;
pub mod get_article;
pub mod search_articles;

mod role {
    pub struct Admin;
    pub struct Api;
}

#[derive(serde::Serialize)]
pub struct ArticleMetaResult {
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub author: String,
    pub tags: Vec<String>,
    pub category: CategoryResult,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(serde::Serialize)]
pub struct ArticleWithContentResult {
    #[serde(flatten)]
    pub parent: ArticleMetaResult,
    pub content: String,
    pub version: String,
}

#[derive(serde::Serialize)]
pub struct ArticleForAdminResult {
    #[serde(flatten)]
    pub parent: ArticleMetaResult,
    pub state: i16,
    // pub content: String,
    pub version: String,
}

#[derive(serde::Serialize)]
pub struct ArticleListResult<T: serde::Serialize = ArticleMetaResult> {
    pub count: usize,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
    pub items: Vec<T>,
}

#[derive(serde::Serialize)]
pub struct CategoryResult {
    pub id: String,
    pub name: String,
}

#[derive(serde::Serialize)]
pub struct ItemsResult<Item: serde::Serialize> {
    pub total: usize,
    pub items: Vec<Item>,
}

impl<I, T> From<T> for ItemsResult<I>
where
    I: serde::Serialize,
    T: IntoIterator<Item = I>,
{
    fn from(iter: T) -> Self {
        let items: Vec<I> = iter.into_iter().collect();
        let total = items.len();
        ItemsResult { total, items }
    }
}
