//! Generates the gRPC code from `proto/`. Needs `protoc` on the PATH
//! (`brew install protobuf`). Server and client code are both generated: the
//! owning service uses the server, everyone else the client. Only the health
//! contract is generated here; IAM's comes from the `zerpto-proto` crate.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto");
    tonic_prost_build::configure()
        .compile_protos(&["proto/zerpto/health/v1/health.proto"], &["proto"])?;
    Ok(())
}
