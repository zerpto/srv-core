use std::sync::Arc;

use async_trait::async_trait;
use loco_rs::app::AppContext;

use super::problem::Problem;

/// Decides whether a user may do something in a tenant.
///
/// The core does not know who answers: a service registers one (the IAM client
/// from the `iam` feature, or its own) with the `Authorization` initializer, and
/// `CurrentSession::require_any` asks it.
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Allowed when the user holds ANY of `keys` in `tenant_id`.
    ///
    /// # Errors
    ///
    /// A `Problem` the caller returns as is, e.g. `403 permission_denied` or
    /// `503` when the answer cannot be had. Fail closed.
    async fn authorize(&self, user_id: &str, tenant_id: &str, keys: &[&str])
        -> Result<(), Problem>;
}

/// The registered [`Authorizer`], kept in `ctx.shared_store`.
#[derive(Clone)]
pub struct SharedAuthorizer(pub Arc<dyn Authorizer>);

impl SharedAuthorizer {
    /// Registers `authorizer` for the whole app.
    pub fn register(ctx: &AppContext, authorizer: Arc<dyn Authorizer>) {
        ctx.shared_store.insert(Self(authorizer));
    }

    /// The registered authorizer, if any.
    #[must_use]
    pub fn get(ctx: &AppContext) -> Option<Arc<dyn Authorizer>> {
        ctx.shared_store.get::<Self>().map(|shared| shared.0)
    }
}
