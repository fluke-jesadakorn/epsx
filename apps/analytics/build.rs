//! `tonic-build` integration for the wave-13a Track B gRPC
//! client. Generates Rust client types into
//! `$OUT_DIR/epsx.identity.v1.rs` from
//! `shared/proto/identity.proto`. The `tonic::include_proto!`
//! call in `src/main.rs` then pulls those types in as the
//! `identity_proto` module.
//!
//! The generated code includes:
//!   - `identity_proto::identity_client::IdentityClient` —
//!     the tonic client stub used by `grpc_client.rs`.
//!   - `identity_proto::GetWalletRankingOffsetRequest` /
//!     `GetWalletRankingOffsetResponse` — the prost message
//!     types sent over the wire.
//!
//! The proto file lives at the workspace root's
//! `shared/proto/`. We resolve the path relative to the crate
//! root (`CARGO_MANIFEST_DIR` is `apps/analytics/`, so two
//! `..` segments reach the workspace root and `shared/proto/`
//! is a sibling of `apps/`).
//!
//! Spec: `docs/wave8-service-boundary/ROADMAP.md` §17.1 (Track B).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "../../shared/proto/identity.proto";
    let proto_include_dir = "../../shared/proto";

    // We do need the server-side scaffolding in the
    // analytics binary's TESTS (the `#[cfg(test)]` mock
    // server implements the generated `Identity` trait),
    // so `build_server(true)` keeps the tests compiling.
    // The production binary never links the server code
    // (the trait is only referenced inside `#[cfg(test)]`
    // blocks), so the binary size impact is just the
    // `IdentityServer` shim struct + trait definition,
    // which is a few hundred bytes — well under 1KB.
    //
    // The real server lives in `epsx-identity-service` and
    // uses the same `shared/proto/identity.proto` schema.
    let config = tonic_build::configure()
        .build_server(true)
        .build_client(true);

    // Re-emit the proto file path so Cargo re-runs the build
    // script when the .proto changes (defensive — the
    // `tonic::include_proto!` macro also emits a
    // `rerun-if-changed` for its own `proto/` argument).
    println!("cargo:rerun-if-changed={proto_file}");

    config.compile_protos(&[proto_file], &[proto_include_dir])?;
    Ok(())
}
