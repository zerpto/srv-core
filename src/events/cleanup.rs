use async_trait::async_trait;
use loco_rs::{
    prelude::*,
    task::{Task, TaskInfo, Vars},
};

use super::outbox;

const DEFAULT_DAYS: i64 = 7;

/// `cargo loco task outbox:cleanup days:7` deletes outbox rows that were
/// published more than `days` ago. Unpublished rows are never touched.
/// Scheduled daily in `config/*.yaml`.
pub struct OutboxCleanup;

#[async_trait]
impl Task for OutboxCleanup {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "outbox:cleanup".to_string(),
            detail: "Delete published outbox events older than `days` (default 7).".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let days = vars
            .cli_arg("days")
            .ok()
            .and_then(|days| days.parse::<i64>().ok())
            .filter(|days| *days >= 0)
            .unwrap_or(DEFAULT_DAYS);
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days);
        let deleted = outbox::Model::delete_published_before(&ctx.db, cutoff).await?;
        tracing::info!(deleted, days, "outbox cleanup done");
        Ok(())
    }
}
