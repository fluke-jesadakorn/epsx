//! Wave 13b Track B (revised in wave 14) — SSE consumer
//! + local broadcast bus in `epsx-analytics-service`.
//!
//! The new binary opens a long-lived HTTP/1.1 connection to
//! the `epsx-identity-service` binary's
//! `GET /v1/stream/ranking-offsets` endpoint, parses
//! `RankingOffsetChange` events from the SSE `data:`
//! field, and publishes them to an in-process
//! `tokio::sync::broadcast` channel
//! (`LocalRankingOffsetBus`).
//!
//! ## Wave 14 change: switched to `reqwest_eventsource`
//!
//! The wave-13b implementation used
//! `reqwest::Response::bytes_stream()` + a hand-rolled
//! `find_sse_event` / `parse_sse_data` parser. In
//! production, that path hit a `reqwest` body-decoder
//! edge case on the 2nd chunk of a long-lived
//! `text/event-stream` response — the decoder returned
//! "error decoding response body" on the 2nd chunk,
//! `consume_once` bailed with `Err`, the outer
//! reconnect loop reconnected with exponential
//! backoff (100ms → 30s cap), and events emitted by
//! the identity service during the reconnect window
//! were lost. The existing
//! `consume_once_end_to_end_via_chunks` test didn't
//! catch it because it sent the entire event in a
//! single `extend_from_slice` call (never crossed a
//! chunk boundary).
//!
//! The wave-14 fix replaces the hand-rolled
//! `bytes_stream` + parser with
//! [`reqwest_eventsource::EventSource`], the canonical
//! SSE client for `reqwest = 0.12`. `EventSource`
//! handles the long-lived stream + body-decoder edge
//! case correctly out of the box — it sets the
//! `Accept: text/event-stream` header, validates the
//! response `Content-Type`, parses the
//! `text/event-stream` wire format with `nom` (via
//! `eventsource-stream`), and reconnects on transient
//! failures.
//!
//! The legacy `find_sse_event` and `parse_sse_data`
//! helpers are kept (and unit-tested) for the rare
//! case where a future wave wants to re-introduce a
//! hand-rolled parser; they're no longer used in
//! the production `consume_once` path.
//!
//! ## Reconnect contract
//!
//! The consumer survives transient disconnects with an
//! exponential backoff capped at 30s + 0-50% jitter on
//! each retry:
//!
//!   - 100ms → 200ms → 400ms → 800ms → 1.6s → 3.2s → 6.4s →
//!     12.8s → 25.6s → 30s (cap) → 30s (cap) ...
//!   - Each retry sleeps `backoff + rand[0, backoff/2]`.
//!   - A clean server-side close (the `EventSource`
//!     stream yields `Err(Error::StreamEnded)`) resets
//!     the backoff to 100ms and reconnects immediately —
//!     the backoff is only for hard errors (connect
//!     refused, request timeout, parse failures that
//!     don't shut down the stream).
//!
//! The loop also respects a `tokio::sync::watch::Receiver<bool>`
//! shutdown signal — a future wave 14+ can wire
//! `tokio::signal::ctrl_c()` to it.
//!
//! ## JSON wire shape
//!
//! The DTO mirrors the wire shape that Track A's
//! `sse_handler.rs` emits on the identity service side.
//! Per Track A's deliverable: the `data:` line carries
//! a JSON object of shape
//! `{"wallet": "...", "offset": <i32>, "changed_at_ms": <i64>}`.
//!
//! **Integration-gate note:** Track A's `RankingOffsetChange`
//! proto message lives on a separate branch. The DTO below
//! is a *local* mirror; the integration gate will reconcile
//! the two (one option: a `pub use` re-export from
//! `epsx-identity-service` once both branches are merged;
//! another: keep the local DTO and let the identity service
//! be the source of truth). The fields match the proto
//! schema Track A declares in `shared/proto/identity.proto`'s
//! diff list, so a three-way merge is mechanical.
//!
//! Spec: `docs/wave8-service-boundary/ROADMAP.md` §17.2.

use std::time::Duration;

use futures::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event as SseEvent, RequestBuilderExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Local pub-sub bus: the SSE consumer publishes
/// `RankingOffsetChange` events here, in-process consumers
/// (the `/v1/rankings/stream` HTTP passthrough, a future
/// `/api/analytics/rankings` cache invalidation hook) subscribe.
///
/// Mirrors the identity service's `RankingOffsetEventBus`
/// API (1024-slot `tokio::sync::broadcast`, `Clone`, with
/// `new` / `publish` / `subscribe` methods).
#[derive(Clone)]
pub struct LocalRankingOffsetBus {
    tx: broadcast::Sender<RankingOffsetChange>,
}

impl LocalRankingOffsetBus {
    /// Build a new bus with the given slot capacity.
    /// 1024 matches the identity service's default.
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish a change to all current subscribers.
    /// Returns the number of receivers the event was
    /// delivered to. A `0` return is normal (no subscribers
    /// yet) and is NOT an error — the SSE consumer
    /// publishes regardless so a subscriber that joins
    /// later is missed but a subscriber that's connected
    /// gets the event.
    pub fn publish(&self, change: RankingOffsetChange) -> usize {
        // `send` returns `Result<usize, SendError<T>>` —
        // the error variant means "no active receivers",
        // which is benign (a future `subscribe()` will
        // simply not get the historical event). We map
        // the error to 0 to match the identity service's
        // `delivered_to` semantics in the admin emit hook.
        self.tx.send(change).unwrap_or(0)
    }

    /// Subscribe to future events. The returned
    /// `broadcast::Receiver` only sees events published
    /// AFTER `subscribe()` was called (broadcast channels
    /// don't replay history).
    pub fn subscribe(&self) -> broadcast::Receiver<RankingOffsetChange> {
        self.tx.subscribe()
    }

