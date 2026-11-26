//! gRPC building blocks every service shares: the health service and its
//! generated code.
//!
//! A service passes the services it serves to the
//! [`GrpcServer`](crate::initializers::GrpcServer) initializer, which adds
//! [`health::HealthService`] and starts the server on
//! `settings::GrpcSettings::bind_addr`. Clients of other services live in their
//! own feature modules (`iam`).

pub mod health;
pub mod pb;
