//! `stream_ranking_offsets` — the SSE endpoint that
//! broadcasts `RankingOffsetChange` events to subscribers.
//!
//! Spec: `docs/wave8-service-boundary/ROADMAP.md` §17.2 (this
//! track creates that section). The endpoint is mounted at
//! `GET /v1/stream/ranking-offsets` on port 50052 (the
//! HTTP/1.1 side of the identity binary's dual-port layout).
//!
//! ## Wire protocol
//!
//! Each `RankingOffsetChange` published to the bus becomes
//! exactly one SSE `data:` line, with a JSON body that mirrors
//! the protobuf field names (snake_case in the wire, prost
//! `Serialize` style in Rust):
//!
//! ```text
//! data: {"wallet":"0x...","offset":100,"changed_at_ms":1700000000000}
//! ```
//!
//! The SSE envelope also carries the standard `:keep-alive`
//! comment every 15 seconds (the `axum::response::sse::KeepAlive`
//! default behavior is a `:` comment, NOT a `data:` line, so
//! clients can ignore it as a no-op heartbeat).
//!
//! ## Why JSON + a separate DTO instead of `serde_json::to_string(&prost_msg)`?
//!
//! `prost::Message` only derives `Clone, PartialEq, prost::Message`.
//! It does NOT implement `serde::Serialize`. Adding a `Serialize`
//! derive to the tonic-build-generated types is possible via
//! `tonic-build`'s `serde` feature, but that pulls `serde` into
//! the generated code's public surface (a wire-shape decision
//! that a future wave-14+ would have to live with). The
//! minimal-friction approach for day 1 is a hand-written
//! `RankingOffsetChangeDto` with `Serialize` that converts from
//! the prost type. This is also the right shape for any future
//! `tier` enum (`Free`/`Pro`/`Vip`) that a wave-N+2 might add to
//! the proto — the DTO is the natural seam for translating
//! the protobuf enum to a string in the JSON.

use std::convert::Infallible;

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use serde::Serialize;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::event_bus::RankingOffsetEventBus;
use crate::generated::RankingOffsetChange;

/// The JSON envelope for a `RankingOffsetChange` over the SSE
/// stream. Field names match the protobuf field names
/// (snake_case) so a consumer that already knows the gRPC
/// schema doesn't have to learn a second naming convention.
#[derive(Debug, Clone, Serialize)]
pub struct RankingOffsetChangeDto {
    pub wallet: String,
    pub offset: i32,
    pub changed_at_ms: i64,
}

impl From<RankingOffsetChange> for RankingOffsetChangeDto {
    fn from(msg: RankingOffsetChange) -> Self {
        Self {
            wallet: msg.wallet,
            offset: msg.offset,
            changed_at_ms: msg.changed_at_ms,
        }
    }
}

/// The SSE handler. Subscribes to the bus and forwards every
/// event as a JSON `data:` line. On the rare
/// `BroadcastStreamRecvError::Lagged` path (a slow consumer
/// fell behind the ring buffer), emits a `:lagged` comment
/// line so the client knows to re-subscribe from the latest
/// event.
///
/// Mounted at `GET /v1/stream/ranking-offsets` on port 50052
/// (see `main.rs` for the router wiring).
pub async fn stream_ranking_offsets(
    State(bus): State<RankingOffsetEventBus>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = bus.subscribe();
    let stream = BroadcastStream::new(rx).map(|item| match item {
        Ok(change) => {
            // Convert the prost message to a JSON-envelope
            // DTO and serialize. `serde_json::to_string` can
            // only fail on a programming error (the DTO has
            // only primitive fields), so the unwrap_or
            // fallback is defensive — it should never fire.
            let dto: RankingOffsetChangeDto = change.into();
            let json = serde_json::to_string(&dto).unwrap_or_else(|_| {
                tracing::error!(?dto, "RankingOffsetChangeDto serialize failed (unexpected)");
                // A canonical empty envelope — the client
                // can parse it as a valid `RankingOffsetChange`
                // with a zeroed `changed_at_ms` and decide
                // what to do. Better than breaking the SSE
                // stream mid-flight.
                r#"{"wallet":"","offset":0,"changed_at_ms":0}"#.to_string()
            });
            Ok(Event::default().data(json))
        }
        Err(tokio_stream::wrappers::errors::BroadcastStreamRecvError::Lagged(
            skipped,
        )) => {
            // Slow consumer; emit a heartbeat so the client
            // knows the connection is still alive + that it
            // missed some events (the count is in the
            // comment so debugging is easy). The client
            // can re-subscribe from scratch if it wants
            // the missed events.
            tracing::warn!(skipped, "SSE client lagged behind broadcast bus");
            Ok(Event::default().comment(format!("lagged: skipped {skipped}")))
        }
    });
    // 15s keep-alive matches the wave-10 notifications SSE
    // handler convention (`apps/backend/src/web/notifications/
    //  sse_handlers.rs:282`). The default `:keep-alive` comment
    // is what `KeepAlive::new()` emits — no `data:` line, just
    // a heartbeat that any SSE client knows to ignore.
    Sse::new(stream).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(15)))
}
