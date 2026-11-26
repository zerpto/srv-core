use std::sync::Arc;

use async_trait::async_trait;
use loco_rs::{app::AppContext, Error, Result};
use tonic::transport::Channel;

use super::{
    iam_settings::IamSettings,
    pb::{authorization_client::AuthorizationClient, AuthorizeRequest, Decision},
};
use crate::{
    helpers,
    http::{authorizer::Authorizer, problem::Problem},
};

/// An [`Authorizer`] that asks the IAM service over gRPC.
///
/// Every call goes to the IAM service so role changes apply at once; nothing is
/// cached. Fails closed: if the IAM service cannot answer, the request is
/// refused with `503`.
///
/// Register it with the `Authorization` initializer:
/// `Box::new(Authorization(IamAuthorizer::build))`.
#[derive(Debug, Clone)]
pub struct IamAuthorizer {
    channel: Channel,
}

impl IamAuthorizer {
    /// Connects lazily to `url`.
    ///
    /// # Errors
    ///
    /// When `url` is not a valid endpoint.
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self {
            channel: helpers::channel(url)?,
        })
    }

    /// Builds the authorizer from `settings.iam.url`, for the
    /// `Authorization` initializer.
    ///
    /// # Errors
    ///
    /// When the url is missing or invalid.
    pub fn build(ctx: &AppContext) -> Result<Arc<dyn Authorizer>> {
        let url = IamSettings::load(ctx)?.url;
        if url.trim().is_empty() {
            return Err(Error::string("settings.iam.url is not set"));
        }
        Ok(Arc::new(Self::new(&url)?))
    }
}

#[async_trait]
impl Authorizer for IamAuthorizer {
    /// # Errors
    ///
    /// `403 permission_denied`, `403 plan_required`, `403 account_deactivated`, or
    /// `503 iam_unavailable`.
    async fn authorize(
        &self,
        user_id: &str,
        tenant_id: &str,
        keys: &[&str],
    ) -> Result<(), Problem> {
        let response = AuthorizationClient::new(self.channel.clone())
            .authorize(AuthorizeRequest {
                user_id: user_id.to_string(),
                tenant_id: tenant_id.to_string(),
                permission_keys: keys.iter().map(|key| (*key).to_string()).collect(),
            })
            .await
            .map_err(|status| {
                tracing::error!(%status, "IAM authorize call failed");
                Problem::service_unavailable(
                    "iam_unavailable",
                    "Permissions could not be checked. Try again shortly.",
                )
            })?
            .into_inner();

        match Decision::try_from(response.decision) {
            Ok(Decision::Allowed) => Ok(()),
            Ok(Decision::Denied) => Err(Problem::forbidden(
                "permission_denied",
                "You do not have permission to do this.",
            )),
            Ok(Decision::PlanRequired) => Err(Problem::forbidden(
                "plan_required",
                format!(
                    "{} (`{}`) is a paid feature. Start the free trial or subscribe under Billing to unlock it.",
                    response.feature_name, response.feature_key
                ),
            )),
            Ok(Decision::AccountDeactivated) => Err(Problem::forbidden(
                "account_deactivated",
                "Your access to this company has been deactivated.",
            )),
            _ => Err(Problem::service_unavailable(
                "iam_unavailable",
                "Permissions could not be checked. Try again shortly.",
            )),
        }
    }
}
