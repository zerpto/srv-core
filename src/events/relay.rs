use std::time::Duration;

use async_nats::jetstream::{self, message::PublishMessage};
use loco_rs::{app::AppContext, Error, Result};

use super::outbox;

const BATCH: u64 = 100;

/// Publishes outbox rows to `JetStream` until the process stops.
///
/// A row is marked published only after the broker acknowledged it. If the
/// process dies in between, the row is published again; the event id is sent as
/// `Nats-Msg-Id` so `JetStream` drops the duplicate inside its dedup window, and
/// consumers must be idempotent anyway.
pub struct OutboxRelay {
    ctx: AppContext,
    js: jetstream::Context,
    interval: Duration,
}

impl OutboxRelay {
    #[must_use]
    pub const fn new(ctx: AppContext, js: jetstream::Context, interval: Duration) -> Self {
        Self { ctx, js, interval }
    }

    /// Relays until the process stops, sleeping `interval` when the outbox is
    /// drained.
    pub async fn run(self) {
        loop {
            match self.relay_once().await {
                Ok(published) if published == BATCH => continue,
                Ok(_) => {}
                Err(err) => tracing::warn!(error = %err, "outbox relay failed"),
            }
            tokio::time::sleep(self.interval).await;
        }
    }

    async fn relay_once(&self) -> Result<u64> {
        let pending = outbox::Model::pending(&self.ctx.db, BATCH).await?;
        let mut published = 0;
        for event in pending {
            let payload = bytes::Bytes::from(serde_json::to_vec(&event.payload)?);
            self.js
                .send_publish(
                    event.subject.clone(),
                    PublishMessage::build()
                        .payload(payload)
                        .message_id(&event.event_id),
                )
                .await
                .map_err(|e| Error::string(&format!("publish failed: {e}")))?
                .await
                .map_err(|e| Error::string(&format!("publish not acknowledged: {e}")))?;
            outbox::Model::mark_published(&self.ctx.db, event.id).await?;
            published += 1;
        }
        Ok(published)
    }
}
