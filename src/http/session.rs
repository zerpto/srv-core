use axum::{extract::FromRequestParts, http::request::Parts};
use loco_rs::{app::AppContext, auth::jwt};

use super::{authorizer::SharedAuthorizer, problem::Problem};

/// The caller, read from the `Authorization: Bearer` access token.
///
/// Stateless: extracting it checks only the signature and expiry against the
/// shared `auth.jwt.secret`. It touches neither the database nor the cache, and
/// no service but the IAM service has a users table. Take it as a handler
/// argument to require a signed-in caller:
///
/// ```ignore
/// async fn list(session: CurrentSession, State(ctx): State<AppContext>) -> ApiResult {
///     let tenant_id = session.require_tenant()?;
///     ...
/// }
/// ```
///
/// What the caller may do (roles, permissions, plan limits) is not in the
/// token. Ask the registered [`Authorizer`] (`require_any`) when a handler
/// needs it.
#[derive(Debug)]
pub struct CurrentSession {
    user_id: String,
    tenant_id: Option<String>,
}

fn unauthorized() -> Problem {
    Problem::unauthorized("invalid_token", "Missing, invalid or expired access token.")
}

impl FromRequestParts<AppContext> for CurrentSession {
    type Rejection = Problem;

    async fn from_request_parts(
        parts: &mut Parts,
        ctx: &AppContext,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(unauthorized)?;

        let secret = &ctx.config.get_jwt_config()?.secret;
        let data = jwt::JWT::new(secret)
            .validate(token)
            .map_err(|_| unauthorized())?;

        let tenant_id = data
            .claims
            .claims
            .get("tenant_id")
            .and_then(|value| value.as_str())
            .map(str::to_string);
        Ok(Self {
            user_id: data.claims.pid,
            tenant_id,
        })
    }
}

impl CurrentSession {
    /// The API id of the user (the token's `pid`).
    #[must_use]
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// The tenant the token is scoped to; `None` before onboarding.
    #[must_use]
    pub fn tenant_id(&self) -> Option<&str> {
        self.tenant_id.as_deref()
    }

    /// Requires that the caller may do something that needs ANY of `keys`
    /// (e.g. `&["expenses.read", "expenses.manage"]`) in their tenant. Asks
    /// the [`Authorizer`](super::authorizer::Authorizer) registered with the
    /// `Authorization` initializer, on every call.
    ///
    /// # Errors
    ///
    /// `403 tenant_required`, whatever the authorizer refuses with (e.g.
    /// `403 permission_denied`), or `503 authorizer_unavailable` when none is
    /// registered.
    pub async fn require_any(&self, ctx: &AppContext, keys: &[&str]) -> Result<(), Problem> {
        let tenant_id = self.require_tenant()?;
        let Some(authorizer) = SharedAuthorizer::get(ctx) else {
            tracing::error!("no Authorizer registered; add the `Authorization` initializer");
            return Err(Problem::service_unavailable(
                "authorizer_unavailable",
                "Permissions could not be checked. Try again shortly.",
            ));
        };
        authorizer.authorize(&self.user_id, tenant_id, keys).await
    }

    /// The tenant, for handlers whose data is per tenant. Scope every query
    /// by it.
    ///
    /// # Errors
    ///
    /// `403 tenant_required` when the token has no tenant.
    pub fn require_tenant(&self) -> Result<&str, Problem> {
        self.tenant_id().ok_or_else(|| {
            Problem::forbidden(
                "tenant_required",
                "This action needs a company; finish onboarding first.",
            )
        })
    }
}
