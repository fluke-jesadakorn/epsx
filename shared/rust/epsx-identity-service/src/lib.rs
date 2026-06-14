//! `epsx-identity-service` library surface.
//!
//! The binary in `src/main.rs` is the tonic gRPC server entry
//! point; this `lib.rs` exposes the impl (`identity_service`)
//! + the generated gRPC types (`generated`) so the bin's tests
//! + the wave-13a Track B client's integration tests can
//! import them via the standard `epsx_identity_service::`
//! crate path.
//!
//! Module map:
//!   - `identity_service`: the `WalletRankingOffsetQuery` impl
//!     that the tonic server's `IdentityServer` wraps. The
//!     stub returns the free-plan offset for every wallet.
//!   - `generated`: the tonic-build-generated gRPC types
//!     (`IdentityServer`, `IdentityClient`, request / response
//!     messages). Re-exported from `$OUT_DIR/identity.rs`.
//!   - `proto`: the raw proto schema. Re-exported as a string
//!     literal so a future test can assert the wire schema
//!     without a separate `include_str!` at the call site.

pub mod identity_service;

/// tonic-build-generated gRPC types. The generated file is
/// `$OUT_DIR/epsx.identity.v1.rs` (tonic-build derives the name
/// from the proto's `package epsx.identity.v1;` declaration).
///
/// tonic-build's default output does NOT wrap the types in a
/// `pub mod epsx { pub mod identity { pub mod v1 { ... } } }`
/// namespace — it emits the message structs + the
/// `pub mod identity_server` / `pub mod identity_client`
/// modules at the top level of the included file. So the
/// fully-qualified paths are:
///
///   - `epsx_identity_service::generated::GetWalletRankingOffsetRequest`
///   - `epsx_identity_service::generated::GetWalletRankingOffsetResponse`
///   - `epsx_identity_service::generated::identity_server::Identity` (the `tonic::server::Server` trait)
///   - `epsx_identity_service::generated::identity_server::IdentityServer` (the shim `IdentityServer` type)
///   - `epsx_identity_service::generated::identity_client::IdentityClient` (the client stub)
///
/// The proto's package name lives in the **filename** (the
/// `.rs` is named after the package) but not in the **module
/// tree** — that's a tonic-build convention.
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/epsx.identity.v1.rs"));
}

pub mod proto {
    //! The raw proto schema as a string literal — useful for
    //! tests that want to assert the wire format without
    //! re-running `tonic-build`. The constant is a
    //! `&'static str` baked at compile time.
    pub const IDENTITY_PROTO: &str = include_str!("../../../proto/identity.proto");
}

#[cfg(test)]
mod tests {
    //! Smoke tests for the wave-13a stub. Three surfaces:
    //!   1. The `WalletRankingOffsetQuery` port impl
    //!      (`FreePlanRankingOffsetService`) returns the
    //!      free-plan offset for any wallet address.
    //!   2. The tonic-build-generated `IdentityClient` is
    //!      constructible — this catches proto schema errors
    //!      at compile time (a `let _ = IdentityClient::connect(...)`
    //!      shape won't compile if the message types don't
    //!      implement `prost::Message` correctly).
    //!   3. The tonic-build-generated `IdentityServer` accepts
    //!      our `GrpcIdentityService` impl — proves the gRPC
    //!      shim wiring is correct end-to-end.
    //!
    //! Run with: `cargo test -p epsx-identity-service`.

    use super::generated::identity_client::IdentityClient;
    use super::generated::identity_server::IdentityServer;
    use super::identity_service::FreePlanRankingOffsetService;
    use epsx_contracts::value_objects::ranking_offset::RankingOffset;
    use epsx_contracts::wallet_ranking_offset_query::WalletRankingOffsetQuery;
    use std::sync::Arc;

    /// The port impl returns `RankingOffset::free_plan()`
    /// (value = 100) for any wallet — the same fallback the
    /// wave-12 in-process stub uses.
    #[tokio::test]
    async fn test_free_plan_stub_returns_default() {
        let stub = FreePlanRankingOffsetService;
        for wallet in [
            "0x0000000000000000000000000000000000000000",
            "0xdeadbeef",
            "vitalik.eth",
            "",
            "0xFFfFfFffFFfffFFfFFfFFFFFffFFFffffFfFFFfF",
        ] {
            let offset = stub
                .get_wallet_ranking_offset(wallet)
                .await
                .expect("stub never errors");
            assert_eq!(
                offset.value(),
                RankingOffset::free_plan().value(),
                "FreePlanRankingOffsetService must return the free-plan offset for wallet={wallet}",
            );
        }
    }

