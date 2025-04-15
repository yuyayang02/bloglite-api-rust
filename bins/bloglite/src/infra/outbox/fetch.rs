use chrono::{DateTime, Local};

use super::error::Error;

#[derive(Debug, sqlx::FromRow)]
pub struct OutboxEvent {
    pub(super) event_id: String,
    pub(super) topic: String,
    pub(super) payload: serde_json::Value,
    pub(super) occurred_at: DateTime<Local>,
    pub(super) retries: i16,
    // pub(super) last_attempt_at: Option<DateTime<Local>>, // 新增最后尝试时间
}

pub struct OutboxFetcher {
    pub(super) db: lib_db::Db,
    pub(super) batch_size: i32,
}

impl OutboxFetcher {
    pub(super) fn new(db: lib_db::Db, batch_size: i32) -> Self {
        Self { db, batch_size }
    }

    pub(super) async fn fetch_events(&self) -> Result<Vec<OutboxEvent>, Error> {
        let events: Vec<OutboxEvent> = sqlx::query_as(
            r#"--sql
            SELECT event_id::text, topic, payload, occurred_at, retries, last_attempt_at
            FROM outbox
            WHERE processed = false
            ORDER BY occurred_at ASC
            FOR UPDATE SKIP LOCKED
            LIMIT $1
            "#,
        )
        .bind(self.batch_size)
        .fetch_all(&self.db)
        .await?;

        Ok(events)
    }

    pub(super) async fn mark_as_processed(
        &self,
        event_ids: &[String],
        executor: impl sqlx::PgExecutor<'_>,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE outbox SET processed = true, processed_at = $1 WHERE event_id::text = ANY($2)",
        )
        .bind(Local::now())
        .bind(event_ids)
        .execute(executor)
        .await?;
        Ok(())
    }

    pub(super) async fn mark_for_retry(
        &self,
        event_id: impl AsRef<str>,
        retries: i16,
        executor: impl sqlx::PgExecutor<'_>,
    ) -> Result<(), Error> {
        sqlx::query(
            "UPDATE outbox SET retries = $1, last_attempt_at = $2 WHERE event_id::text = $3",
        )
        .bind(retries)
        .bind(Local::now())
        .bind(event_id.as_ref())
        .execute(executor)
        .await?;
        Ok(())
    }

    pub(super) async fn mark_as_failed(
        &self,
        event_id: impl AsRef<str>,
        error: impl AsRef<str>,
        executor: impl sqlx::PgExecutor<'_>,
    ) -> Result<(), Error> {
        let event_id = event_id.as_ref();
        let error = error.as_ref();

        tracing::error!(
            eid = event_id,
            error,
            "The event failed multiple retries and was marked as processed."
        );

        sqlx::query("UPDATE outbox SET processed = true, error = $1 WHERE event_id::text = $2")
            .bind(error)
            .bind(event_id)
            .execute(executor)
            .await?;
        Ok(())
    }
}
