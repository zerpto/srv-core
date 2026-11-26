use std::sync::Arc;

use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};

use crate::http::authorizer::{Authorizer, SharedAuthorizer};

/// Registers the [`Authorizer`] that `CurrentSession::require_any` asks. The
/// service passes the function that builds it, e.g. with the `iam`
/// feature: `Box::new(Authorization(IamAuthorizer::build))`.
pub struct Authorization(pub fn(&AppContext) -> Result<Arc<dyn Authorizer>>);

#[async_trait]
impl Initializer for Authorization {
    fn name(&self) -> String {
        "authorization".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        SharedAuthorizer::register(ctx, (self.0)(ctx)?);
        Ok(router)
    }
}
