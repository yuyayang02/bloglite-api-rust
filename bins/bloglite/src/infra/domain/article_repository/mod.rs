pub mod model;

use crate::domain::articles::{self, Article};
use lib_db::Result;

pub struct ArticleRepository {
    db: lib_db::Db,
}

impl ArticleRepository {
    pub fn new(db: lib_db::Db) -> Self {
        Self { db }
    }
}

impl articles::repository::ArticleRepository for ArticleRepository {
    type Error = lib_db::Error;
    async fn find(&self, id: &articles::ArticleId) -> Result<Option<Article>> {
        let result = sqlx::query_as::<_, model::ArticleRow>(
            r#"--sql
            select * from articles where id = ($1) AND state != -1
            "#,
        )
        .bind(id.as_ref())
        .fetch_optional(&self.db)
        .await?;

        result.map(|a| articles::Article::try_from(a)).transpose()
    }

    async fn find_by_slug(
        &self,
        slug: &articles::ArticleSlug,
    ) -> std::result::Result<Option<Article>, Self::Error> {
        let result = sqlx::query_as::<_, model::ArticleRow>(
            r#"--sql
            select * from articles where slug = ($1) AND state != -1
            "#,
        )
        .bind(slug.as_ref())
        .fetch_optional(&self.db)
        .await?;

        result.map(|a| articles::Article::try_from(a)).transpose()
    }

    // async fn save(&self, article: Article) -> Result<()> {
    //     save_article(&self.db, model::ArticleRow::from(article)).await
    // }
    async fn save_all<I>(&self, article: Article, events: I) -> Result<()>
    where
        I: IntoIterator<Item = articles::repository::Event> + Send,
        I::IntoIter: Send,
    {
        let mut tx = self.db.begin().await?;

        if let Err(e) = save_article(tx.as_mut(), article.into()).await {
            tx.rollback().await?;
            return Err(e.into());
        }

        for event in events {
            if let Err(e) = save_event(tx.as_mut(), event).await {
                tx.rollback().await?;
                return Err(e.into());
            }
        }

        tx.commit().await?;
        Ok(())
    }
}

async fn save_article<'a>(
    executor: impl sqlx::PgExecutor<'a>,
    row: model::ArticleRow,
) -> Result<()> {
    sqlx::query(
        r#"--sql
        INSERT INTO articles (id, slug, category, state, version_history)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO UPDATE
        SET slug = $2, category = $3, state = $4, version_history = $5
        "#,
    )
    .bind(row.id)
    .bind(row.slug)
    .bind(row.category)
    .bind(row.state)
    .bind(row.version_history)
    .execute(executor)
    .await?;
    Ok(())
}

async fn save_event<'a>(
    executor: impl sqlx::PgExecutor<'a>,
    event: articles::repository::Event,
) -> Result<()> {
    let topic = event.topic();
    let msg = event.message();

    sqlx::query(
        r#"--sql
        INSERT INTO outbox
            (event_id, topic, payload, occurred_at)
        VALUES 
            ($1::uuid, $2, $3::json, $4)
        "#,
    )
    .bind(msg.id())
    .bind(topic)
    .bind(msg.payload_as::<serde_json::Value>())
    .bind(msg.time())
    .execute(executor)
    .await?;

    Ok(())
}
