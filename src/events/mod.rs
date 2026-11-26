//! Event streaming over NATS `JetStream`.
//!
//! Publishing is transactional: code writes an event to the `outbox_events`
//! table in the same database transaction as its own change
//! (`outbox::Model::enqueue`), and [`relay::OutboxRelay`] publishes it to `JetStream`
//! afterwards. Consuming is at-least-once: implement
//! [`consumer::EventHandler`], make it idempotent, and pass a
//! [`consumer::Subscription`] list to the `Events` initializer.
//!
//! Subject convention: `zerpto.<service>.<entity>.<event>`, e.g.
//! `zerpto.billing.invoice.paid`. Each service owns the stream named after it.

pub mod cleanup;
pub mod consumer;
pub mod envelope;
pub mod jetstream;
pub mod outbox;
pub mod relay;
