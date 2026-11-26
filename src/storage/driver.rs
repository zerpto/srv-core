use loco_rs::{
    storage::{
        drivers::{aws, local},
        Storage,
    },
    Error, Result,
};

use crate::settings::StorageSettings;

/// Builds the storage backend from `settings.storage`: local disk for
/// `driver: local`, otherwise Cloudflare R2. Called from `App::after_context`;
/// handlers read it as `ctx.storage`.
pub struct StorageDriver {
    settings: StorageSettings,
}

impl StorageDriver {
    #[must_use]
    pub const fn new(settings: StorageSettings) -> Self {
        Self { settings }
    }

    /// # Errors
    ///
    /// When an R2 setting is missing or the backend cannot be built.
    pub fn build(self) -> Result<Storage> {
        let settings = self.settings;
        let driver = if settings.driver == "local" {
            local::new_with_prefix(settings.dir)?
        } else {
            let require = |value: Option<String>, field: &str| {
                value
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| Error::string(&format!("settings.storage.{field} is required")))
            };
            aws::with_credentials_and_endpoint(
                &require(settings.r2_bucket, "r2_bucket")?,
                "auto",
                &require(settings.r2_endpoint, "r2_endpoint")?,
                aws::Credential {
                    key_id: require(settings.r2_access_key_id, "r2_access_key_id")?,
                    secret_key: require(settings.r2_secret_access_key, "r2_secret_access_key")?,
                    token: None,
                },
            )?
        };
        Ok(Storage::single(driver))
    }
}
