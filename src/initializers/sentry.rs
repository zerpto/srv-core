use std::sync::OnceLock;

use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};

use crate::settings::CoreSettings;

/// Keeps the client alive for the life of the process; dropping it would stop
/// reporting.
static GUARD: OnceLock<sentry::ClientInitGuard> = OnceLock::new();

/// Starts Sentry when `settings.sentry.dsn` is set.
///
/// Panics are reported automatically; `/v1` unexpected errors are reported by `Problem` (see
/// `controllers/problem.rs`) with the request id as a tag. Personal data is
/// not attached (`send_default_pii` stays off).
pub struct Sentry;

#[async_trait]
impl Initializer for Sentry {
    fn name(&self) -> String {
        "sentry".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let settings = CoreSettings::load(ctx)?;
        let Some(dsn) = settings.sentry.dsn.filter(|dsn| !dsn.trim().is_empty()) else {
            return Ok(());
        };
        // `ClientOptions` is non-exhaustive: build it with its setters.
        let options = sentry::ClientOptions::new()
            .release(format!(
                "{}@{}",
                settings.service,
                option_env!("BUILD_SHA")
                    .or(option_env!("GITHUB_SHA"))
                    .unwrap_or(env!("CARGO_PKG_VERSION"))
            ))
            .environment(format!("{:?}", ctx.environment).to_lowercase())
            // The setter panics outside 0.0..=1.0, so keep a typo from crashing boot.
            .traces_sample_rate(settings.sentry.traces_sample_rate.clamp(0.0, 1.0));
        let guard = sentry::init((dsn, options));
        let _ = GUARD.set(guard);
        Ok(())
    }
}
