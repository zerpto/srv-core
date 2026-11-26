use async_trait::async_trait;
use axum::{middleware::from_fn, Router as AxumRouter};
use loco_rs::{
    app::{AppContext, Initializer},
    environment::Environment,
    Result,
};

use super::QueryReport;

/// Wires [`QueryReport::collect`] into the router, in development only.
pub struct QueryLog;

#[async_trait]
impl Initializer for QueryLog {
    fn name(&self) -> String {
        "query_log".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        if ctx.environment == Environment::Development {
            Ok(router.layer(from_fn(QueryReport::collect)))
        } else {
            Ok(router)
        }
    }
}
