//! The transactional outbox: events are written to `outbox_events` in the same
//! transaction as the change that caused them, then relayed to the broker.
//!
//! Add [`Migration`] to the service's `Migrator`, before the service's own
//! migrations: `Box::new(zerpto_core::events::outbox::Migration)`.

pub mod entity;
// A released migration is never edited, so its lints are allowed here.
#[allow(clippy::wildcard_imports, elided_lifetimes_in_paths)]
mod m20261121_000001_outbox_events;
mod model;

pub use entity::{ActiveModel, Column, Entity, Model};
pub use m20261121_000001_outbox_events::Migration;