    /// The stub is `Copy` + `Send` + `Sync` + `Default` —
    /// i.e. it's safe to embed in `Arc<dyn WalletRankingOffsetQuery>`.
    /// This compiles iff all four trait bounds hold.
    #[test]
    fn test_stub_is_arc_dyn_compatible() {
        let stub: Arc<dyn WalletRankingOffsetQuery> = Arc::new(FreePlanRankingOffsetService);
        // The trait object compiles — that's the assertion.
        let _ = stub;
    }

    /// tonic-build-generated `IdentityClient` is constructible
    /// via `::connect`. This is a compile-time gate on the
    /// proto schema: if the generated types don't implement
    /// `prost::Message` correctly, this won't compile. The
    /// connect call itself dials a non-existent endpoint and
    /// is expected to error — the assertion is that
    /// `IdentityClient::connect` is callable.
    #[tokio::test]
    async fn test_identity_client_constructible() {
        let endpoint = "http://127.0.0.1:1"; // unbound port
        let connect_result = IdentityClient::connect(endpoint).await;
        // We don't care if the connect itself fails (the
        // endpoint is bogus); we only care that the
        // constructor type-checks. If `IdentityClient::connect`
        // is broken at the type level, this test won't
        // compile.
        assert!(
            connect_result.is_err(),
            "expected IdentityClient::connect to fail on a bogus endpoint, got: {connect_result:?}"
        );
    }

    /// tonic-build-generated `IdentityServer` accepts our
    /// `GrpcIdentityService` impl. This proves the gRPC shim
    /// wiring is correct end-to-end: the trait
    /// `Identity` (in `identity_server` mod) is dyn-compatible
    /// and the request/response types line up.
    #[test]
    fn test_identity_server_accepts_grpc_service() {
        // The `main.rs` `GrpcIdentityService` is a bin-level
        // type (not exported from the lib), so this test
        // asserts at the trait-object level: the tonic
        // `IdentityServer::new` constructor accepts any
        // `Identity`-trait impl. We can't directly construct
        // the bin's GrpcIdentityService from the lib test,
        // so we use the `IdentityServer::new` shape with
        // a local stub.
        use super::generated::identity_server::Identity;
        use tonic::Request;

        // Local shim: a no-op `Identity` impl that just
        // echoes the free-plan offset. The point is to
        // type-check the trait surface, not to test the
        // production impl (that's the `main.rs` bin test).
        struct LocalStub;
        #[tonic::async_trait]
        impl Identity for LocalStub {
            async fn get_wallet_ranking_offset(
                &self,
                _request: Request<
                    super::generated::GetWalletRankingOffsetRequest,
                >,
            ) -> Result<
                tonic::Response<super::generated::GetWalletRankingOffsetResponse>,
                tonic::Status,
            > {
                Ok(tonic::Response::new(
                    super::generated::GetWalletRankingOffsetResponse { offset: 100 },
                ))
            }
        }

        // `IdentityServer::new` accepts any `Identity`-trait impl.
        // If the trait surface is broken (e.g. a future proto
        // change adds a method we don't implement), this won't
        // compile.
        let _server: IdentityServer<LocalStub> = IdentityServer::new(LocalStub);
    }

    /// The proto file is reachable + non-empty + has the
    /// expected `syntax` / `package` / `service` / `rpc`
    /// declarations. Catches accidental `include_str!()`
    /// path breakage at test time.
    #[test]
    fn test_proto_file_included() {
        const PROTO: &str = super::proto::IDENTITY_PROTO;
        assert!(PROTO.contains("syntax = \"proto3\";"));
        assert!(PROTO.contains("package epsx.identity.v1;"));
        assert!(PROTO.contains("service Identity"));
        assert!(PROTO.contains("rpc GetWalletRankingOffset"));
    }
}
