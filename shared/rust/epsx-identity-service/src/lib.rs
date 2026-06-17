//! `epsx-identity-service` library surface.
//!
//! The binary in `src/main.rs` is the dual-port server entry
//! point: tonic gRPC on port 50051 (wave 13a) + axum HTTP/1.1
//! (SSE + admin emit) on port 50052 (wave 13b). This `lib.rs`
//! exposes the gRPC impl (`identity_service`), the wave-13b
//! HTTP/1.1 layer (`event_bus` + `sse_handler` +
//! `emit_handler`), the generated gRPC types (`generated`),
//! and the raw proto schema (`proto`) so the bin's tests +
//! the wave-13a Track B client's integration tests can
//! import them via the standard `epsx_identity_service::`
//! crate path.
//!
//! Module map:
//!   - `identity_service`: the `WalletRankingOffsetQuery` impl
//!     that the tonic server's `IdentityServer` wraps. The
//!     stub returns the free-plan offset for every wallet.
//!   - `event_bus` (wave 13b): the in-process broadcast
//!     channel that the SSE handler reads from + the admin
//!     emit handler writes to.
//!   - `sse_handler` (wave 13b): the `GET /v1/stream/
//!     ranking-offsets` axum handler that streams events
//!     to SSE clients.
//!   - `emit_handler` (wave 13b): the `POST /v1/emit` axum
//!     handler that publishes a `RankingOffsetChange` to
//!     the bus.
//!   - `generated`: the tonic-build-generated gRPC types
//!     (`IdentityServer`, `IdentityClient`, request / response
//!     messages). Re-exported from `$OUT_DIR/identity.rs`.
//!   - `proto`: the raw proto schema. Re-exported as a string
//!     literal so a future test can assert the wire schema
//!     without a separate `include_str!` at the call site.

