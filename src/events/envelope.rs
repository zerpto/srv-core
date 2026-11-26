use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The message every service publishes and consumes. The body is JSON in
/// `data`; its shape is part of the publishing service's contract, so only add
/// fields to it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique per event. Consumers use it to ignore duplicates, and `JetStream`
    /// uses it to drop duplicate publishes.
    pub id: String,
    /// What happened, e.g. `invoice.paid`.
    #[serde(rename = "type")]
    pub event_type: String,
    pub tenant_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
    pub data: serde_json::Value,
}

impl EventEnvelope {
    #[must_use]
    pub fn new(
        event_type: impl Into<String>,
        tenant_id: Option<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            event_type: event_type.into(),
            tenant_id,
            occurred_at: Utc::now(),
            data,
        }
    }
}
