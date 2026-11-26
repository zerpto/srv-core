use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use tonic::{service::Routes, transport::Server};

use crate::{
    grpc::{health::HealthService, pb::health_v1::health_server::HealthServer},
    settings::CoreSettings,
};

/// Starts the gRPC server next to the HTTP server. Does nothing unless
/// `settings.grpc.enabled` is true.
///
/// The service passes the function that builds the services it serves:
/// `Box::new(GrpcServer(my_service::grpc::server::routes))`. It returns only
/// those: the health service is added here, and registering a gRPC service
/// twice panics.
///
/// A service with no gRPC services of its own returns `Routes::default()`.
pub struct GrpcServer(pub fn(&AppContext) -> Result<Routes>);

#[async_trait]
impl Initializer for GrpcServer {
    fn name(&self) -> String {
        "grpc-server".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let settings = CoreSettings::load(ctx)?;
        if !settings.grpc.enabled {
            return Ok(());
        }
        // Built before spawning, so a bad address or a failing service stops
        // the boot.
        let addr = settings.grpc.bind_addr()?;
        let routes = (self.0)(ctx)?.add_service(HealthServer::new(HealthService {
            service: settings.service,
        }));
        tokio::spawn(async move {
            tracing::info!(%addr, "gRPC server listening");
            if let Err(err) = Server::builder().add_routes(routes).serve(addr).await {
                tracing::error!(error = %err, "gRPC server stopped");
            }
        });
        Ok(())
    }
}
