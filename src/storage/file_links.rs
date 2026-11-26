//! Links to stored files that work without signing in, like presigned URLs:
//! each carries a signed token that names the file and the purpose.

use loco_rs::{auth::jwt::JWT, prelude::*};

use crate::{http::problem::Problem, settings::CoreSettings};

/// How long a download link works.
pub const LINK_TTL_SECONDS: u64 = 900;

/// The signed links of one request's [`AppContext`] (it holds the signing
/// secret and the API's public URL).
#[derive(Clone, Copy)]
pub struct FileLinks<'a> {
    ctx: &'a AppContext,
}

impl<'a> FileLinks<'a> {
    #[must_use]
    pub const fn new(ctx: &'a AppContext) -> Self {
        Self { ctx }
    }

    /// A link to download file `id` through `<route>/{id}/download`, good for
    /// 15 minutes. `purpose` keeps a link for one kind of file from opening
    /// another.
    ///
    /// # Errors
    ///
    /// When the link cannot be signed.
    pub fn download_url(
        &self,
        id: &str,
        purpose: &'static str,
        route: &str,
    ) -> Result<String, Problem> {
        self.signed(id, purpose, route, "download")
    }

    /// A link to play video `id` through `<route>/{id}/playback`, otherwise as
    /// [`FileLinks::download_url`].
    ///
    /// # Errors
    ///
    /// When the link cannot be signed.
    pub fn playback_url(
        &self,
        id: &str,
        purpose: &'static str,
        route: &str,
    ) -> Result<String, Problem> {
        self.signed(id, purpose, route, "playback")
    }

    fn signed(
        self,
        id: &str,
        purpose: &'static str,
        route: &str,
        action: &str,
    ) -> Result<String, Problem> {
        let secret = &self.ctx.config.get_jwt_config()?.secret;
        let token = JWT::new(secret)
            .generate_token(
                LINK_TTL_SECONDS,
                id.to_string(),
                serde_json::Map::from_iter([("purpose".to_string(), purpose.into())]),
            )
            .map_err(|err| Problem::from(ModelError::msg(&err.to_string())))?;
        let base = CoreSettings::load(self.ctx)?
            .api_url
            .unwrap_or_else(|| self.ctx.config.server.full_url());
        Ok(format!(
            "{}{route}/{id}/{action}?token={token}",
            base.trim_end_matches('/'),
        ))
    }

    /// Checks that `token` is a live link for file `id` made for `purpose`.
    ///
    /// # Errors
    ///
    /// `401 invalid_token` when it is not.
    pub fn verify(&self, token: &str, id: &str, purpose: &str) -> Result<(), Problem> {
        let expired =
            || Problem::unauthorized("invalid_token", "This link is not valid or has expired.");
        let secret = &self.ctx.config.get_jwt_config()?.secret;
        let claims = JWT::new(secret)
            .validate(token)
            .map_err(|_| expired())?
            .claims;
        let for_file = claims.pid == id
            && claims.claims.get("purpose").and_then(|p| p.as_str()) == Some(purpose);
        if for_file {
            Ok(())
        } else {
            Err(expired())
        }
    }
}
