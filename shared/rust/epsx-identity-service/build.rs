//! `tonic-build` integration for `shared/proto/identity.proto`.
//!
//! Generates Rust types into `$OUT_DIR/<package>.rs`. tonic-build
//! derives the output file name from the proto's `package`
//! declaration — our proto uses `package epsx.identity.v1;`, so
//! the generated file is `$OUT_DIR/epsx.identity.v1.rs`. The
//! `include!` in `src/lib.rs` references that path.
//!
//! The generated code includes:
//!   - `epsx::identity::v1::IdentityServer` (the `tonic::server::Server`
//!     trait our `GrpcIdentityService` impls in `src/main.rs`)
//!   - `epsx::identity::v1::IdentityClient` (for wave-13a Track B's
//!     analytics-binary client + the integration-gate smoke test)
//!   - `epsx::identity::v1::GetWalletRankingOffsetRequest` /
//!     `GetWalletRankingOffsetResponse` (the prost message types)
//!
//! We do NOT import any `google/protobuf/*.proto` well-known types
//! (no `Timestamp`, no `Duration`), so `compile_well_known_types(false)`
//! is a no-op optimization. We do NOT use proto3 optional fields
//! (every field is required-by-default in proto3), so no
//! `--experimental_allow_proto3_optional` is needed.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The proto file lives at the workspace root's `shared/proto/`.
    // We resolve the path relative to the crate root
    // (`CARGO_MANIFEST_DIR` is `shared/rust/epsx-identity-service/`,
    // so two `..` segments reach the workspace root and `proto/`
    // is a sibling of `rust/`).
    let proto_file = "../../proto/identity.proto";
    let proto_include_dir = "../../proto";

    let config = tonic_build::configure()
        // Emit the server-side `tonic::server::Server` trait impl
        // scaffolding (the `IdentityServer` type).
        .build_server(true)
        // Emit the client-side stub (used by wave-13a Track B +
        // the integration-gate test).
        .build_client(true);

    // Re-emit the proto file path so Cargo re-runs the build
    // script when the .proto changes.
    println!("cargo:rerun-if-changed={proto_file}");

    config.compile_protos(&[proto_file], &[proto_include_dir])?;
    Ok(())
}
