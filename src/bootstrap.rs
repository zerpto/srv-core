use loco_rs::{app::AppContext, environment::Environment, Result};

use crate::initializers::query_log::QueryReport;

/// What every service does in `Hooks::after_context`: builds `ctx.storage`
/// from `settings.storage` (feature `storage`) and, in development, starts the
/// per-request SQL report.
///
/// ```ignore
/// async fn after_context(ctx: AppContext) -> Result<AppContext> {
///     zerpto_core::bootstrap::after_context(ctx).await
/// }
/// ```
///
/// # Errors
///
/// When the settings are malformed or the storage backend cannot be built.
#[allow(clippy::unused_async)]
pub async fn after_context(ctx: AppContext) -> Result<AppContext> {
    let mut ctx = ctx;
    if ctx.environment == Environment::Development {
        // Feeds the per-request SQL report (`meta.debug`).
        ctx.db.set_metric_callback(QueryReport::record);
    }
    #[cfg(feature = "storage")]
    {
        let storage = crate::storage::driver::StorageDriver::new(
            crate::settings::CoreSettings::load(&ctx)?.storage,
        )
        .build()?;
        ctx = ctx
            .into_builder()
            .storage(std::sync::Arc::new(storage))
            .build();
    }
    Ok(ctx)
}
