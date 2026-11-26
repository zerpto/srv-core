//! Sending messages (email, `WhatsApp`, push, ...) is the relay
//! service's job. A service only pushes a job onto that service's queue.

pub mod message;
pub mod relay_settings;

pub use message::RelayMessage;
pub use relay_settings::RelaySettings;
