use tonic::{Request, Response, Status};

use super::pb::health_v1::{health_server::Health, CheckRequest, CheckResponse};

/// Answers `zerpto.health.v1.Health/Check`.
pub struct HealthService {
    pub service: String,
}

#[tonic::async_trait]
impl Health for HealthService {
    async fn check(
        &self,
        _request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        Ok(Response::new(CheckResponse {
            status: "ok".to_string(),
            service: self.service.clone(),
        }))
    }
}
