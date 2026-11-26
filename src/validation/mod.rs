//! Declarative request validation.
//!
//! Rules go on the fields of a request struct
//! with the `validator` crate, [`ValidatedJson`] checks them before the handler
//! runs, and every failure is the shared `422` error with `details`
//! ([`ValidationDetails`] builds them). The shared [`rules`] set an explicit,
//! stable code and message; [`failure`] is how a rule builds its error.

mod failure;
pub mod rules;
mod validated_json;
mod validation_details;

pub use failure::failure;
pub use validated_json::ValidatedJson;
pub use validation_details::ValidationDetails;
