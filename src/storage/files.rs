use std::path::PathBuf;

use axum::body::Bytes;
use loco_rs::{prelude::*, storage::Storage};

/// The stored files of the service: writing, reading and removing the bytes
/// in one [`Storage`] backend (`ctx.storage`).
#[derive(Clone, Copy)]
pub struct Files<'a> {
    storage: &'a Storage,
}

impl<'a> Files<'a> {
    #[must_use]
    pub const fn new(storage: &'a Storage) -> Self {
        Self { storage }
    }

    /// Writes the file under `folder/<tenant_id>/<id>.<extension>` and returns
    /// the path it was stored at. `id` is the file record's `UUIDv7`;
    /// `tenant_id` is the tenant's plain UUID. Keeping tenants in separate
    /// folders makes a cross tenant read impossible to build by accident.
    ///
    /// # Errors
    ///
    /// When the file cannot be written.
    pub async fn store(
        &self,
        folder: &str,
        tenant_id: &Uuid,
        id: &Uuid,
        extension: &str,
        bytes: &Bytes,
    ) -> ModelResult<String> {
        let path = format!("{folder}/{tenant_id}/{id}.{extension}");
        self.storage
            .upload(&PathBuf::from(&path), bytes)
            .await
            .map_err(|err| ModelError::msg(&err.to_string()))?;
        Ok(path)
    }

    /// The stored file's content.
    ///
    /// # Errors
    ///
    /// When the file cannot be read.
    pub async fn read(&self, path: &str) -> ModelResult<Bytes> {
        self.storage
            .download::<Vec<u8>>(&PathBuf::from(path))
            .await
            .map(Bytes::from)
            .map_err(|err| ModelError::msg(&err.to_string()))
    }

    /// Removes a file that is no longer used. A file that is already gone is
    /// not a failure.
    pub async fn remove(&self, path: &str) {
        let _ = self.storage.delete(&PathBuf::from(path)).await;
    }
}