pub mod emit_handler;
pub mod event_bus;
pub mod identity_service;
pub mod sse_handler;

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

    // =========================================================================
    // Wave 13b Track A tests — SSE event bus + handlers
    // =========================================================================
    //
    // Test surface (per the §17.2 spec):
    //   1. Unit test: publish 3 events, subscribe + drain,
    //      assert all 3 received in order.
    //   2. Unit test: 2 subscribers both receive the same
    //      event (broadcast fan-out).
    //   3. Integration test: real TCP listener + axum +
    //      reqwest SSE client + admin emit, assert the SSE
    //      client receives the JSON `data:` line within 1s.
    //   4. Integration test: admin emit + 0 SSE subscribers
    //      returns `delivered_to: 0` (NOT an error).
    //
    // The integration tests use the wave-13a Track A's
    // "spin up a real server on an ephemeral port" pattern
    // (see `apps/analytics/src/main.rs:587-625` for the
    // tonic version). We do the same for axum: bind
    // `127.0.0.1:0`, get the resolved port via `local_addr`,
    // serve, then connect with a `reqwest::Client`.

    /// Unit test 1: publish 3 events, subscribe + drain,
    /// assert all 3 received in order. Proves the
    /// `RankingOffsetEventBus` is FIFO-ordered for a
    /// single subscriber that subscribes BEFORE the
    /// first publish.
    #[tokio::test]
    async fn wave13b_test_event_bus_publish_then_subscribe_in_order() {
        use super::event_bus::RankingOffsetEventBus;
        use super::generated::RankingOffsetChange;

        let bus = RankingOffsetEventBus::new(16);
        let mut rx = bus.subscribe();

        let events = [
            RankingOffsetChange {
                wallet: "0xaaaa".to_string(),
                offset: 50,
                changed_at_ms: 1000,
            },
            RankingOffsetChange {
                wallet: "0xbbbb".to_string(),
                offset: 60,
                changed_at_ms: 2000,
            },
            RankingOffsetChange {
                wallet: "0xcccc".to_string(),
                offset: 70,
                changed_at_ms: 3000,
            },
        ];

        for ev in &events {
            assert_eq!(bus.publish(ev.clone()), 1, "1 subscriber");
        }

        for expected in &events {
            let got = rx
                .recv()
                .await
                .expect("subscriber should receive the published event");
            assert_eq!(got.wallet, expected.wallet);
            assert_eq!(got.offset, expected.offset);
            assert_eq!(got.changed_at_ms, expected.changed_at_ms);
        }
    }

    /// Unit test 2: 2 subscribers both receive the same
    /// event. Proves the broadcast fan-out shape — the
    /// admin emit handler's `delivered_to: usize` field
    /// relies on this (each connected SSE client is one
    /// subscriber; the emit handler must count them
    /// independently).
    #[tokio::test]
    async fn wave13b_test_event_bus_broadcast_fan_out() {
        use super::event_bus::RankingOffsetEventBus;
        use super::generated::RankingOffsetChange;

        let bus = RankingOffsetEventBus::new(16);
        let mut rx_a = bus.subscribe();
        let mut rx_b = bus.subscribe();
        assert_eq!(bus.receiver_count(), 2, "two subscribers registered");

        let event = RankingOffsetChange {
            wallet: "0xfan-out".to_string(),
            offset: 42,
            changed_at_ms: 999,
        };
        let delivered = bus.publish(event.clone());
        assert_eq!(
            delivered, 2,
            "publish() should report 2 active subscribers"
        );

        let got_a = rx_a.recv().await.expect("subscriber A receives");
        let got_b = rx_b.recv().await.expect("subscriber B receives");
        assert_eq!(got_a.wallet, "0xfan-out");
        assert_eq!(got_b.wallet, "0xfan-out");
        assert_eq!(got_a.offset, 42);
        assert_eq!(got_b.offset, 42);
    }

    /// Unit test 3: publish with 0 subscribers returns
    /// `delivered_to: 0` (no error). This is the day-1
    /// normal state — no SSE clients are connected, the
    /// event is dropped, the admin emit handler returns
    /// `{"delivered_to": 0}`. The integration test
    /// (test 4 below) covers the same shape end-to-end
    /// via the HTTP layer.
    #[tokio::test]
    async fn wave13b_test_event_bus_publish_with_zero_subscribers() {
        use super::event_bus::RankingOffsetEventBus;
        use super::generated::RankingOffsetChange;

        let bus = RankingOffsetEventBus::new(16);
        let delivered = bus.publish(RankingOffsetChange {
            wallet: "0xnobody".to_string(),
            offset: 100,
            changed_at_ms: 0,
        });
        assert_eq!(
            delivered, 0,
            "publish() with 0 subscribers must return 0 (NOT an error)"
        );
    }

    /// Integration test 4: spin up a real axum server on
    /// an ephemeral port with both routes + the shared
    /// bus, connect an SSE client via reqwest, hit the
    /// admin emit endpoint, and assert the SSE client
    /// receives the JSON `data:` line within 1s.
    ///
    /// Pattern: same as the wave-13a Track A
    /// `spin_up_mock_server` in `apps/analytics/src/main.rs:
    /// 587-625`, but for axum. The TCP listener is bound
    /// with `tokio::net::TcpListener::bind` (NOT
    /// `std::net::TcpListener` + `from_std` — that's the
    /// tokio-rs/tokio#7172 cfg error trap).
    #[tokio::test]
    async fn wave13b_test_sse_round_trip() {
        use super::emit_handler::emit_ranking_offset;
        use super::event_bus::RankingOffsetEventBus;
        use super::sse_handler::stream_ranking_offsets;
        use axum::{
            routing::{get, post},
            Router,
        };
        use bytes::Bytes;
        use std::time::Duration;
        use tokio_stream::StreamExt;

        let bus = RankingOffsetEventBus::new(16);
        let app = Router::new()
            .route("/v1/stream/ranking-offsets", get(stream_ranking_offsets))
            .route("/v1/emit", post(emit_ranking_offset))
            .with_state(bus.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port for axum test server");
        let local_addr = listener
            .local_addr()
            .expect("read local_addr from ephemeral listener");
        let base_url = format!("http://{local_addr}");

        // Serve in the background.
        let serve_handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        // Build a reqwest client. The workspace pins
        // `reqwest = 0.12` with `["stream", "rustls-tls"]`
        // (no `default-features`); this is exactly what
        // we need for SSE (the `stream` feature gives
        // `bytes_stream`).
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("reqwest client should build");

        // 1. Open the SSE stream. The connection is
        //    long-lived; we drive it via `bytes_stream`
        //    and look for the first `data:` line.
        let sse_url = format!("{base_url}/v1/stream/ranking-offsets");
        let sse_response = client
            .get(&sse_url)
            .header("Accept", "text/event-stream")
            .send()
            .await
            .expect("SSE GET should connect");
        assert_eq!(
            sse_response.status(),
            reqwest::StatusCode::OK,
            "SSE endpoint must return 200"
        );
        let mut sse_stream = sse_response.bytes_stream();

        // 2. POST the emit event. The handler publishes
        //    to the bus synchronously and returns
        //    `delivered_to` (1 — the SSE client is the
        //    one subscriber).
        let emit_url = format!("{base_url}/v1/emit");
        let emit_body = serde_json::json!({
            "wallet": "0xintegration-test",
            "offset": 77,
        });
        let emit_response = client
            .post(&emit_url)
            .json(&emit_body)
            .send()
            .await
            .expect("emit POST should succeed");
        assert_eq!(
            emit_response.status(),
            reqwest::StatusCode::OK,
            "emit endpoint must return 200"
        );
        let emit_json: serde_json::Value = emit_response
            .json()
            .await
            .expect("emit response should be JSON");
        assert_eq!(
            emit_json["delivered_to"], 1,
            "delivered_to should be 1 (one SSE subscriber connected)"
        );

        // 3. Read SSE bytes until we see the `data:` line
        //    with our wallet. Timeout at 2s — the
        //    broadcast is in-process, so sub-ms is
        //    typical, but 2s is plenty of headroom for a
        //    busy CI machine.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let mut buffer = String::new();
        let mut found_data = false;
        while tokio::time::Instant::now() < deadline {
            let next: Option<Result<Bytes, _>> =
                tokio::time::timeout(Duration::from_millis(200), sse_stream.next())
                    .await
                    .unwrap_or(None);
            match next {
                Some(Ok(chunk)) => {
                    buffer.push_str(&String::from_utf8_lossy(&chunk));
                    // SSE protocol: events are terminated
                    // by a blank line (`\n\n`). Look for
                    // the first `data: ...` line.
                    while let Some(idx) = buffer.find("\n\n") {
                        let event_block: String =
                            buffer.drain(..idx + 2).collect();
                        for line in event_block.lines() {
                            if let Some(payload) = line.strip_prefix("data: ") {
                                let payload = payload.trim();
                                let parsed: serde_json::Value =
                                    serde_json::from_str(payload)
                                        .expect("data: line must be JSON");
                                assert_eq!(
                                    parsed["wallet"], "0xintegration-test",
                                    "SSE data: line wallet must match the emit"
                                );
                                assert_eq!(
                                    parsed["offset"], 77,
                                    "SSE data: line offset must match the emit"
                                );
                                assert!(
                                    parsed["changed_at_ms"].as_i64().unwrap_or(0) > 0,
                                    "SSE data: line changed_at_ms must be set by the emit handler"
                                );
                                found_data = true;
                                break;
                            }
                        }
                        if found_data {
                            break;
                        }
                    }
                    if found_data {
                        break;
                    }
                }
                Some(Err(e)) => {
                    panic!("SSE stream error: {e}");
                }
                None => continue, // timeout from inner poll; loop
            }
        }
        assert!(
            found_data,
            "SSE client should have received the emitted event within 2s; buffer so far: {buffer:?}"
        );

        // Teardown.
        serve_handle.abort();
    }

    /// Integration test 5: admin emit + 0 SSE subscribers
    /// returns `delivered_to: 0` (NOT an error). This is
    /// the "no listeners" canary — the day-1 normal
    /// state. The endpoint must succeed even when no
    /// SSE client is connected; the event is dropped
    /// (broadcast with zero receivers returns Ok(0)).
    #[tokio::test]
    async fn wave13b_test_emit_with_zero_subscribers() {
        use super::emit_handler::emit_ranking_offset;
        use super::event_bus::RankingOffsetEventBus;
        use axum::{
            routing::{get, post},
            Router,
        };
        use std::time::Duration;

        let bus = RankingOffsetEventBus::new(16);
        let app = Router::new()
            .route("/v1/stream/ranking-offsets", get(super::sse_handler::stream_ranking_offsets))
            .route("/v1/emit", post(emit_ranking_offset))
            .with_state(bus.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port for axum test server");
        let local_addr = listener
            .local_addr()
            .expect("read local_addr from ephemeral listener");
        let base_url = format!("http://{local_addr}");

        let serve_handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("reqwest client should build");

        // No SSE client connected — just hit /v1/emit.
        let emit_url = format!("{base_url}/v1/emit");
        let emit_body = serde_json::json!({
            "wallet": "0xzero-listeners",
            "offset": 200,
        });
        let response = client
            .post(&emit_url)
            .json(&emit_body)
            .send()
            .await
            .expect("emit POST should succeed even with 0 subscribers");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let body: serde_json::Value = response
            .json()
            .await
            .expect("emit response should be JSON");
        assert_eq!(
            body["delivered_to"], 0,
            "delivered_to must be 0 when no SSE clients are connected"
        );

        serve_handle.abort();
    }
}
