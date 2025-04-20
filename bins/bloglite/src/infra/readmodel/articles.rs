use chrono::{DateTime, Local};
use sqlx::QueryBuilder;

#[derive(Debug, sqlx::FromRow)]
pub struct ArticleRow {
    pub id: String,
    pub slug: String,
    pub category_id: String,
    pub category_name: String,
    pub author: String,
    pub state: i16,
    pub current_version: String,
    pub title: String,
    pub tags: Vec<String>,
    pub rendered_summary: String,
    pub rendered_content: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

pub struct TagsQuery;

impl TagsQuery {
    pub async fn get_tags(
        executor: impl sqlx::PgExecutor<'_>,
        include_private_article: bool,
    ) -> Result<Vec<String>, lib_db::Error> {
        let query = if include_private_article {
            sqlx::query_scalar::<_, String>(
                r#"--sql
                select DISTINCT unnest(tags) AS tag
                from articles_rm
                where state = 1
                order by tag;
                "#,
            )
        } else {
            sqlx::query_scalar::<_, String>(
                r#"--sql
                select DISTINCT unnest(tags) AS tag
                from articles_rm
                order by tag;
                "#,
            )
        };

        // 执行查询
        Ok(query.fetch_all(executor).await?)
    }
}

pub struct ArticleQueryBuilder<'a> {
    query: QueryBuilder<'a, sqlx::Postgres>,
    has_where: bool,
    executor: &'a lib_db::Db,
}

impl<'a> ArticleQueryBuilder<'a> {
    pub async fn get_one(
        executor: &'a lib_db::Db,
        slug: &'a str,
    ) -> Result<Option<ArticleRow>, lib_db::Error> {
        Ok(sqlx::query_as::<_, ArticleRow>(
            "select * from articles_rm where state = 1 and slug = $1",
        )
        .bind(slug)
        .fetch_optional(executor)
        .await?)
    }

    pub async fn get_with_filter(
        executor: &'a lib_db::Db,
        page: i32,
        limit: i32,
        category: Option<String>,
        tags: &'a [String],
        author: Option<String>,
        include_private_article: bool,
    ) -> Result<(Vec<ArticleRow>, i64), lib_db::Error> {
        let mut builder = Self::new(executor);

        if !include_private_article {
            builder = builder.with_state(1);
        }

        if let Some(c) = category {
            builder = builder.with_category(c);
        }

        if let Some(a) = author {
            builder = builder.with_author(a);
        }

        if !tags.is_empty() {
            builder = builder.with_tags(tags);
        }

        builder
            .order_by("updated_at", false)
            .search(page, limit)
            .await
    }

    pub fn new(executor: &'a lib_db::Db) -> Self {
        Self {
            query: QueryBuilder::new("SELECT *, COUNT(*) OVER() AS total_count FROM articles_rm"),
            has_where: false,
            executor,
        }
    }

    fn add_where(&mut self, condition: &str) {
        if !self.has_where {
            self.query.push(" WHERE ");
            self.has_where = true;
        } else {
            self.query.push(" AND ");
        }
        self.query.push(condition);
    }

    // pub fn with_slug(mut self, slug: &'a str) -> Self {
    //     self.add_where("slug = ");
    //     self.query.push_bind(slug);
    //     self
    // }

    pub fn with_state(mut self, state: i16) -> Self {
        self.add_where("state = ");
        self.query.push_bind(state);
        self
    }

    pub fn with_category(mut self, category: String) -> Self {
        self.add_where("category_id = ");
        self.query.push_bind(category);
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.add_where("author = ");
        self.query.push_bind(author);
        self
    }

    pub fn with_tags(mut self, tags: &'a [String]) -> Self {
        self.add_where("tags @> ");
        self.query.push_bind(tags);
        self
    }

    pub fn order_by(mut self, order_by: &'static str, asc: bool) -> Self {
        self.query
            .push(" ORDER BY ")
            .push(order_by)
            .push(if asc { " ASC" } else { " DESC" });
        self
    }

    pub async fn search(
        mut self,
        page: i32,
        limit: i32,
    ) -> Result<(Vec<ArticleRow>, i64), lib_db::Error> {
        self.query.push(" LIMIT ");
        self.query.push_bind(limit);
        self.query.push(" OFFSET ");
        self.query
            .push_bind(lib_utils::pagination::offset(limit, page));

        #[derive(sqlx::FromRow)]
        struct ArticleWithCount {
            #[sqlx(flatten)]
            items: ArticleRow,
            total_count: i64,
        }

        let results = self
            .query
            .build_query_as::<ArticleWithCount>()
            .fetch_all(self.executor)
            .await?;

        let total = results.first().map(|r| r.total_count).unwrap_or(0);
        let articles = results.into_iter().map(|r| r.items).collect();

        Ok((articles, total))
    }
}

#[cfg(test)]
mod tests {
    use crate::_dev_utils;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test() {
        let db = _dev_utils::init_db().await;

        let builder = ArticleQueryBuilder::new(&db);

        let (a, i) = builder.with_state(1).search(1, 2).await.unwrap();
        println!("{} {}", a.len(), i);
    }
}
