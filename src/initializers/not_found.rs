use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};

use crate::http::problem::not_found_fallback;

/// Answers every unmatched route with the JSON `404` error envelope
/// (`{error: {message, request_id}}`).
///
/// Loco's own `fallback` middleware replaces the router fallback, so the
/// service must set `server.middlewares.fallback.enable: false` in its config.
pub struct NotFound;

#[async_trait]
impl Initializer for NotFound {
    fn name(&self) -> String {
        "not-found".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        Ok(router.fallback(not_found_fallback))
    }
}
