use loco_rs::{app::AppContext, Result};
use serde::Deserialize;

use crate::settings::CoreSettings;

/// The `settings.relay` block. The relay service takes messages
/// from its own Redis queue; this service pushes jobs onto that queue
/// (`RelayMessage::send`).
#[derive(Debug, Clone, Deserialize)]
pub struct RelaySettings {
    /// Off until the service sends messages; `RelayMessage::send` then does nothing.
    #[serde(default)]
    pub enabled: bool,
    /// Redis of the relay service (not this service's own Redis).
    #[serde(default)]
    pub redis_url: String,
    /// Queue name the relay service reads.
    #[serde(default = "default_queue")]
    pub queue: String,
    /// Class name of the relay service's worker.
    #[serde(default = "default_worker")]
    pub worker_class: String,
}

impl Default for RelaySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            redis_url: String::new(),
            queue: default_queue(),
            worker_class: default_worker(),
        }
    }
}

impl RelaySettings {
    /// # Errors
    ///
    /// When the `settings.relay` block is malformed.
    pub fn load(ctx: &AppContext) -> Result<Self> {
        CoreSettings::section(ctx, "relay")
    }
}

fn default_queue() -> String {
    "relay".to_string()
}

fn default_worker() -> String {
    "RelayWorker".to_string()
}
