use loco_rs::{
    app::AppContext,
    bgworker::{redis, Queue},
    config::RedisQueueConfig,
    Result,
};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use super::relay_settings::RelaySettings;

static QUEUE: OnceCell<Queue> = OnceCell::const_new();

/// The job argument the relay service's worker takes. Its shape is that
/// service's contract: keep it identical to the worker's args and only add
/// optional fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayMessage {
    /// How to deliver it: `email`, `whatsapp`, `sms`, `push`, ...
    pub channel: String,
    /// Where to: an address, phone number or user id, depending on `channel`.
    pub recipient: String,
    /// The message template in the relay service, e.g. `rfq.invited`.
    pub template: String,
    pub tenant_id: Option<String>,
    /// Values the template renders.
    pub data: serde_json::Value,
}

impl RelayMessage {
    /// Queues this message for the relay service. Returns as soon
    /// as the job is in its Redis queue; delivery, retries and templates are
    /// that service's. A no-op unless `settings.relay.enabled` is true.
    ///
    /// Call it after your own transaction has committed, never inside one, so
    /// a rolled-back change does not send a message.
    ///
    /// # Errors
    ///
    /// When the relay queue cannot be reached.
    pub async fn send(&self, ctx: &AppContext) -> Result<()> {
        let settings = RelaySettings::load(ctx)?;
        if !settings.enabled {
            // Relay is off (the relay service may not exist
            // yet): show the full payload so it can be checked. It can hold
            // links with tokens, so this is only logged while disabled.
            tracing::info!(
                channel = %self.channel,
                recipient = %self.recipient,
                template = %self.template,
                tenant_id = ?self.tenant_id,
                data = %self.data,
                "relay message not sent (relay disabled)"
            );
            return Ok(());
        }
        tracing::debug!(
            channel = %self.channel,
            template = %self.template,
            queue = %settings.queue,
            "queueing relay message"
        );
        let queue = QUEUE
            .get_or_try_init(|| async {
                redis::create_provider(&RedisQueueConfig {
                    uri: settings.redis_url.clone(),
                    dangerously_flush: false,
                    queues: None,
                    num_workers: 1,
                    reaper: None,
                })
                .await
            })
            .await?;
        queue
            .enqueue(
                settings.worker_class,
                Some(settings.queue),
                self,
                None,
                None,
            )
            .await?;
        Ok(())
    }
}
