# zerpto-core

The shared crate of the Zerpto microservices. See [AGENTS.md](AGENTS.md) for
what belongs here, the change rules and how to release.

```toml
# in a service's Cargo.toml
zerpto-core = { git = "<repo url>", tag = "v0.1.0", features = ["events", "storage", "iam", "relay"] }
```

Needs `protoc` to build (`brew install protobuf`). Postgres only; no SQLite.

## gRPC server

A service lists the gRPC services it serves in one function and registers it in
`App::initializers`:

```rust
Box::new(GrpcServer(my_service::grpc::server::routes))
```

The function has the type `fn(&AppContext) -> Result<tonic::service::Routes>`.
Core adds the health service and listens on `settings.grpc`; do not add health
in the service.
