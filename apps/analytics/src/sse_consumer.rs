//! Wave 13b Track B — SSE consumer + local broadcast bus in
//! `epsx-analytics-service`.
//!
//! The new binary opens a long-lived HTTP/1.1 connection to
//! the `epsx-identity-service` binary's
//! `GET /v1/stream/ranking-offsets` endpoint, parses
//! `RankingOffsetChange` events from the SSE `data:`
//! field, and publishes them to an in-process
//! `tokio::sync::broadcast` channel
//! (`LocalRankingOffsetBus`).
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
//!   - A clean server-side close (the `stream.next().await`
//!     returns `None`) resets the backoff to 100ms and
//!     reconnects immediately — the backoff is only for
//!     hard errors (connect refused, request timeout, parse
//!     failures that don't shut down the stream).
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
/// server-side close (the bytes stream yielded `None`) and
/// `Err(_)` on connect failure / HTTP error / IO error / a
/// mid-stream IO error. Parse failures on individual
/// `data:` lines are logged + skipped (NOT bubbled up — a
/// single malformed event shouldn't tear down the
/// connection).
async fn consume_once(
    url: &str,
    bus: &LocalRankingOffsetBus,
    client: &Client,
    shutdown: &mut tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    info!(url = %url, "SSE consumer connecting");
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| {
            anyhow::anyhow!("SSE connect failed: {e}")
        })?;
    let resp = resp.error_for_status().map_err(|e| {
        anyhow::anyhow!("SSE connect non-2xx status: {e}")
    })?;
    let mut stream = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = stream.next().await {
        if *shutdown.borrow() {
            info!("SSE consumer shutdown requested mid-stream, exiting consume_once");
            return Ok(());
        }
        let chunk = chunk.map_err(|e| {
            anyhow::anyhow!("SSE bytes_stream chunk error: {e}")
        })?;
        buf.extend_from_slice(&chunk);
        // Parse SSE event boundaries (events are delimited
        // by a blank line — `\n\n` per the SSE spec). Each
        // chunk may contain zero, one, or many events; we
        // drain the buffer left-to-right.
        while let Some(idx) = find_sse_event(&buf) {
            // `idx` is the position of the first `\n` in
            // the `\n\n` pair. We want to keep everything
            // AFTER the second `\n` in the buffer.
            let split_at = idx + 2;
            let event_bytes: Vec<u8> = buf.drain(..split_at).collect();
            match parse_sse_data(&event_bytes) {
                Some(data) => match serde_json::from_str::<RankingOffsetChange>(&data) {
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
                },
                None => {
                    // An SSE event with no `data:` field is a
                    // comment / heartbeat / event-type-only
                    // event. Per the SSE spec, those are
                    // legitimate. Log at debug and move on.
                    debug!("SSE event had no data: field; skipping");
                }
            }
        }
    }
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
}
