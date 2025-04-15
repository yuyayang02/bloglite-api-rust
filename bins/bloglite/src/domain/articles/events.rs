#[derive(serde::Deserialize, serde::Serialize)]
#[pubsub::topic("article.created")]
pub struct ArticleCreated {
    pub slug: String,
    pub current_version: String,

    pub category_id: String,
    pub author: String,
    pub state: i16,

    pub title: String,
    pub tags: Vec<String>,
    pub body: String,
    pub rendered_body: String,
    pub summary: String,
    pub rendered_summary: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[pubsub::topic("article.content_updated")]
pub struct ArticleContentUpdated {
    pub slug: String,

    pub parent_version: String,
    pub current_version: String,

    pub title: String,
    pub tags: Vec<String>,
    pub body: String,
    pub rendered_body: String,
    pub summary: String,
    pub rendered_summary: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[pubsub::topic("article.content_reverted")]
pub struct ArticleContentReverted {
    pub slug: String,
    pub prev_version: String,
    pub current_version: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[pubsub::topic("article.category_changed")]
pub struct ArticleCategoryChanged {
    pub slug: String,
    pub old_category_id: String,
    pub new_category_id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[pubsub::topic("article.state_changed")]
pub struct ArticleStateChanged {
    pub slug: String,
    pub state: i16,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[pubsub::topic("article.deleted")]
pub struct ArticleDeleted {
    pub slug: String,
}
