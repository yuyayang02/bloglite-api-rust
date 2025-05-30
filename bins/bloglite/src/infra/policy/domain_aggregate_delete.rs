use chrono::{DateTime, Local};

use crate::domain::articles::events;

use super::Error;

pub struct DomainAggregateDeletePolicy;

impl DomainAggregateDeletePolicy {
    pub async fn project<'a, C: sqlx::PgExecutor<'a>>(
        event: &events::ArticleDeleted,
        _: DateTime<Local>,
        executor: C,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"--sql 
            DELETE FROM articles WHERE id = $1
            "#,
        )
        .bind(&event.id)
        .execute(executor)
        .await?;

        Ok(())
    }
}
