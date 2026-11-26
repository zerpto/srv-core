use std::time::Duration;

use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};

use crate::{
    events::{consumer::Subscription, jetstream::JetStream, relay::OutboxRelay},
    settings::CoreSettings,
};

/// Connects to NATS `JetStream`, starts the outbox relay and one consumer per
/// subscription. Does nothing unless `settings.events.enabled` is true.
///
/// The service passes the function that lists what it consumes:
/// `Box::new(Events(my_service::events::subscriptions::subscriptions))`.
pub struct Events(pub fn() -> Vec<Subscription>);

#[async_trait]
impl Initializer for Events {
    fn name(&self) -> String {
        "events".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let settings = CoreSettings::load(ctx)?;
        if !settings.events.enabled {
            return Ok(());
        }
        let js = JetStream::new(&settings).connect().await?;

        tokio::spawn(
            OutboxRelay::new(
                ctx.clone(),
                js.clone(),
                Duration::from_millis(settings.events.relay_interval_ms),
            )
            .run(),
        );
        for subscription in (self.0)() {
            tokio::spawn(subscription.run(ctx.clone(), js.clone()));
        }
        Ok(())
    }
}
