mod error;
mod fetch;

use std::time::Duration;

use tokio::time;

use pubsub::Topic;

use error::Error;
use fetch::{OutboxEvent, OutboxFetcher};
use tracing::instrument;

use crate::{
    domain::articles,
    infra::{
        self,
        policy::{self, ReadmodelUpdatePolicyProjection},
    },
};

type Render = infra::domain::ArticleContentRender;

type ReadmodelUpdatePolicy = infra::policy::ReadmodelUpdatePolicy<Render>;

#[instrument(name = "outbox", skip_all)]
pub async fn init_outbox(render: Render, db: lib_db::Db) {
    EventDispatcher::new(render, db.clone(), OutboxFetcher::new(db.clone(), 10), 3)
        .run()
        .await
}

pub struct EventDispatcher {
    db: lib_db::Db,
    outbox: OutboxFetcher,
    max_retries: i16,
    rm_update_policy: ReadmodelUpdatePolicy,
}

impl EventDispatcher {
    const DURATION: Duration = Duration::from_secs(5);

    pub fn new(render: Render, db: lib_db::Db, fetcher: OutboxFetcher, max_retries: i16) -> Self {
        Self {
            db: db.clone(),
            outbox: fetcher,
            max_retries,
            rm_update_policy: ReadmodelUpdatePolicy::new(db, render),
        }
    }

    pub async fn run(self) {
        let mut interval = time::interval(Self::DURATION);

        tracing::info!("start listening events.");
        loop {
            interval.tick().await;

            let _ = self.process_batch().await;
        }
    }

    async fn process_batch(&self) -> Result<(), Error> {
        let events: Vec<OutboxEvent> = self.outbox.fetch_events().await?;

        if events.is_empty() {
            tracing::debug!("No events to process.");
            return Ok(());
        }

        tracing::info!("received {} events.", events.len());

        let event_ids = events
            .iter()
            .map(|e| e.event_id.to_owned())
            .collect::<Vec<_>>();

        let mut tx = self.db.begin().await?;

        for event in events {
            tracing::debug!("processing event: {}", event.event_id);
            let event_id = event.event_id.clone();
            let retries = event.retries;

            if let Err(e) = self.process_event(event, &mut *tx).await {
                tx.rollback().await?;
                if retries > self.max_retries {
                    self.outbox
                        .mark_as_failed(event_id, e.to_string(), &self.db)
                        .await?;
                } else {
                    self.outbox
                        .mark_for_retry(event_id, retries + 1, &self.db)
                        .await?;
                }
                return Err(e);
            }
        }

        self.outbox.mark_as_processed(&event_ids, &mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn process_event(
        &self,
        event: OutboxEvent,
        executor: impl sqlx::Acquire<'_, Database = sqlx::Postgres>,
    ) -> Result<(), Error> {
        let _span =
            tracing::info_span!("handler", eid = event.event_id, topic = event.topic).entered();

        macro_rules! handle_event {
                (
                    $event:ident => {
                        $($event_type:path => $e:ident $body:block)*
                    }
                ) => {
                    match $event.topic.as_str() {
                        $(
                            <$event_type>::TOPIC => {
                                let $e: $event_type = serde_json::from_value($event.payload)?;
                                $body
                            }
                        )*
                        _ => return Err(Error::UnknownEvent($event.topic)),
                    }
                };
            }

        let event_time = event.occurred_at;
        let mut executor = executor.acquire().await?;

        handle_event! {
            event => {
                articles::events::ArticleDeleted => e {
                    self.rm_update_policy
                        .project(&e, event_time, &mut *executor)
                        .await?;
                    // 删除领域聚合对象
                    policy::DomainAggregateDeletePolicy::project(&e, event_time, &mut *executor).await?;
                }
                articles::events::ArticleCreated => e {
                    self.rm_update_policy
                        .project(&e, event_time, &mut *executor)
                        .await?;
                }
                articles::events::ArticleCategoryChanged => e {
                    self.rm_update_policy
                        .project(&e, event_time, &mut *executor)
                        .await?;
                }
                articles::events::ArticleStateChanged => e {
                    self.rm_update_policy
                        .project(&e, event_time, &mut *executor)
                        .await?;
                }
                articles::events::ArticleContentUpdated => e {
                    self.rm_update_policy
                        .project(&e, event_time, &mut *executor)
                        .await?;
                }
                articles::events::ArticleContentReverted => e {
                    self.rm_update_policy
                        .project(&e, event_time, &mut *executor)
                        .await?;
                }
            }
        };
        Ok(())
    }
}