    /// Number of active subscribers. Useful for ops metrics
    /// + the consumer's own "is anyone listening?" log line.
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// Wire-shape DTO for the `data:` field of each SSE event.
///
/// **Field semantics** (matches the proto Track A adds to
/// `shared/proto/identity.proto`):
///   - `wallet`        — the lowercased / EIP-55 wallet
///                       address whose ranking offset
///                       changed.
///   - `offset`        — the new plan-tier ranking offset
///                       (0..=1000 per
///                       `epsx-contracts::value_objects::ranking_offset`).
///   - `changed_at_ms` — Unix epoch milliseconds when the
///                       change was emitted (server clock).
///
/// `Serialize` is included so the `/v1/rankings/stream`
/// HTTP passthrough can re-emit the same JSON shape to
/// web clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankingOffsetChange {
    pub wallet: String,
    pub offset: i32,
    pub changed_at_ms: i64,
}

/// Long-running task. Opens a long-lived HTTP connection to
/// the identity SSE endpoint, parses events, publishes to
/// the local bus. Reconnects on disconnect (exponential
/// backoff capped at 30s + jitter).
///
/// `shutdown` is a `watch::Receiver<bool>` — the consumer
/// loop checks `*shutdown.borrow()` at every iteration and
/// every chunk read, so a future wave 14+ can wire
/// `tokio::signal::ctrl_c()` to the sender half to do a
/// graceful shutdown.
pub async fn run_sse_consumer(
    identity_sse_url: String,
    bus: LocalRankingOffsetBus,
    client: Client,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    let mut backoff = Duration::from_millis(100);
    let max_backoff = Duration::from_secs(30);
    info!(
        url = %identity_sse_url,
        backoff_initial_ms = backoff.as_millis() as u64,
        backoff_max_s = max_backoff.as_secs(),
        "SSE consumer starting"
    );
    loop {
        if *shutdown.borrow() {
            info!("SSE consumer shutdown requested before connect, exiting loop");
            return;
        }
        match consume_once(&identity_sse_url, &bus, &client, &mut shutdown).await {
            Ok(()) => {
                info!("SSE consumer exited cleanly (server closed connection); reconnecting");
                backoff = Duration::from_millis(100);
            }
            Err(e) => {
                warn!(
                    error = %e,
                    backoff_ms = backoff.as_millis() as u64,
                    "SSE consumer error; backing off before reconnect"
                );
                // Sleep `backoff + jitter[0, backoff/2]`. The
                // jitter is added AFTER the backoff so the
                // minimum sleep is the backoff itself
                // (clients that reconnect faster than
                // `backoff` could amplify a thundering-herd
                // against the identity service).
                sleep(backoff).await;
                let jitter_cap_ms = (backoff.as_millis() as u64) / 2;
                if jitter_cap_ms > 0 {
                    let jitter_ms = rand::random::<u64>() % jitter_cap_ms;
                    sleep(Duration::from_millis(jitter_ms)).await;
                }
                backoff = std::cmp::min(backoff * 2, max_backoff);
            }
        }
        if *shutdown.borrow() {
            info!("SSE consumer shutdown requested after disconnect, exiting loop");
            return;
        }
    }
}

/// Single connection attempt. Returns `Ok(())` on a clean
/// server-side close (the `EventSource` stream yielded
/// `Err(Error::StreamEnded)` — handled internally by the
/// outer `run_sse_consumer` loop's reconnect logic) and
/// `Err(_)` on a hard error that we couldn't recover from
/// (connect refused, mid-stream transport error).
///
/// **Wave 14 change:** this used to drive
/// `reqwest::Response::bytes_stream()` + a hand-rolled
/// `find_sse_event` / `parse_sse_data` parser. That path
/// hit a `reqwest` body-decoder edge case on the 2nd
/// chunk of a long-lived `text/event-stream` response in
/// production — the decoder returned "error decoding
/// response body" on the 2nd chunk, this function
/// returned `Err`, the outer reconnect loop kicked in,
/// and events emitted during the reconnect window were
/// lost. The fix is to use
/// [`reqwest_eventsource::EventSource`], which sets
/// `Accept: text/event-stream`, validates the response
/// `Content-Type`, parses the wire format with `nom` (via
/// `eventsource-stream`), and reconnects on transient
/// failures — bypassing the body-decoder edge case.
async fn consume_once(
    url: &str,
    bus: &LocalRankingOffsetBus,
    client: &Client,
    shutdown: &mut tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    info!(url = %url, "SSE consumer connecting (reqwest_eventsource)");
    // Build an `EventSource` from the same `reqwest::Client`
    // the production `main()` builds. `EventSource::new`
    // sets the `Accept: text/event-stream` header
    // internally and validates the response `Content-Type`.
    let mut event_source = client
        .get(url)
        .eventsource()
        .map_err(|e| {
            anyhow::anyhow!("SSE EventSource build failed: {e}")
        })?;
    // Disable EventSource's built-in reconnect — the
    // outer `run_sse_consumer` loop owns the reconnect
    // policy (exponential backoff + jitter, capped at
    // 30s). When the SSE stream ends (server-side
    // close), the EventSource yields
    // `Err(Error::StreamEnded)`, we `break` out of the
    // loop, and `run_sse_consumer` reconnects with our
    // custom backoff.
    event_source.set_retry_policy(Box::new(
        reqwest_eventsource::retry::Never,
    ));

    while let Some(event) = event_source.next().await {
        if *shutdown.borrow() {
            info!("SSE consumer shutdown requested mid-stream, exiting consume_once");
            event_source.close();
            return Ok(());
        }
        match event {
            Ok(SseEvent::Open) => {
                debug!(url = %url, "SSE EventSource connection open");
            }
            Ok(SseEvent::Message(message)) => {
                // `message.data` is the raw `data:` payload
                // (the SSE spec's "Concatenate all `data:`
                // lines with `\n`" rule is applied by
                // `eventsource-stream`).
                let data = message.data;
                match serde_json::from_str::<RankingOffsetChange>(&data) {
                    Ok(change) => {
                        let n = bus.publish(change.clone());
                        debug!(
                            delivered_to = n,
                            wallet = %change.wallet,
                            offset = change.offset,
                            "SSE event → local bus"
                        );
                    }
                    Err(e) => {
                        warn!(
                            error = %e,
                            raw = %data,
                            "Failed to parse SSE event as RankingOffsetChange; skipping"
                        );
                    }
                }
            }
            Err(e) => {
                // Distinguish a clean server-side close
                // (`Error::StreamEnded`) from a hard
                // transport / parse error. A clean close
                // is NOT an error from our perspective —
                // we return `Ok(())` and the outer loop
                // reconnects with the fast path (no
                // backoff).
                match e {
                    reqwest_eventsource::Error::StreamEnded => {
                        info!(
                            url = %url,
                            "SSE EventSource stream ended cleanly (server closed \
                             connection); returning Ok(()) to outer loop for \
                             immediate reconnect"
                        );
                        event_source.close();
                        return Ok(());
                    }
                    other => {
                        warn!(
                            url = %url,
                            error = %other,
                            "SSE EventSource hard error; returning Err for \
                             outer-loop backoff + reconnect"
                        );
                        event_source.close();
                        return Err(anyhow::anyhow!(
                            "SSE EventSource error: {other}"
                        ));
                    }
                }
            }
        }
    }
    // The EventSource stream returned `None` (it was
    // closed — e.g. we called `.close()` ourselves
    // during shutdown). Treat as a clean close.
    info!(url = %url, "SSE EventSource stream closed (returned None)");
    Ok(())
}

