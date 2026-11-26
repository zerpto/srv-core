use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// The transactional outbox: events are written here in the same transaction
/// as the change that caused them, then relayed to the broker.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        create_table(
            m,
            "outbox_events",
            &[
                // `id` is a UUIDv7 made by the app (no database default, no serial).
                ("id", ColType::PkUuid),
                ("event_id", ColType::StringUniq),
                ("subject", ColType::String),
                ("payload", ColType::JsonBinary),
                ("published_at", ColType::TimestampWithTimeZoneNull),
            ],
            &[],
        )
        .await
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        drop_table(m, "outbox_events").await
    }
}
