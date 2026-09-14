//! `emit_ranking_offset` — the admin hook that publishes a
//! `RankingOffsetChange` to the bus.
//!
//! Spec: `docs/wave8-service-boundary/ROADMAP.md` §17.2 (this
//! track creates that section). The endpoint is mounted at
//! `POST /v1/emit` on port 50052 (the HTTP/1.1 side of the
//! identity binary's dual-port layout).
//!
//! Day 1: this is the ONLY publisher. The integration gate
//! uses it to drive a round-trip smoke test; future wave-13c+
//! will hook the gRPC `GetWalletRankingOffset` path (or a new
//! `UpdateRankingOffset` gRPC) into the publish path so every
//! offset change fans out automatically.
//!
//! ## Auth / authorization
//!
//! The day-1 endpoint is **unauthenticated** — the dev cluster
//! has no JWT verification on the HTTP/1.1 path. The K8s base
//! manifest keeps the service `ClusterIP` (the dev overlay
//! exposes a NodePort for smoke testing), so production
//! never sees the endpoint directly. A future wave-14+ will
//! either (a) move the endpoint behind a JWT bearer middleware
//! that calls the wave-10 R8b `validate_access_token`, or (b)
//! deprecate the HTTP admin hook in favor of a `tonic::Request`
//! header in the gRPC layer.
//!
//! ## Request / response shape
//!
//! Request body (JSON):
//!
//! ```json
//! { "wallet": "0xdeadbeef", "offset": 100 }
//! ```
//!
//! Response body (JSON):
//!
//! ```json
//! { "delivered_to": 3 }
//! ```
//!
//! `delivered_to` is the number of active SSE subscribers that
//! received the event. Zero is a valid response (no SSE
//! clients are currently connected — the event is dropped).

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::event_bus::RankingOffsetEventBus;
use crate::generated::RankingOffsetChange;

/// The JSON request body for `POST /v1/emit`.
///
/// Both fields are required. The server does not validate
/// `wallet` (it treats the string as opaque, matching the
/// `GetWalletRankingOffsetRequest.wallet` contract). `offset`
/// is the new ranking offset; out-of-range values are
/// forwarded as-is (the protobuf wire type is `int32`; a
/// future wave-14+ may add a 0..=1000 range check).
#[derive(Debug, Clone, Deserialize)]
pub struct EmitRequest {
    pub wallet: String,
    pub offset: i32,
}

/// The JSON response body for `POST /v1/emit`.
///
/// `delivered_to` is the number of active SSE subscribers
/// that received the event. Zero means no SSE clients were
/// connected at the moment of the publish — the event was
/// dropped. This is the day-1 normal state; the integration
/// gate's smoke test runs with one SSE client connected so
/// `delivered_to >= 1` is the canary for "the round-trip
/// works".
#[derive(Debug, Clone, Serialize)]
pub struct EmitResponse {
    pub delivered_to: usize,
}

/// The admin emit handler. Builds a `RankingOffsetChange`
/// from the request (stamping `changed_at_ms` at publish
/// time so the consumer doesn't have to trust its own
/// clock), publishes it to the bus, and returns the number
/// of subscribers that received it.
///
/// Mounted at `POST /v1/emit` on port 50052 (see `main.rs`
/// for the router wiring).
pub async fn emit_ranking_offset(
    State(bus): State<RankingOffsetEventBus>,
    Json(req): Json<EmitRequest>,
) -> Json<EmitResponse> {
    let changed_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let change = RankingOffsetChange {
        wallet: req.wallet.clone(),
        offset: req.offset,
        changed_at_ms,
    };

    let delivered_to = bus.publish(change);

    info!(
        wallet = %req.wallet,
        offset = req.offset,
        changed_at_ms,
        delivered_to,
        "RankingOffsetEventBus: admin emit"
    );

    Json(EmitResponse { delivered_to })
}
