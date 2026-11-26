//! Zerpto core: what every Zerpto service shares, so each service holds only
//! its own domain code. Services depend on this crate by git tag and update by
//! bumping the tag.
//!
//! Features: `events` (NATS `JetStream`, outbox), `storage` (files, R2),
//! `iam` (client of the IAM service), `relay` (queue jobs to the
//! relay service). The HTTP, validation, gRPC health, settings and
//! Sentry parts are always on. The core names no other service: each feature
//! client owns its own settings block and contract.
//!
//! This crate pins the Loco version; a service must use the same `loco-rs`
//! minor version, and both are upgraded together.

pub use loco_rs;
pub use tonic;

pub mod bootstrap;
#[cfg(feature = "events")]
pub mod events;
pub mod grpc;
pub mod helpers;
pub mod http;
#[cfg(feature = "iam")]
pub mod iam;
pub mod initializers;
#[cfg(feature = "relay")]
pub mod relay;
pub mod settings;
#[cfg(feature = "storage")]
pub mod storage;
pub mod validation;
