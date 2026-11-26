use async_nats::jetstream::{self, stream};
use loco_rs::{Error, Result};

use crate::settings::CoreSettings;

/// The NATS `JetStream` connection of this service, from its settings.
#[derive(Debug, Clone, Copy)]
pub struct JetStream<'a> {
    settings: &'a CoreSettings,
}

impl<'a> JetStream<'a> {
    #[must_use]
    pub const fn new(settings: &'a CoreSettings) -> Self {
        Self { settings }
    }

    /// Connects to NATS and makes sure this service's stream exists. The
    /// stream captures `zerpto.<service>.>`; peers consume from it.
    ///
    /// # Errors
    ///
    /// When the broker is unreachable or the stream cannot be created.
    pub async fn connect(&self) -> Result<jetstream::Context> {
        let client = async_nats::connect(&self.settings.events.nats_url)
            .await
            .map_err(|e| Error::string(&format!("NATS connect failed: {e}")))?;
        let js = jetstream::new(client);
        js.get_or_create_stream(stream::Config {
            name: self.settings.stream_name(),
            subjects: vec![format!("{}.>", self.settings.subject_prefix())],
            ..Default::default()
        })
        .await
        .map_err(|e| Error::string(&format!("JetStream stream setup failed: {e}")))?;
        Ok(js)
    }
}
