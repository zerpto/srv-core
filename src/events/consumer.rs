use std::sync::Arc;

use async_nats::jetstream::{
    self,
    consumer::{pull, AckPolicy},
    AckKind,
};
use async_trait::async_trait;
use futures_util::StreamExt;
use loco_rs::{app::AppContext, Error, Result};

use super::envelope::EventEnvelope;

/// Handles one event. Delivery is at-least-once, so a handler must be
/// idempotent (use `event.id`). Return `Err` to have the event redelivered.
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, ctx: &AppContext, event: EventEnvelope) -> Result<()>;
}

/// One durable subscription: the owner's stream, a durable consumer name that
/// is unique to this service, and the subjects to receive.
pub struct Subscription {
    /// The publishing service's stream, e.g. `BILLING`.
    pub stream: String,
    /// Durable name; use `<this-service>-<purpose>`. Instances sharing it
    /// share the work, like a Kafka consumer group.
    pub durable: String,
    /// Subject filter, e.g. `zerpto.billing.invoice.paid`.
    pub filter_subject: String,
    pub handler: Arc<dyn EventHandler>,
}

impl Subscription {
    /// Receives events for this subscription until the process stops.
    pub async fn run(self, ctx: AppContext, js: jetstream::Context) {
        if let Err(err) = self.consume(&ctx, &js).await {
            tracing::error!(durable = %self.durable, error = %err, "event consumer stopped");
        }
    }

    async fn consume(&self, ctx: &AppContext, js: &jetstream::Context) -> Result<()> {
        let stream = js
            .get_stream(&self.stream)
            .await
            .map_err(|e| Error::string(&format!("stream `{}`: {e}", self.stream)))?;
        let consumer = stream
            .get_or_create_consumer(
                &self.durable,
                pull::Config {
                    durable_name: Some(self.durable.clone()),
                    filter_subject: self.filter_subject.clone(),
                    ack_policy: AckPolicy::Explicit,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| Error::string(&format!("consumer `{}`: {e}", self.durable)))?;
        let mut messages = consumer
            .messages()
            .await
            .map_err(|e| Error::string(&format!("consumer messages: {e}")))?;

        while let Some(message) = messages.next().await {
            let message = message.map_err(|e| Error::string(&format!("receive failed: {e}")))?;
            let kind = match serde_json::from_slice::<EventEnvelope>(&message.payload) {
                // A malformed event can never succeed; drop it instead of retrying.
                Err(err) => {
                    tracing::error!(error = %err, subject = %message.subject, "bad event payload");
                    AckKind::Term
                }
                Ok(event) => match self.handler.handle(ctx, event).await {
                    Ok(()) => AckKind::Ack,
                    Err(err) => {
                        tracing::warn!(error = %err, subject = %message.subject, "event handler failed");
                        AckKind::Nak(None)
                    }
                },
            };
            if let Err(err) = message.ack_with(kind).await {
                tracing::warn!(error = %err, "event ack failed");
            }
        }
        Ok(())
    }
}
