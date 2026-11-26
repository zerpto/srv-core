use loco_rs::{Error, Result};
use tonic::transport::{Channel, Endpoint};

/// A lazily connected channel to a peer service, e.g. `http://billing:50051`.
///
/// Take the address from config (`GRPC_<PEER>_URL`), never hard-code it.
/// Cloning a `Channel` is cheap; build one per peer and share it.
///
/// ```ignore
/// let channel = helpers::channel("http://billing:50051")?;
/// let mut health = HealthClient::new(channel);
/// ```
///
/// # Errors
///
/// When `url` is not a valid endpoint.
pub fn channel(url: &str) -> Result<Channel> {
    let endpoint = Endpoint::from_shared(url.to_string())
        .map_err(|e| Error::string(&format!("invalid gRPC endpoint `{url}`: {e}")))?
        .connect_timeout(std::time::Duration::from_secs(3))
        .timeout(std::time::Duration::from_secs(10));
    Ok(endpoint.connect_lazy())
}
