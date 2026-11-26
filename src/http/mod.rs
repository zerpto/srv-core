//! Pieces of the HTTP API every service shares: the error type, the response
//! envelope, paging, id encoding and the `CurrentSession` extractor.
//!
//! A service keeps only its own versioned controllers, requests and responses.

pub mod authorizer;
pub mod envelope;
pub mod filters;
pub mod idempotency;
pub mod ids;
pub mod nullable;
pub mod paging;
pub mod problem;
pub mod session;
