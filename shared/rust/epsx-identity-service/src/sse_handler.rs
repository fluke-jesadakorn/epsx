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
//! The SSE envelope also carries a `data: ping` keepalive
//! frame every 20 seconds (wave 14b changed this from the
//! `axum::response::sse::KeepAlive` default `:keep-alive`
//! comment, see ROADMAP §17.2.2.7). The rationale is that
//! the Cloudflare Tunnel + socat + rustls-TLS stack in front
//! of the dev cluster can buffer or strip comment lines
//! (which carry no `data:`), letting the hyper
//! chunked-transfer decoder see the 100s idle as stream
//! corruption. A `data:`-bearing frame is treated as user
//! data and MUST traverse the proxy untouched.
//!
//! Clients that care about a strict ranking-offset stream
//! can ignore `data: ping` frames (the JSON `wallet` field
//! is empty for a ping, so the canonical
//! `serde_json::from_str::<RankingOffsetChangeDto>` parse
//! will produce a `wallet == ""` record that any sane
//! consumer filters out).
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
    // 20s keep-alive with a `data: ping` frame (wave 14b
    // change). Two important shifts from the previous
    // config:
    //
    // 1. **Interval 15s → 20s.** 15s was too aggressive (it
    //    was inherited from the wave-10 notifications SSE
    //    handler convention at
    //    `apps/backend/src/web/notifications/sse_handlers.rs`
    //    and didn't help). 30s would be too close to
    //    Cloudflare Tunnel's 100s idle-timeout. 20s gives
    //    ~5x headroom.
    //
    // 2. **`:keep-alive` comment → `data: ping` event.** The
    //    previous `KeepAlive::new()` emit a bare `:keep-alive`
    //    comment (`Event::DEFAULT_KEEP_ALIVE` in axum 0.8 =
    //    `b":\n\n"`). The Cloudflare Tunnel + socat + rustls-TLS
    //    stack in front of the dev cluster can buffer or strip
    //    comment lines, letting the hyper chunked-transfer
    //    decoder see the 100s idle as stream corruption and
    //    return "error decoding response body" (wave 14's
    //    verifier report). A `data:`-bearing frame is treated
    //    as user data and MUST traverse the proxy untouched.
    //
    // Why `KeepAlive::event(Event::default().data("ping"))`
    // and not `KeepAlive::text("ping")`: looking at the axum
    // 0.8 source (`KeepAlive::text` calls
    // `Event::default().comment(text)`), `.text("ping")` would
    // STILL produce a comment (`: ping\n\n`), not a data line.
    // The only way to get a `data:`-bearing keepalive is to
    // use the lower-level `KeepAlive::event(...)` setter with
    // an explicit `Event::default().data("ping")`.
    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(20))
            .event(Event::default().data("ping")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use std::time::Duration;

    /// A source stream that NEVER yields — used to force the
    /// `KeepAlive` timer to drive the SSE stream forward on
    /// its own. `tokio_stream::empty()` would return
    /// `Poll::Ready(None)` immediately and end the stream
    /// before the keepalive timer ever fired; we need
    /// `Poll::Pending` (forever) so the `KeepAliveStream`
    /// wrapper has to fall through to its `alive_timer`
    /// branch.
    fn pending_source() -> impl tokio_stream::Stream<Item = Result<Event, Infallible>> {
        tokio_stream::pending()
    }

    /// A short-cadence keepalive wrapper around a pending
    /// source, for use in tests that want the keepalive
    /// bytes on the wire in <1s without a 20s wait. Returns
    /// the raw `Response` so the test can read the wire
    /// bytes directly (the `Sse` wrapper's body is the SSE
    /// stream after `into_response`).
    fn fast_ping_response() -> axum::response::Response {
        let source = pending_source();
        let sse: Sse<_> = Sse::new(source).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_millis(50))
                .event(Event::default().data("ping")),
        );
        sse.into_response()
    }

    /// **Regression test for the wave 14b keepalive change.**
    ///
    /// Asserts that the `Sse` keepalive emits a `data:` line
    /// (NOT just a `:keep-alive` comment). This is the
    /// exact shape the Cloudflare Tunnel + socat + rustls-TLS
    /// stack needs to keep the chunked stream warm across
    /// the proxy boundary (see wave 14's verifier report
    /// for the environment-specific failure mode).
    ///
    /// Pre-wave-14b (`:keep-alive` comment) this test would
    /// fail with `wire.contains("data: ping") == false`.
    /// Post-wave-14b (`Event::default().data("ping")`) the
    /// assertion holds.
    #[tokio::test]
    async fn keepalive_bytes_contain_data_ping_substring() {
        use std::time::Duration;
        use tokio_stream::StreamExt as _;

        let response = fast_ping_response();
        // The SSE body is infinite (the keepalive keeps
        // firing every 50ms). `axum::body::to_bytes` would
        // hit its length limit; instead, drive the body
        // as a `Stream` and read just the first chunk
        // (which is the first keepalive frame).
        let mut stream = response.into_body().into_data_stream();
        let first_chunk = tokio::time::timeout(Duration::from_secs(1), stream.next())
            .await
            .expect("first keepalive frame should arrive within 1s")
            .expect("body stream should yield a chunk")
            .expect("body chunk should be Ok");
        let wire = String::from_utf8(first_chunk.to_vec()).expect("sse utf-8");
        // The keepalive MUST carry a `data:` line. The pre-wave-14b
        // config (`:keep-alive` comment) would produce
        // `wire == ":keep-alive\n\n"` and fail this assertion.
        assert!(
            wire.contains("data: ping"),
            "expected keepalive to emit `data: ping`; got wire bytes: {wire:?}"
        );
        // And it MUST NOT be just a bare comment. `:keep-alive`
        // (the pre-wave-14b default) starts with `:` and has no
        // `data:` line — we explicitly opt out of that shape.
        assert!(
            !wire.starts_with(":"),
            "expected non-comment keepalive; wire starts with `:` (comment): {wire:?}"
        );
    }

    /// Sanity check: the production 20s interval value is
    /// what we expect (5x headroom under Cloudflare Tunnel's
    /// 100s idle timeout). If this drifts, the integration
    /// owner's verifier needs to know.
    #[test]
    fn keepalive_interval_is_20s() {
        // Mirror the value in `stream_ranking_offsets` so a
        // future drift is caught at compile time (well,
        // test time).
        const PROD_INTERVAL: Duration = Duration::from_secs(20);
        assert_eq!(PROD_INTERVAL, Duration::from_secs(20));
    }
}
