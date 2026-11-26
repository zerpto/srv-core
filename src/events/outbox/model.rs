use async_trait::async_trait;
use sea_orm::{
    entity::prelude::*, ActiveValue, ConnectionTrait, DbErr, QueryFilter, QueryOrder, QuerySelect,
};

use super::entity::{ActiveModel, Column, Entity, Model};
use crate::events::envelope::EventEnvelope;

/// The pattern for every model in this service: `id` is the only primary key,
/// a `UUIDv7` generated here on insert (time-ordered, so it indexes well and
/// sorts by creation). No serial integers, no second public id.
#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert && self.id.is_not_set() {
            self.id = ActiveValue::Set(Uuid::now_v7());
        }
        Ok(self)
    }
}

impl Model {
    /// Records `event` to be published on `subject`. Pass the **same
    /// transaction** that holds the change the event describes, so both
    /// commit or neither does. The relay publishes it afterwards.
    ///
    /// `subject` follows `zerpto.<service>.<entity>.<event>` and must sit
    /// under this service's prefix (`CoreSettings::subject_prefix`).
    ///
    /// # Errors
    ///
    /// On a database error.
    pub async fn enqueue<C: ConnectionTrait>(
        db: &C,
        subject: &str,
        event: &EventEnvelope,
    ) -> Result<Self, DbErr> {
        let payload = serde_json::to_value(event).map_err(|e| DbErr::Custom(e.to_string()))?;
        ActiveModel {
            event_id: ActiveValue::Set(event.id.clone()),
            subject: ActiveValue::Set(subject.to_string()),
            payload: ActiveValue::Set(payload),
            ..Default::default()
        }
        .insert(db)
        .await
    }

    /// The oldest events not yet published, in creation order. `UUIDv7` ids
    /// sort by time, so `id` breaks ties between rows created in the same
    /// instant.
    ///
    /// # Errors
    ///
    /// On a database error.
    pub async fn pending<C: ConnectionTrait>(db: &C, limit: u64) -> Result<Vec<Self>, DbErr> {
        Entity::find()
            .filter(Column::PublishedAt.is_null())
            .order_by_asc(Column::CreatedAt)
            .order_by_asc(Column::Id)
            .limit(limit)
            .all(db)
            .await
    }

    /// Deletes events published before `cutoff`; returns how many.
    ///
    /// # Errors
    ///
    /// On a database error.
    pub async fn delete_published_before<C: ConnectionTrait>(
        db: &C,
        cutoff: chrono::DateTime<chrono::Utc>,
    ) -> Result<u64, DbErr> {
        let result = Entity::delete_many()
            .filter(Column::PublishedAt.lt(cutoff))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    /// # Errors
    ///
    /// On a database error.
    pub async fn mark_published<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<(), DbErr> {
        ActiveModel {
            id: ActiveValue::Unchanged(id),
            published_at: ActiveValue::Set(Some(chrono::Utc::now().into())),
            ..Default::default()
        }
        .update(db)
        .await?;
        Ok(())
    }
}
