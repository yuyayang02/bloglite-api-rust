#[derive(sqlx::FromRow)]
pub struct ArticleVersionRow {
    pub slug: String,
    pub version: String,
    pub prev_version: Option<String>,
    pub title: String,
    pub tags: Vec<String>,
    pub summary: String,
    pub content: String,
}

pub struct ArticleVersionsReadModel;

impl ArticleVersionsReadModel {
    pub async fn get_article_version<'a>(
        executor: impl sqlx::PgExecutor<'a>,
        slug: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<Option<ArticleVersionRow>, lib_db::Error> {
        let query = sqlx::query_as(
            r#"--sql
            select * from article_versions_rm  
                where slug = $1
                and version = $2
            "#,
        )
        .bind(slug.as_ref())
        .bind(version.as_ref());

        let result = query.fetch_optional(executor).await?;

        Ok(result)
    }

    pub async fn get_article_all_versions<'a>(
        executor: impl sqlx::PgExecutor<'a>,
        slug: impl AsRef<str>,
    ) -> Result<Vec<ArticleVersionRow>, lib_db::Error> {
        let query = sqlx::query_as::<_, ArticleVersionRow>(
            r#"--sql
            select * from article_versions_rm 
                where slug = $1
                order by created_at
            "#,
        )
        .bind(slug.as_ref());

        let result = query.fetch_all(executor).await?;
        Ok(result)
    }
}
