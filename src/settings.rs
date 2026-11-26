use loco_rs::prelude::*;
use serde::Deserialize;

/// The `settings:` block of `config/*.yaml`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CoreSettings {
    /// This service's own name, e.g. `billing`. Names its event stream.
    #[serde(default = "default_service")]
    pub service: String,
    #[serde(default)]
    pub grpc: GrpcSettings,
    #[serde(default)]
    pub events: EventsSettings,
    #[serde(default)]
    pub sentry: SentrySettings,
    /// Uploaded files.
    #[serde(default)]
    pub storage: StorageSettings,
    /// The address clients reach this service at, for links it hands out,
    /// such as file downloads. Left out, it is the server's host and port,
    /// which fits development but not a server behind a proxy.
    pub api_url: Option<String>,
}

/// Where uploaded files are kept. `driver: local` keeps them on this
/// machine's disk; anything else is Cloudflare R2.
#[derive(Debug, Clone, Deserialize)]
pub struct StorageSettings {
    #[serde(default = "default_storage_driver")]
    pub driver: String,
    /// Local disk root, when `driver` is `local`.
    #[serde(default = "default_storage_dir")]
    pub dir: String,
    /// R2 bucket name, when `driver` is not `local`.
    pub r2_bucket: Option<String>,
    /// R2's S3-compatible endpoint, e.g.
    /// `https://<account id>.r2.cloudflarestorage.com`.
    pub r2_endpoint: Option<String>,
    pub r2_access_key_id: Option<String>,
    pub r2_secret_access_key: Option<String>,
}

impl Default for StorageSettings {
    fn default() -> Self {
        Self {
            driver: default_storage_driver(),
            dir: default_storage_dir(),
            r2_bucket: None,
            r2_endpoint: None,
            r2_access_key_id: None,
            r2_secret_access_key: None,
        }
    }
}

/// Error reporting to Sentry. Without a DSN nothing is sent.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SentrySettings {
    /// The project DSN. Empty or missing turns reporting off.
    pub dsn: Option<String>,
    /// Fraction of requests to trace, `0.0` to `1.0`. Defaults to off.
    #[serde(default)]
    pub traces_sample_rate: f32,
}

/// The gRPC server other services call.
#[derive(Debug, Clone, Deserialize)]
pub struct GrpcSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_grpc_binding")]
    pub binding: String,
    #[serde(default = "default_grpc_port")]
    pub port: u16,
}

impl GrpcSettings {
    /// The address this service's gRPC server binds to.
    ///
    /// # Errors
    ///
    /// When the binding and port do not form a valid address.
    pub fn bind_addr(&self) -> Result<std::net::SocketAddr> {
        format!("{}:{}", self.binding, self.port)
            .parse()
            .map_err(|e| loco_rs::Error::string(&format!("invalid gRPC address: {e}")))
    }
}

impl Default for GrpcSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            binding: default_grpc_binding(),
            port: default_grpc_port(),
        }
    }
}

/// Event streaming over NATS `JetStream`.
#[derive(Debug, Clone, Deserialize)]
pub struct EventsSettings {
    /// Off by default so a service runs without a broker until it needs one.
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_nats_url")]
    pub nats_url: String,
    /// How often the outbox relay looks for unpublished events.
    #[serde(default = "default_relay_interval_ms")]
    pub relay_interval_ms: u64,
}

impl Default for EventsSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            nats_url: default_nats_url(),
            relay_interval_ms: default_relay_interval_ms(),
        }
    }
}

impl CoreSettings {
    /// # Errors
    ///
    /// When the `settings:` block is malformed.
    pub fn load(ctx: &AppContext) -> Result<Self> {
        ctx.config.settings()
    }

    /// A named block of `settings:` that belongs to one feature, e.g.
    /// `iam` or `relay`. A missing block is its `Default`. Keeps
    /// other services' addresses out of `CoreSettings`.
    ///
    /// # Errors
    ///
    /// When the block is malformed.
    pub fn section<T: serde::de::DeserializeOwned + Default>(
        ctx: &AppContext,
        key: &str,
    ) -> Result<T> {
        match ctx.config.settings.as_ref().and_then(|all| all.get(key)) {
            Some(value) => Ok(serde_json::from_value(value.clone())?),
            None => Ok(T::default()),
        }
    }

    /// The `JetStream` stream this service publishes to: the service name,
    /// upper-cased (`billing` -> `BILLING`). It owns the subjects
    /// `zerpto.<service>.>`.
    #[must_use]
    pub fn stream_name(&self) -> String {
        self.service.to_uppercase().replace('-', "_")
    }

    /// The subject prefix this service publishes under: `zerpto.<service>`.
    #[must_use]
    pub fn subject_prefix(&self) -> String {
        format!("zerpto.{}", self.service.replace('_', "-"))
    }
}

fn default_service() -> String {
    "service-template".to_string()
}

const fn default_true() -> bool {
    true
}

fn default_grpc_binding() -> String {
    "0.0.0.0".to_string()
}

const fn default_grpc_port() -> u16 {
    50051
}

fn default_nats_url() -> String {
    "nats://localhost:4222".to_string()
}

const fn default_relay_interval_ms() -> u64 {
    500
}

fn default_storage_driver() -> String {
    "local".to_string()
}

fn default_storage_dir() -> String {
    "storage".to_string()
}
