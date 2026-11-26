use loco_rs::{app::AppContext, Result};
use serde::Deserialize;

use crate::settings::CoreSettings;

/// The `settings.iam` block: where the IAM service is.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct IamSettings {
    /// Address of the IAM service's gRPC server, e.g.
    /// `http://iam:5001`.
    #[serde(default)]
    pub url: String,
}

impl IamSettings {
    /// # Errors
    ///
    /// When the `settings.iam` block is malformed.
    pub fn load(ctx: &AppContext) -> Result<Self> {
        CoreSettings::section(ctx, "iam")
    }
}