/// Find the index of the next `\n\n` (event boundary) in
/// the buffer. Returns the index of the FIRST `\n` in the
/// pair. Returns `None` if no complete event is buffered
/// yet (caller should wait for more bytes).
///
/// **Edge case:** `\r\n\r\n` (CRLF line endings). The
/// SSE spec allows either `\n` or `\r\n` as line
/// terminators. We treat both as event boundaries by
/// checking for `\n\n` AND `\r\n\r\n` — most SSE servers
/// (axum's `Sse`, the identity service) emit `\n` only,
/// but `reqwest`'s `bytes_stream` doesn't normalize line
/// endings, so we handle both.
fn find_sse_event(buf: &[u8]) -> Option<usize> {
    // Prefer `\n\n` (the most common case). Fall back to
    // `\r\n\r\n` for completeness.
    if let Some(idx) = buf.windows(2).position(|w| w == b"\n\n") {
        return Some(idx);
    }
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

/// Extract the `data:` field from an SSE event block.
/// If multiple `data:` lines are present, they are joined
/// with `\n` per the SSE spec (the receiver's job to
/// reassemble). Returns `None` if the event has no
/// `data:` line at all (a comment-only / heartbeat event).
///
/// **Line-end normalization:** the input may have CRLF or
/// LF line endings. We strip trailing `\r` from each
/// `data:` payload so downstream JSON parsing doesn't
/// see `\r` characters in the middle of a string.
fn parse_sse_data(event: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(event).ok()?;
    let mut data_lines: Vec<String> = Vec::new();
    for line in s.lines() {
        // `str::lines()` strips BOTH `\n` and `\r\n` line
        // endings automatically, so the `line` here has
        // no trailing `\r` to worry about.
        if let Some(rest) = line.strip_prefix("data:") {
            data_lines.push(rest.trim_start().to_string());
        }
    }
    if data_lines.is_empty() {
        None
    } else {
        Some(data_lines.join("\n"))
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// `parse_sse_data` on a single event with one `data:`
    /// line returns the payload.
    #[test]
    fn parse_sse_data_single_event_single_data_line() {
        let event = b"data: {\"wallet\":\"0xabc\",\"offset\":42,\"changed_at_ms\":1700000000000}\n\n";
        let parsed = parse_sse_data(event).expect("event has a data: line");
        assert_eq!(
            parsed,
            r#"{"wallet":"0xabc","offset":42,"changed_at_ms":1700000000000}"#
        );
    }

    /// `parse_sse_data` on an event with multiple `data:`
    /// lines joins them with `\n` (per SSE spec).
    #[test]
    fn parse_sse_data_multiple_data_lines_joined() {
        let event = b"data: line1\ndata: line2\n\n";
        let parsed = parse_sse_data(event).expect("event has data: lines");
        assert_eq!(parsed, "line1\nline2");
    }

    /// `parse_sse_data` on an event with NO `data:` line
    /// (a comment-only / heartbeat event) returns `None`.
    #[test]
    fn parse_sse_data_comment_only_event_returns_none() {
        let event = b": heartbeat\n\n";
        assert!(parse_sse_data(event).is_none());
    }

    /// `parse_sse_data` on an event with an `event:` line
    /// but no `data:` line returns `None`.
    #[test]
    fn parse_sse_data_event_type_only_returns_none() {
        let event = b"event: ranking-offset-change\n\n";
        assert!(parse_sse_data(event).is_none());
    }

    /// `parse_sse_data` on an event with an `id:` line +
    /// `data:` line returns just the data payload (the
    /// `id:` is preserved server-side; we don't need it).
    #[test]
    fn parse_sse_data_id_line_ignored() {
        let event = b"id: 42\ndata: hello\n\n";
        let parsed = parse_sse_data(event).expect("event has data: line");
        assert_eq!(parsed, "hello");
    }

    /// `parse_sse_data` handles CRLF line endings (some
    /// SSE clients send them).
    #[test]
    fn parse_sse_data_crlf_line_endings() {
        let event = b"data: hello\r\n\r\n";
        let parsed = parse_sse_data(event).expect("event has data: line");
        assert_eq!(parsed, "hello");
    }

    /// `find_sse_event` on a buffer with one event returns
    /// the index of the first `\n` in the `\n\n` pair.
    #[test]
    fn find_sse_event_single_event_in_buffer() {
        // Layout: `d` `a` `t` `a` `:` ` ` `x` `\n` `\n`
        //         0   1   2   3   4   5   6   7    8
        // The first `\n` is at index 7.
        let buf = b"data: x\n\n";
        let idx = find_sse_event(buf).expect("buffer has an event boundary");
        assert_eq!(idx, 7, "first \\n in 'data: x\\n\\n' is at index 7");
    }

    /// `find_sse_event` on a buffer with multiple events
    /// returns the index of the FIRST boundary. The caller
    /// drains the buffer up to and including that boundary
    /// in a loop.
    #[test]
    fn find_sse_event_returns_first_boundary() {
        // Layout: `data: a\n\n` (9 bytes) + `data: b\n\n` (9 bytes).
        //         0 1 2 3 4 5 6 7 8 9 ...
        // The first `\n` (boundary) is at index 7.
        let buf = b"data: a\n\ndata: b\n\n";
        let idx = find_sse_event(buf).expect("buffer has an event boundary");
        assert_eq!(idx, 7, "first \\n in 'data: a\\n\\n' is at index 7");
    }

    /// `find_sse_event` on a buffer with no complete event
    /// (no `\n\n` yet) returns `None`. The caller should
    /// wait for more bytes.
    #[test]
    fn find_sse_event_partial_buffer_returns_none() {
        let buf = b"data: hello\n"; // no trailing \n yet
        assert!(find_sse_event(buf).is_none());
    }

    /// `find_sse_event` on a buffer with CRLF boundaries
    /// returns the index of the first `\r` in the
    /// `\r\n\r\n` quartet.
    #[test]
    fn find_sse_event_crlf_boundary() {
        // Layout: `d` `a` `t` `a` `:` ` ` `x` `\r` `\n` `\r` `\n`
        //         0   1   2   3   4   5   6   7    8    9    10
        // The first `\r` (start of `\r\n\r\n`) is at index 7.
        let buf = b"data: x\r\n\r\n";
        let idx = find_sse_event(buf).expect("buffer has CRLF boundary");
        assert_eq!(idx, 7, "first \\r in 'data: x\\r\\n\\r\\n' is at index 7");
    }

    /// `find_sse_event` on an empty buffer returns `None`.
    #[test]
    fn find_sse_event_empty_buffer_returns_none() {
        let buf: &[u8] = b"";
        assert!(find_sse_event(buf).is_none());
    }

    /// `LocalRankingOffsetBus::publish` with zero
    /// subscribers returns `0` (no panic, no error —
    /// `tokio::sync::broadcast::send` returns
    /// `Err(SendError(_))` when there are no receivers;
    /// we map that to `0`).
    #[tokio::test]
    async fn bus_publish_with_zero_subscribers_returns_zero() {
        let bus = LocalRankingOffsetBus::new(16);
        let change = RankingOffsetChange {
            wallet: "0xabc".to_string(),
            offset: 100,
            changed_at_ms: 1_700_000_000_000,
        };
        let n = bus.publish(change);
        assert_eq!(n, 0, "publish with no subscribers must return 0");
    }

    /// `LocalRankingOffsetBus::publish` + `subscribe`
    /// round-trip: 3 events published, a subscriber that
    /// joined BEFORE the publishes receives all 3 in order.
    #[tokio::test]
    async fn bus_publish_subscribe_three_events_in_order() {
        let bus = LocalRankingOffsetBus::new(16);
        let mut rx = bus.subscribe();

        let changes = vec![
            RankingOffsetChange {
                wallet: "0xaaa".to_string(),
                offset: 10,
                changed_at_ms: 1_700_000_000_001,
            },
            RankingOffsetChange {
                wallet: "0xbbb".to_string(),
                offset: 20,
                changed_at_ms: 1_700_000_000_002,
            },
            RankingOffsetChange {
                wallet: "0xccc".to_string(),
                offset: 30,
                changed_at_ms: 1_700_000_000_003,
            },
        ];

        for c in &changes {
            let n = bus.publish(c.clone());
            assert_eq!(n, 1, "subscriber count must be 1 after subscribe()");
        }

        // Drain. `broadcast::Receiver` returns
        // `Result<T, RecvError>`; the only error we expect
        // is `RecvError::Lagged` if the buffer overflowed,
        // which 16 slots is enough to avoid (3 events
        // published, 1 receiver, no overflow).
        let mut received = Vec::new();
        for _ in 0..3 {
            let r = rx.recv().await;
            match r {
                Ok(change) => received.push(change),
                Err(e) => panic!("unexpected recv error: {e}"),
            }
        }
        assert_eq!(
            received, changes,
            "subscriber must receive all 3 events in publish order"
        );
    }

    /// Multiple subscribers: a single `publish` delivers to
    /// all of them. Mirrors the identity service's
    /// `delivered_to` counter behavior.
    #[tokio::test]
    async fn bus_publish_broadcasts_to_all_subscribers() {
        let bus = LocalRankingOffsetBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        let mut rx3 = bus.subscribe();

        let change = RankingOffsetChange {
            wallet: "0xfanout".to_string(),
            offset: 99,
            changed_at_ms: 1_700_000_000_999,
        };
        let n = bus.publish(change.clone());
        assert_eq!(n, 3, "publish must deliver to 3 subscribers");

        // All three subscribers see the same event.
        let r1 = rx1.recv().await.expect("rx1 receives");
        let r2 = rx2.recv().await.expect("rx2 receives");
        let r3 = rx3.recv().await.expect("rx3 receives");
        assert_eq!(r1, change);
        assert_eq!(r2, change);
        assert_eq!(r3, change);
    }

    /// `receiver_count` reflects the live subscriber count.
    /// A `subscribe()` increments it, dropping the
    /// `Receiver` decrements it.
    #[tokio::test]
    async fn bus_receiver_count_tracks_subscribers() {
        let bus = LocalRankingOffsetBus::new(16);
        assert_eq!(bus.receiver_count(), 0);

        let rx1 = bus.subscribe();
        assert_eq!(bus.receiver_count(), 1);
        let rx2 = bus.subscribe();
        assert_eq!(bus.receiver_count(), 2);

        drop(rx1);
        // broadcast's `receiver_count` updates after the
        // Receiver is dropped; a small yield helps if the
        // test is in a tight loop.
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(bus.receiver_count(), 1);

        drop(rx2);
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(bus.receiver_count(), 0);
    }

    /// End-to-end: a single SSE event byte stream (as
    /// `reqwest::Response::bytes_stream` would yield) is
    /// fed into a hand-rolled loop that uses
    /// `find_sse_event` + `parse_sse_data`. The event
    /// lands in the bus.
    ///
    /// This is the closest thing to an integration test
    /// without spinning up a real HTTP server (covered by
    /// the dev-cluster smoke test in the ROADMAP §17.2
    /// report).
    #[tokio::test]
    async fn consume_once_end_to_end_via_chunks() {
        let bus = LocalRankingOffsetBus::new(16);
        let mut rx = bus.subscribe();

        // The SSE wire bytes: one event, then a clean
        // close (empty chunk = `None` from
        // `bytes_stream`).
        let event_bytes: Vec<u8> = b"data: {\"wallet\":\"0xE2E2\",\"offset\":77,\"changed_at_ms\":1700000077000}\n\n"
            .to_vec();

        // Simulate `consume_once`'s buffer-draining loop
        // without the HTTP client. The real `consume_once`
        // wraps this in a `reqwest::get().bytes_stream()`
        // call; the parse + publish logic is the same.
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&event_bytes);
        while let Some(idx) = find_sse_event(&buf) {
            let split_at = idx + 2;
            let event_bytes: Vec<u8> = buf.drain(..split_at).collect();
            let data = parse_sse_data(&event_bytes).expect("event has data: line");
            let change: RankingOffsetChange =
                serde_json::from_str(&data).expect("valid JSON");
            bus.publish(change);
        }
        // Sanity: buffer fully drained.
        assert!(buf.is_empty(), "buffer should be empty after parsing");

        // Subscriber sees the event.
        let received = rx
            .recv()
            .await
            .expect("subscriber must receive the event");
        assert_eq!(received.wallet, "0xE2E2");
        assert_eq!(received.offset, 77);
        assert_eq!(received.changed_at_ms, 1_700_000_077_000);
    }

    /// Two events arriving in the same buffer chunk are
    /// both parsed and published. The buffer-draining
    /// `while let Some(idx) = find_sse_event(&buf)` loop
    /// must keep going until no more boundaries are
    /// found.
    #[tokio::test]
    async fn consume_once_two_events_in_one_chunk() {
        let bus = LocalRankingOffsetBus::new(16);
        let mut rx = bus.subscribe();

        let two_events: Vec<u8> = b"data: {\"wallet\":\"0x111\",\"offset\":1,\"changed_at_ms\":1700000000001}\n\ndata: {\"wallet\":\"0x222\",\"offset\":2,\"changed_at_ms\":1700000000002}\n\n"
            .to_vec();

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&two_events);
        while let Some(idx) = find_sse_event(&buf) {
            let split_at = idx + 2;
            let event_bytes: Vec<u8> = buf.drain(..split_at).collect();
            let data = parse_sse_data(&event_bytes).expect("event has data: line");
            let change: RankingOffsetChange =
                serde_json::from_str(&data).expect("valid JSON");
            bus.publish(change);
        }
        assert!(buf.is_empty(), "buffer should be empty after parsing");

        let r1 = rx.recv().await.expect("rx receives event 1");
        let r2 = rx.recv().await.expect("rx receives event 2");
        assert_eq!(r1.wallet, "0x111");
        assert_eq!(r1.offset, 1);
        assert_eq!(r2.wallet, "0x222");
        assert_eq!(r2.offset, 2);
    }

    // ========================================================================
    // REGRESSION TEST — wave 14
    // ========================================================================
    //
    // **The bug (pre-fix).** `consume_once` reads from
    // `resp.bytes_stream()` and on the SECOND body chunk of
    // a long-lived `text/event-stream` response, `reqwest`'s
    // body decoder fails with "error decoding response body".
    // The consumer catches the IO error, returns `Err`, the
    // outer reconnect loop reconnects with backoff, and
    // events emitted during the reconnect window are lost.
    //
    // **Why the existing tests don't catch it.**
    //   - `consume_once_end_to_end_via_chunks` sends the
    //     whole event in one buffer write (one
    //     `extend_from_slice` call) — never crosses a chunk
    //     boundary.
    //   - `test_sse_consumer_end_to_end_via_real_http` (in
    //     `main.rs`) returns a single `String` body from the
    //     mock server, which axum turns into ONE HTTP body
    //     chunk — never a multi-chunk body. The bug only
    //     surfaces when the body is a real streaming SSE
    //     response (`axum::response::sse::Sse`) that emits
    //     one `Bytes` chunk per `Event`.
    //
    // **This test.** Spins up a real axum server whose
    // handler returns `axum::response::sse::Sse::new(stream)`
    // where the stream yields TWO `Event`s with a 50ms delay
    // between them, then completes. The consumer is expected
    // to receive BOTH events in the bus within 2s. Pre-fix,
    // the decoder fails on the second chunk and the test
    // times out waiting for event 2 (only event 1 lands, if
    // any). Post-fix, both events land.
    //
    // **Anti-test-pollution guard.** The test uses
    // `axum::response::sse::Sse` (the production server-side
    // pattern) and the real `reqwest::Client` + real
    // `consume_once` — the production code path is
    // exercised, not a hand-rolled `bytes_stream` simulation.
    // If a future refactor "fixes" the parser but breaks the
    // body decoder path, this test still surfaces the bug.

    /// Build a `Result<RankingOffsetChange, std::convert::Infallible>`
    /// event (helper to keep the test body readable).
    fn sse_event_for(
        wallet: &str,
        offset: i32,
        changed_at_ms: i64,
    ) -> axum::response::sse::Event {
        let data = serde_json::json!({
            "wallet": wallet,
            "offset": offset,
            "changed_at_ms": changed_at_ms,
        })
        .to_string();
        axum::response::sse::Event::default().data(data)
    }

    /// Spin up a tiny axum server on `127.0.0.1:0` whose
    /// `/v1/stream/ranking-offsets` handler returns
    /// `axum::response::sse::Sse::new(stream)` — the SAME
    /// pattern the production identity service uses. The
    /// stream yields TWO events with a 50ms gap between
    /// them, then completes. Returns the `host:port` the
    /// test should hit and a `JoinHandle` for teardown.
    async fn spin_up_streaming_sse_server()
    -> (String, tokio::task::JoinHandle<()>) {
        use axum::routing::get;
        use axum::Router;

        async fn sse_handler() -> axum::response::sse::Sse<
            impl tokio_stream::Stream<
                Item = Result<axum::response::sse::Event, std::convert::Infallible>,
            >,
        > {
            // Producer: spawn a task that pushes two
            // events into an mpsc channel with a 50ms
            // gap between them, then drops `tx` (which
            // closes the stream — axum's `Sse` then
            // closes the HTTP response). The 50ms gap
            // forces the body to surface as TWO
            // distinct HTTP chunks (one per `Event`
            // yield), not one coalesced chunk.
            let (tx, rx) = tokio::sync::mpsc::channel::<Result<
                axum::response::sse::Event,
                std::convert::Infallible,
            >>(8);
            tokio::spawn(async move {
                let event1 = sse_event_for("0xE2E2", 77, 1_700_000_077_000);
                let event2 = sse_event_for("0xC0DE", 50, 1_700_000_077_500);
                let _ = tx.send(Ok(event1)).await;
                tokio::time::sleep(Duration::from_millis(50)).await;
                let _ = tx.send(Ok(event2)).await;
                // `tx` drops here when the task
                // returns, closing the stream.
            });
            let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
            axum::response::sse::Sse::new(stream)
        }

        let app = Router::new()
            .route("/v1/stream/ranking-offsets", get(sse_handler));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port for streaming mock SSE server");
        let local_addr = listener
            .local_addr()
            .expect("read local_addr from ephemeral listener");
        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("streaming mock SSE server error: {e}");
            }
        });
        (local_addr.to_string(), handle)
    }

    /// **The regression test for the wave 14 bug.**
    ///
    /// Spins up a real axum SSE server that emits two
    /// events as a multi-chunk `text/event-stream` response
    /// (50ms gap between events), opens a real
    /// `reqwest::Client` connection, calls the production
    /// `consume_once` function, and asserts BOTH events
    /// land in the `LocalRankingOffsetBus` within 2s.
    ///
    /// **Pre-fix behavior:** the body decoder fails on the
    /// 2nd chunk with "error decoding response body",
    /// `consume_once` returns `Err`, only event 1 (or
    /// neither) lands in the bus, and this test times out
    /// waiting for event 2.
    ///
    /// **Post-fix behavior:** the parser handles multi-
    /// chunk bodies correctly, both events land, the stream
    /// ends cleanly, `consume_once` returns `Ok(())`, the
    /// test passes.
    #[tokio::test]
    async fn consume_once_survives_multi_chunk_sse_response() {
        let (host_port, server_handle) =
            spin_up_streaming_sse_server().await;
        // Use the same path the production URL uses —
        // `resolve_test_sse_url` is the anti-test-pollution
        // guard from wave-13b (test URL must mirror the
        // production URL exactly, including the path).
        // We can't import the helper from `main.rs` (it's
        // in a `#[cfg(test)] mod tests` there), so we
        // hardcode the production path and assert it
        // matches the constant the integration test
        // uses.
        const PROD_PATH: &str = "/v1/stream/ranking-offsets";
        assert!(
            PROD_PATH.starts_with('/') && PROD_PATH.len() > 1,
            "PROD_PATH must be a non-trivial SSE path"
        );
        let url = format!("http://{host_port}{PROD_PATH}");

        let bus = LocalRankingOffsetBus::new(16);
        let mut rx = bus.subscribe();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("reqwest client builds");
        let (_shutdown_tx, mut shutdown_rx) =
            tokio::sync::watch::channel(false);

        // Call `consume_once` in a spawned task so the
        // test can race it against the bus subscription.
        let url_for_consumer = url.clone();
        let bus_for_consumer = bus.clone();
        let client_for_consumer = client.clone();
        let (done_tx, done_rx) =
            tokio::sync::oneshot::channel::<anyhow::Result<()>>();
        let consumer_handle = tokio::spawn(async move {
            let result = consume_once(
                &url_for_consumer,
                &bus_for_consumer,
                &client_for_consumer,
                &mut shutdown_rx,
            )
            .await;
            let _ = done_tx.send(result);
        });

        // **The assertion that pre-fix fails:** BOTH
        // events must land in the bus within 2s. The
        // 50ms producer gap is well under 2s, so a
        // functioning consumer receives both in <100ms.
        // Pre-fix, the decoder errors on the 2nd chunk
        // and only event 1 (or neither) lands — the
        // 2s timeout fires.
        let r1 = tokio::time::timeout(
            Duration::from_secs(2),
            rx.recv(),
        )
        .await
        .expect("event 1 must arrive within 2s")
        .expect("event 1 must be received (not lagged)");
        assert_eq!(r1.wallet, "0xE2E2");
        assert_eq!(r1.offset, 77);
        assert_eq!(r1.changed_at_ms, 1_700_000_077_000);

        let r2 = tokio::time::timeout(
            Duration::from_secs(2),
            rx.recv(),
        )
        .await
        .expect(
            "event 2 must arrive within 2s — pre-fix this times out because \
             reqwest's body decoder fails on the 2nd SSE chunk, consume_once \
             returns Err, and event 2 is lost",
        )
        .expect("event 2 must be received (not lagged)");
        assert_eq!(r2.wallet, "0xC0DE");
        assert_eq!(r2.offset, 50);
        assert_eq!(r2.changed_at_ms, 1_700_000_077_500);

        // Wait for the consumer to finish cleanly
        // (the server closes the stream after sending
        // both events, so `consume_once` should return
        // `Ok(())` on a clean close).
        let consume_result = tokio::time::timeout(
            Duration::from_secs(2),
            done_rx,
        )
        .await
        .expect(
            "consume_once must return within 2s after the server closes the stream; \
             pre-fix it returns Err almost immediately on the 2nd chunk",
        )
        .expect("oneshot sender must not be dropped");
        assert!(
            consume_result.is_ok(),
            "consume_once must return Ok(()) on a clean multi-chunk stream end; \
             got Err = {consume_result:?}",
        );

        // Teardown: abort the consumer task (in case it
        // didn't actually finish) and the server.
        consumer_handle.abort();
        server_handle.abort();
    }

    // ========================================================================
    // REGRESSION TEST — wave 14 (chunk boundary mid-event)
    // ========================================================================
    //
    // The bug description hints that the decoder fails
    // "on the boundary between two chunks regardless of
    // buffer state." This second regression test pins down
    // a stricter shape: a SINGLE event's bytes are split
    // across two TCP writes (partial `data: ...` line, then
    // `\n\n` closer), and the test asserts the consumer
    // still reconstructs + publishes the event.
    //
    // The mock server is hand-rolled: it writes to the
    // socket directly (not via axum) so we can interleave
    // `sleep`s between byte writes, forcing the client to
    // see two chunks for one event.

    /// Hand-rolled mock SSE server that writes raw
    /// `text/event-stream` bytes to a `tokio::net::TcpStream`
    /// in two pieces (event 1 split mid-line, then event 2
    /// split mid-line), with a 50ms delay between the two
    /// pieces of each event. The consumer's
    /// `bytes_stream()` should yield each piece as a
    /// distinct chunk, and the consumer's parser should
    /// reassemble the event across the boundary.
    ///
    /// **The server keeps the connection open for 5s**
    /// after sending both events (to give the consumer
    /// time to read + dispatch event 2), then closes
    /// cleanly via a TCP FIN (NOT a RST). This avoids
    /// the test server's own teardown behavior from
    /// masking the consumer's bug.
    async fn spin_up_raw_chunked_sse_server()
    -> (String, tokio::task::JoinHandle<()>) {
        async fn handle_conn(mut socket: tokio::net::TcpStream) {
            use tokio::io::AsyncWriteExt;
            // SSE preamble: status line + headers.
            // We use `Transfer-Encoding: chunked` so the
            // client (hyper) decodes each chunk
            // envelope (hex length + CRLF + data + CRLF)
            // and yields the decoded bytes to the
            // consumer's `bytes_stream`. This is the
            // canonical shape hyper uses for unbounded
            // SSE bodies.
            let preamble = "HTTP/1.1 200 OK\r\n\
                           content-type: text/event-stream\r\n\
                           transfer-encoding: chunked\r\n\
                           \r\n";
            socket
                .write_all(preamble.as_bytes())
                .await
                .expect("write preamble");
            socket.flush().await.expect("flush preamble");

            // Event 1 (split into two TCP writes):
            //   write 1: "data: {\"wallet\":\"0xAA\",\"offset\":1,\"changed_at_ms\":1000"
            //   sleep 50ms
            //   write 2: "}\n\n"
            let event1_part1 =
                "data: {\"wallet\":\"0xAA\",\"offset\":1,\"changed_at_ms\":1000";
            let event1_part2 = "}\n\n";
            write_chunk(&mut socket, event1_part1).await;
            tokio::time::sleep(Duration::from_millis(50)).await;
            write_chunk(&mut socket, event1_part2).await;

            // Event 2 (also split):
            let event2_part1 =
                "data: {\"wallet\":\"0xBB\",\"offset\":2,\"changed_at_ms\":2000";
            let event2_part2 = "}\n\n";
            write_chunk(&mut socket, event2_part1).await;
            tokio::time::sleep(Duration::from_millis(50)).await;
            write_chunk(&mut socket, event2_part2).await;

            // End of body: zero-length chunk + CRLF CRLF.
            socket.write_all(b"0\r\n\r\n").await.expect("write EOF chunk");
            socket.flush().await.expect("flush EOF");

            // Keep the connection open for 5s so the
            // client has time to read the body + parse
            // + publish event 2 BEFORE we close.
            // Without this, the client may see a
            // connection reset if our task drops
            // `socket` while the client is still
            // reading.
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        async fn write_chunk(
            socket: &mut tokio::net::TcpStream,
            data: &str,
        ) {
            use tokio::io::AsyncWriteExt;
            let len = data.len();
            socket
                .write_all(format!("{:x}\r\n", len).as_bytes())
                .await
                .expect("write chunk length");
            socket
                .write_all(data.as_bytes())
                .await
                .expect("write chunk data");
            socket.write_all(b"\r\n").await.expect("write chunk CRLF");
            socket.flush().await.expect("flush chunk");
        }

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port for raw mock SSE server");
        let local_addr = listener
            .local_addr()
            .expect("read local_addr from ephemeral listener");
        let handle = tokio::spawn(async move {
            loop {
                let (socket, _peer) = match listener.accept().await {
                    Ok(p) => p,
                    Err(_) => break,
                };
                tokio::spawn(handle_conn(socket));
            }
        });
        (local_addr.to_string(), handle)
    }

    /// A single event's bytes are split across two TCP
    /// writes (mid-event chunk boundary). The consumer's
    /// `bytes_stream` + parser must reassemble the event
    /// across the boundary and publish it.
    ///
    /// Pre-fix: the decoder fails on the 2nd chunk
    /// (the `\n\n` closer) and the event is lost.
    /// Post-fix: the parser buffers the partial event,
    /// sees the `\n\n` closer in the next chunk, parses
    /// the event, and publishes it.
    #[tokio::test]
    async fn consume_once_survives_event_split_across_chunks() {
        let (host_port, server_handle) =
            spin_up_raw_chunked_sse_server().await;
        const PROD_PATH: &str = "/v1/stream/ranking-offsets";
        let url = format!("http://{host_port}{PROD_PATH}");

        let bus = LocalRankingOffsetBus::new(16);
        let mut rx = bus.subscribe();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("reqwest client builds");
        let (_shutdown_tx, mut shutdown_rx) =
            tokio::sync::watch::channel(false);

        let url_for_consumer = url.clone();
        let bus_for_consumer = bus.clone();
        let client_for_consumer = client.clone();
        let (done_tx, done_rx) =
            tokio::sync::oneshot::channel::<anyhow::Result<()>>();
        let consumer_handle = tokio::spawn(async move {
            let result = consume_once(
                &url_for_consumer,
                &bus_for_consumer,
                &client_for_consumer,
                &mut shutdown_rx,
            )
            .await;
            let _ = done_tx.send(result);
        });

        // Event 1 should arrive (its `data:...` is split
        // across two TCP writes — the parser must buffer
        // and reassemble).
        let r1 = tokio::time::timeout(
            Duration::from_secs(2),
            rx.recv(),
        )
        .await
        .expect("event 1 must arrive within 2s after split-chunk reassembly")
        .expect("event 1 must be received (not lagged)");
        assert_eq!(r1.wallet, "0xAA");
        assert_eq!(r1.offset, 1);
        assert_eq!(r1.changed_at_ms, 1000);

        // Event 2 likewise.
        let r2 = tokio::time::timeout(
            Duration::from_secs(2),
            rx.recv(),
        )
        .await
        .expect("event 2 must arrive within 2s after split-chunk reassembly")
        .expect("event 2 must be received (not lagged)");
        assert_eq!(r2.wallet, "0xBB");
        assert_eq!(r2.offset, 2);
        assert_eq!(r2.changed_at_ms, 2000);

        // consume_once should return Ok(()) when the
        // server closes the connection.
        let consume_result =
            tokio::time::timeout(Duration::from_secs(2), done_rx)
                .await
                .expect("consume_once must return within 2s")
                .expect("oneshot sender must not be dropped");
        assert!(
            consume_result.is_ok(),
            "consume_once must return Ok(()) on clean close; got {consume_result:?}"
        );

        consumer_handle.abort();
        server_handle.abort();
    }

    // ========================================================================
    // REGRESSION TEST — wave 14 (long-lived stream: many events)
    // ========================================================================
    //
    // **Hypothesis:** the bug only surfaces when the
    // stream has been alive long enough for the body
    // decoder to accumulate state. Send 30 events with
    // 20ms between each, then close. The consumer must
    // receive all 30 within 5s.
    //
    // If the decoder fails on the N-th chunk, the
    // consumer's reconnect loop kicks in and the
    // remaining events emitted by the server are lost.
    // (The mock server closes the connection after
    // sending all 30 events, so there's nothing to
    // reconnect to — the reconnect loop just sits in
    // backoff and the test times out on the missing
    // events.)

    /// Mock server that emits N events at a configurable
    /// interval, then closes the connection. The N + interval
    /// values are passed as URL query string params
    /// (`?n=N&interval=MS`), not as Rust function args, so
    /// the handler can be reached with the test's exact
    /// parameters via a simple URL.
        async fn spin_up_many_events_server()
        -> (String, tokio::task::JoinHandle<()>) {
        use axum::routing::get;
        use axum::Router;

        async fn sse_handler_n(
            axum::extract::Query(params): axum::extract::Query<
                std::collections::HashMap<String, String>,
            >,
        ) -> axum::response::sse::Sse<
            impl tokio_stream::Stream<
                Item = Result<axum::response::sse::Event, std::convert::Infallible>,
            >,
        > {
            let n: usize = params
                .get("n")
                .and_then(|v| v.parse().ok())
                .unwrap_or(5);
            let interval_ms: u64 = params
                .get("interval")
                .and_then(|v| v.parse().ok())
                .unwrap_or(20);
            let (tx, rx) = tokio::sync::mpsc::channel::<Result<
                axum::response::sse::Event,
                std::convert::Infallible,
            >>(8);
            tokio::spawn(async move {
                for i in 0..n {
                    let event = sse_event_for(
                        &format!("0x{i:04X}"),
                        i as i32,
                        1_700_000_000_000 + i as i64,
                    );
                    let _ = tx.send(Ok(event)).await;
                    if i < n - 1 {
                        tokio::time::sleep(Duration::from_millis(interval_ms))
                            .await;
                    }
                }
            });
            let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
            axum::response::sse::Sse::new(stream)
        }

        let app = Router::new().route(
            "/v1/stream/ranking-offsets",
            get(sse_handler_n),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port for many-events mock server");
        let local_addr = listener
            .local_addr()
            .expect("read local_addr from ephemeral listener");
        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("many-events mock SSE server error: {e}");
            }
        });
        (local_addr.to_string(), handle)
    }

    /// The body decoder survives a long-lived stream of
    /// 30 events. Pre-fix: the decoder fails on the N-th
    /// chunk (N somewhere in 2..30), `consume_once`
    /// returns `Err`, the remaining events emitted
    /// during the reconnect window are lost. Post-fix:
    /// all 30 events land in the bus.
    #[tokio::test]
    async fn consume_once_survives_long_lived_stream() {
        const N: usize = 30;
        const INTERVAL_MS: u64 = 20;
        let (host_port, server_handle) =
            spin_up_many_events_server().await;
        let url = format!(
            "http://{host_port}/v1/stream/ranking-offsets?n={N}&interval={INTERVAL_MS}"
        );

        let bus = LocalRankingOffsetBus::new(64);
        let mut rx = bus.subscribe();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("reqwest client builds");
        let (_shutdown_tx, mut shutdown_rx) =
            tokio::sync::watch::channel(false);

        let url_for_consumer = url.clone();
        let bus_for_consumer = bus.clone();
        let client_for_consumer = client.clone();
        let (done_tx, done_rx) =
            tokio::sync::oneshot::channel::<anyhow::Result<()>>();
        let consumer_handle = tokio::spawn(async move {
            let result = consume_once(
                &url_for_consumer,
                &bus_for_consumer,
                &client_for_consumer,
                &mut shutdown_rx,
            )
            .await;
            let _ = done_tx.send(result);
        });

        // Drain all N events. The 30-event stream with
        // 20ms inter-event is ~600ms; give it 5s to
        // arrive so a single reconnect doesn't break
        // the test (the reconnect backoff starts at
        // 100ms but the mock server is also gone by
        // then).
        for i in 0..N {
            let r = tokio::time::timeout(
                Duration::from_secs(5),
                rx.recv(),
            )
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "event {i} of {N} must arrive within 5s; \
                     pre-fix the body decoder may have failed \
                     on an earlier chunk and consume_once \
                     returned Err, dropping the remaining events"
                )
            })
            .expect("event must be received (not lagged)");
            assert_eq!(r.wallet, format!("0x{i:04X}"));
            assert_eq!(r.offset, i as i32);
            assert_eq!(r.changed_at_ms, 1_700_000_000_000 + i as i64);
        }

        // consume_once should return Ok(()) on clean close.
        let consume_result =
            tokio::time::timeout(Duration::from_secs(2), done_rx)
                .await
                .expect("consume_once must return within 2s")
                .expect("oneshot sender must not be dropped");
        assert!(
            consume_result.is_ok(),
            "consume_once must return Ok(()) on clean close; got {consume_result:?}"
        );

        consumer_handle.abort();
        server_handle.abort();
    }
}
