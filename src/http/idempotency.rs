//! `Idempotency-Key`: a retry with the same key returns the original result
//! instead of doing the work twice. Keys are kept for 24 hours.
//!
//! The cache remembers a small reference to the result (the id of the record
//! that was created), never a response body, so no token or secret is cached.

use std::time::Duration;

use axum::http::HeaderMap;
use loco_rs::prelude::*;

use crate::http::problem::Problem;

const HEADER: &str = "idempotency-key";
const MAX_LENGTH: usize = 255;
const TTL: Duration = Duration::from_hours(24);

/// The cache key of one request: scoped by operation and caller, so the same
/// `Idempotency-Key` sent to another endpoint or by another user is a
/// different request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// The key of a request, or `None` when the client sent no
    /// `Idempotency-Key`.
    ///
    /// # Errors
    ///
    /// `400` when the header is empty or longer than 255 characters.
    pub fn from_headers(
        headers: &HeaderMap,
        operation: &str,
        caller: &str,
    ) -> Result<Option<Self>, Problem> {
        let Some(value) = headers.get(HEADER) else {
            return Ok(None);
        };
        match value.to_str() {
            Ok(key) if !key.is_empty() && key.len() <= MAX_LENGTH => {
                Ok(Some(Self(format!("idem:{operation}:{caller}:{key}"))))
            }
            _ => Err(Problem::bad_request(
                "invalid_idempotency_key",
                format!("`Idempotency-Key` must be 1 to {MAX_LENGTH} characters."),
            )),
        }
    }

    /// The scoped key as stored in the cache.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// What the first request left behind for this key, if anything.
    ///
    /// # Errors
    ///
    /// When the cache is unreachable.
    pub async fn recall(&self, ctx: &AppContext) -> Result<Option<String>, Problem> {
        Ok(ctx
            .cache
            .get::<String>(&self.0)
            .await
            .map_err(Error::wrap)?)
    }

    /// Remembers the result of a request that succeeded.
    ///
    /// # Errors
    ///
    /// When the cache is unreachable.
    pub async fn remember(&self, ctx: &AppContext, reference: &str) -> Result<(), Problem> {
        ctx.cache
            .insert_with_expiry(&self.0, &reference.to_string(), TTL)
            .await
            .map_err(Error::wrap)?;
        Ok(())
    }
}
