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
//!
//! ## Wave 16: warn-once + downgrade for first-connect chunked-decoder error
//!
//! The `reqwest`/`reqwest_eventsource` body-decoder fires a
//! `Transport(ReqwestError)` with Display `"error decoding
//! response body"` exactly ONCE per consumer process lifetime,
//! at ~42-53s after the very first connect, BEFORE any
//! app-level events have crossed the wire. This is a
//! hyper-chunked-decoder edge case on the first parse of a
//! `Transfer-Encoding: chunked` long-lived response — the
//! consumer's exponential-backoff reconnect handles it
//! cleanly with no data loss.
//!
//! Wave 15 verifier evidence documented this as a known
//! pre-existing first-connect edge case distinct from the
//! 60s-cadence bug they fixed. Wave 16 silences the
//! resulting WARN log line as a noise-reduction step, NOT
//! a correctness fix. The behavior:
//!
//!   - First occurrence in this process: log `warn!` with
//!     a one-time marker ("known recoverable chunked-decoder
//!     edge case; subsequent identical errors suppressed for
//!     this process") + the process start time + error time
//!     so timing analysis is preserved.
//!   - Subsequent identical-pattern errors: log at `debug!`
//!     level (still observable, but no longer page-worthy).
//!   - Return semantics unchanged: the warn-once is purely
//!     a log filter — the outer-loop `return Err(...)` path
//!     that drives the exponential backoff is preserved
//!     verbatim.
//!
//! The `static WARN_ONCE_CHUNKED_DECODER` `AtomicBool` flips
//! the FIRST time the heuristic matches and stays flipped for
//! the rest of the process. See
//! `consume_once_first_connect_chunked_decoder_warn_once` for
//! the unit test that pins this behavior.
//!
//! See `~/.mavis/agents/mavis/memory/rust-sse-over-proxies.md`
//! §"SSE first-connect edge case (chunked-decoder parsing)"
//! for the full repro recipe and distinguishing signal vs the
//! 60s-cadence timeout bug.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use futures::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event as SseEvent, RequestBuilderExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Process start time, captured at module-load. Used by the
/// wave-16 warn-once heuristic to distinguish the
/// "first-connect chunked-decoder" error (fires ~42-53s
/// after process start) from any sustained-stream error
/// that might fire later.
///
/// Using `Instant` (monotonic) is intentional — wall-clock
/// skew between the process and the host doesn't matter for
/// this heuristic; we only need elapsed-since-start.
static PROCESS_START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Process-lifetime flag for the wave-16 warn-once behavior.
/// Flips the first time the chunked-decoder heuristic matches
/// (in the `Err(e)` arm of `consume_once`'s event loop), and
/// stays flipped for the rest of the process. Subsequent
/// identical-pattern errors log at `debug!` and are otherwise
/// treated the same as the first occurrence (the
/// `return Err(anyhow::anyhow!(...))` path is preserved
/// verbatim, so the outer-loop reconnect-with-backoff
/// behavior is unchanged).
///
/// **Why a single global flag and not a per-`consume_once`
/// flag:** the chunked-decoder edge case fires exactly once
/// per process (it's tied to the first parse of a fresh
/// chunked stream, which only happens on the first
/// `consume_once` call). A per-call flag would miss the
/// "subsequent identical errors" case if a future wave
/// re-introduces a second chunked-decoder trigger.
///
/// **Why `AtomicBool` and not `tokio::sync::Once` / `OnceLock`:
/// we want the flag to be both writer (the heuristic arm)
/// and observer (the warn-vs-debug branch) callable from
/// any context, including `&self` methods that can't
/// borrow. `AtomicBool::compare_exchange` gives us a
/// single-writer / multi-reader flag with no `await` point
/// and no allocation. Cheap on the hot path.
static WARN_ONCE_CHUNKED_DECODER: AtomicBool = AtomicBool::new(false);

/// Heuristic: is this error the known first-connect
/// chunked-decoder edge case? Matches the wave-15 verifier
/// evidence — Display contains "error decoding" (the
/// reqwest/hyper chunked-decoder signature), and the error
/// fires within the first 60s of process lifetime (the
/// edge case fires at ~42-53s).
///
/// **Tightness:** anything that does NOT contain "error
/// decoding" in its Display falls through to the
/// unchanged `warn!` path. Real transport errors (TLS
/// handshake failures, connection refused, mid-stream
/// 5xx) won't match — they have different Display
/// strings ("error trying to connect", "HTTP status
/// server error", etc.).
///
/// **Timing window:** the 60s cap is generous on purpose.
/// The wave-15 evidence shows the edge case fires
/// 42-53s after process start; 60s gives 1.5x headroom
/// and still won't accidentally mask a mid-stream
/// chunked-decoder failure during sustained streaming
/// (those only fire at 60s cadence from the timeout,
/// which is now gone — see wave 15). The window is
/// only consulted ONCE per process (gated by
/// `WARN_ONCE_CHUNKED_DECODER`), so a long-lived
/// stream's mid-flight errors are never filtered.
fn is_first_connect_chunked_decoder_err(e: &reqwest_eventsource::Error) -> bool {
    let s = e.to_string();
    if !s.contains("error decoding") {
        return false;
    }
    match PROCESS_START.get() {
        // If the static isn't initialized yet, treat as
        // first-connect (the very first consume_once call
        // initializes it below). This is the
        // cold-start-safe fallback.
        None => true,
        Some(start) => start.elapsed() < Duration::from_secs(60),
    }
}

/// Wave 16: emit the warn-or-debug log line for a hard
/// SSE EventSource error, applying the
/// first-connect-chunked-decoder warn-once heuristic.
///
/// **Behavior:**
///   - First matching error: `warn!` with a one-time
///     marker + process-start-to-error elapsed ms.
///   - Subsequent matching errors: `debug!` only.
///   - Non-matching errors: unchanged `warn!` (the
///     pre-wave-16 behavior).
///
/// **Extracted from the `consume_once` match arm so the
/// heuristic + flag flip are directly unit-testable.**
/// The caller in `consume_once` always follows this call
/// with `event_source.close()` + `return Err(anyhow!(...))`
/// to drive the outer-loop reconnect — this function
/// intentionally does NOT touch return semantics.
fn log_sse_hard_error(url: &str, e: &reqwest_eventsource::Error) {
    if is_first_connect_chunked_decoder_err(e) {
        let first = WARN_ONCE_CHUNKED_DECODER
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok();
        if first {
            let elapsed = PROCESS_START
                .get()
                .map(|t| t.elapsed())
                .unwrap_or_default();
            warn!(
                url = %url,
                error = %e,
                process_start_to_error_ms = elapsed.as_millis() as u64,
                "SSE EventSource first-connect chunked-decoder error \
                 (known recoverable edge case; subsequent identical \
                 errors suppressed to debug! for this process); \
                 returning Err for outer-loop backoff + reconnect"
            );
        } else {
            debug!(
                url = %url,
                error = %e,
                "SSE EventSource chunked-decoder error (subsequent \
                 occurrence suppressed to debug!; see warn-once log \
                 line for the one-time marker)"
            );
        }
    } else {
        warn!(
            url = %url,
            error = %e,
            "SSE EventSource hard error; returning Err for outer-loop \
             backoff + reconnect"
        );
    }
}

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

/// **Wave 17: the shared `reqwest::Client` builder for the
/// SSE consumer.** Both the production construction site
/// (`apps/analytics/src/main.rs`) and the wave-15 regression
/// test (`consume_once_survives_long_lived_stream_with_no_per_request_timeout`)
/// call this function to get the long-lived HTTP client. This
/// is the load-bearing piece of **construction-site-parity**:
/// the production path and the test path share the SAME
/// builder chain, so a future refactor that re-adds a
/// dangerous knob (like the old `.timeout(60s)`) is caught
/// by BOTH at the same time, not just by reading the test
/// in isolation.
///
/// **The wave-15 bug this prevents.** `reqwest = 0.12`'s
/// `ClientBuilder::timeout()` is the **total request
/// timeout** — measured from `client.get(url).send()` until
/// the response body stream ends. For a long-lived SSE
/// stream that never ends, a 60s timeout kills the stream
/// at t+60s regardless of activity, and the cancellation
/// surfaces through hyper's chunked-decoder as
/// "error decoding response body" (the user-visible error
/// in the wave-14/14b reports). `reqwest = 0.12` does NOT
/// expose a separate read-timeout / idle-timeout knob —
/// the closest is `.connect_timeout()`, which bounds only
/// the TCP connect phase (a hung TCP handshake, NOT a
/// slow body). So the right answer is no client-level
/// timeout for SSE; the consumer's reconnect logic
/// (`run_sse_consumer` below) handles hung connections via
/// exponential backoff on `consume_once` errors. The 20s
/// keepalive ping in the identity service (`wave14b`) is
/// the liveness check.
///
/// **Return type.** `Result<reqwest::Client, reqwest::Error>`
/// — preserved from the original inline `main.rs:493-495`
/// shape so the call site can keep its `?` + `.context()`
/// error-propagation chain. The default `reqwest` builder
/// (no overrides) almost never returns `Err` — the only
/// failure modes are system-level (out-of-memory, TLS
/// backend init) — but returning the error preserves the
/// original main.rs's clean-error-rather-than-panic
/// behavior.
///
/// **Why this lives in `sse_consumer.rs` and not `main.rs`:
/// `sse_consumer` is the module that OWNS the SSE wire
/// contract (URL, retry policy, parsing). The HTTP client
/// is part of that contract — the test for the contract
/// (`consume_once_survives_long_lived_stream_with_no_per_request_timeout`)
/// also lives in this module, so it can import the builder
/// directly. The `main.rs` import was the inline builder
/// chain this function replaces.
///
/// **The companion marker constant.**
/// `SSE_CONSUMER_CLIENT_BUILDER_CHAIN` below is the
/// load-bearing documentation of the expected builder
/// shape. The `construction_site_parity_guards` test in
/// the `#[cfg(test)] mod` at the bottom of this file
/// pins the marker constant AND scans the function body
/// for forbidden knob patterns. A future re-add of a
/// timer-style knob is a test failure, not a code-review
/// miss.
///
/// **The construction-site-parity project memory reference.**
/// See `~/.mavis/agents/coder/memory/epsx-realtime-stack.md`
/// §"Construction-site-parity for reqwest clients" for the
/// full pattern (the SSE consumer is the canonical worked
/// example) and `modular-monolith-split-audit.md` for the
/// cross-project framework.
pub fn sse_consumer_client() -> Result<reqwest::Client, reqwest::Error> {
    // The builder chain MUST be a verbatim copy of the
    // previous main.rs:478-481 chain — same set of
    // `.timeout()`, `.connect_timeout()`,
    // `.pool_max_idle_per_host()`, etc. knobs AND the same
    // absence of `.timeout(_)`. The construction-site-
    // parity invariant is: production and the wave-15
    // regression test build the client via THIS function.
    // If you intentionally add a knob, update BOTH the
    // function body AND the marker constant
    // (`SSE_CONSUMER_CLIENT_BUILDER_CHAIN`) AND the
    // `construction_site_parity_guards::sse_consumer_client_source_has_no_timer_knobs`
    // test, and re-run the wave-15 long-lived stream test
    // with `--ignored` to verify the new builder survives
    // a 90s+ sustained stream.
    //
    // Wave 15: do NOT set `.timeout(_)` on this client.
    // See the function doc comment for the full rationale.
    //
    // The construction-site-parity guard test scans the
    // region between the marker comments immediately
    // above and below this builder call. The exact marker
    // strings are intentionally NOT mentioned in any
    // prose comment in this file (the
    // `construction_site_parity_guards` test holds them
    // as const strings only) — that way the test's
    // `find()` matches the function-body markers
    // exclusively, never a backticked prose mention.
    // The marker format is `// wave17-builder-scan-{begin|end}`
    // on its own line.
    // wave17-builder-scan-begin
    reqwest::Client::builder().build()
    // wave17-builder-scan-end
}

/// Marker constant for the shared SSE consumer builder
/// chain. Pinned by the `construction_site_parity_guards`
/// test below. If a future refactor intentionally changes
/// the builder (e.g. adds a knob), update this string to
/// match the new shape AND update the test in
/// `consume_once_survives_long_lived_stream_with_no_per_request_timeout`
/// to assert the new behavior end-to-end.
///
/// **Why a string and not a function pointer:** the
/// builder is `reqwest::Client::builder()`, which is
/// opaque — there's no stable way to extract a "builder
/// chain signature" from the live `Client` value (the
/// inner fields are private; `reqwest = 0.12` exposes no
/// `.timeout()` getter on the built `Client`). The string
/// is a human-readable contract: the future reader who
/// changes the builder MUST also update this string and
/// the test that pins it.
pub const SSE_CONSUMER_CLIENT_BUILDER_CHAIN: &str =
    "reqwest::Client::builder().build()";

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
    // Wave 16: lazy-initialize the process-start time on
    // the first `consume_once` call. This is the
    // cold-start-safe equivalent of a `static_init!` —
    // the heuristic consults `PROCESS_START.get()` (Option)
    // and treats `None` as "still in the first 60s of
    // process lifetime" so the very first error on a
    // freshly-launched consumer is always eligible for
    // the warn-once path even if the static is uninitialized.
    let _ = PROCESS_START.set(Instant::now());

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
                        // Wave 16: warn-once + downgrade for the
                        // known first-connect chunked-decoder
                        // edge case (see module doc + the
                        // `log_sse_hard_error` helper). The
                        // helper handles the heuristic + flag
                        // flip + warn-or-debug log emission;
                        // the return semantics below are
                        // unchanged — we still `return Err(...)`
                        // so the outer-loop exponential backoff
                        // kicks in exactly as before.
                        log_sse_hard_error(url, &other);
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

    // ========================================================================
    // REGRESSION TEST — wave 15
    // ========================================================================
    //
    // **The bug (pre-fix).** `reqwest::Client::builder().timeout(60s)`
    // was applied to the long-lived SSE consumer client in
    // `apps/analytics/src/main.rs:479-481`. `reqwest`'s
    // `timeout` is the **total request timeout** (request
    // start → end of response body read), NOT a per-chunk
    // read timeout. For a long-lived SSE stream, the body
    // never ends, so the timer fires at exactly 60s after
    // the request starts, the consumer's `bytes_stream`
    // gets cancelled mid-chunk, and the user-visible error
    // is "error decoding response body" (the wave-14/14b
    // symptom). This is independent of the keepalive
    // cadence, the identity service's behavior, or the
    // Cloudflare Tunnel idle timeout — it fires in the
    // in-cluster path too (analytics pod → identity pod
    // direct, no Tunnel involved).
    //
    // **The fix.** Drop the `.timeout(60s)` from the
    // production `reqwest::Client::builder()` call in
    // `apps/analytics/src/main.rs`. SSE is long-lived; the
    // total-request-timeout doesn't apply. The 20s keepalive
    // ping on the identity service (wave 14b) is the
    // liveness check, and the consumer's reconnect loop
    // (exponential backoff, 100ms → 30s cap, in
    // `run_sse_consumer`) handles true hangs.
    //
    // **This test.** Spins up a real axum SSE server that
    // emits one event every 500ms for
    // `WAVE15_SSE_DURATION_SECS` seconds (default 90s =
    // longer than the old 60s timeout), then closes. The
    // consumer must drain ALL events into the bus.
    //
    //   - **Pre-fix behavior:** the `reqwest::Client` the
    //     test uses (built the SAME way the production
    //     `main()` builds it — see "construction site" below)
    //     carries a 60s total-request-timeout. The stream
    //     dies at t+60s, the consumer reconnects with
    //     backoff, the events emitted by the server between
    //     t+60s and t+90s are lost, the test fails because
    //     the bus has fewer than the expected count.
    //   - **Post-fix behavior:** the client has no
    //     per-request timeout, the stream runs the full
    //     duration, ALL events land, the test passes.
    //
    // **The construction site.** This test calls the
    // shared `sse_consumer_client()` builder exported
    // from this module, which is THE SAME function the
    // production `apps/analytics/src/main.rs` calls.
    // Pre-wave-17, this test inlined the production
    // builder chain; a future refactor could re-introduce
    // `.timeout(60s)` to either site without the other
    // being updated. Wave 17 fixes that structurally:
    // both sites call `sse_consumer_client()`, drift
    // between them is impossible, and the
    // `construction_site_parity_guards` test pins the
    // builder shape (forbidden knob scan + marker
    // constant). See the doc comment on
    // `sse_consumer_client()` for the full wave-15 bug
    // rationale + the reqwest=0.12 timeout landscape.
    //
    // **Default duration + env override.** The default
    // `WAVE15_SSE_DURATION_SECS` is 90 (1.5x the old
    // 60s timeout — large enough that pre-fix code fails
    // the test with a wide margin, not a flake). Override
    // with `WAVE15_SSE_DURATION_SECS=...` to speed up
    // local dev (e.g. set to 65 for a minimal-but-failing
    // pre-fix run). The test is `#[ignore]`d by default
    // because 90s of real time is too slow for the
    // ordinary `cargo test` loop — run it explicitly with:
    //
    //   cargo test --bin epsx-analytics-service \
    //       consume_once_survives_long_lived_stream_with_no_per_request_timeout \
    //       -- --ignored
    //
    // (or with a smaller env override for a faster smoke
    // check:
    //  `WAVE15_SSE_DURATION_SECS=65 cargo test --bin epsx-analytics-service
    //     consume_once_survives_long_lived_stream_with_no_per_request_timeout
    //     -- --ignored`).

    /// Build a streaming SSE server that emits one event
    /// every `interval_ms` for `duration_secs` seconds, then
    /// closes the connection. The total event count is
    /// `duration_secs * 1000 / interval_ms + 1` (the
    /// `+1` is the first event emitted at t=0). Both values
    /// are passed via the URL query string so the handler
    /// can be reached with the test's exact parameters.
    async fn spin_up_long_lived_sse_server() -> (String, tokio::task::JoinHandle<()>) {
        use axum::routing::get;
        use axum::Router;

        async fn sse_handler(
            axum::extract::Query(params): axum::extract::Query<
                std::collections::HashMap<String, String>,
            >,
        ) -> axum::response::sse::Sse<
            impl tokio_stream::Stream<
                Item = Result<axum::response::sse::Event, std::convert::Infallible>,
            >,
        > {
            let duration_secs: u64 = params
                .get("duration_secs")
                .and_then(|v| v.parse().ok())
                .unwrap_or(90);
            let interval_ms: u64 = params
                .get("interval_ms")
                .and_then(|v| v.parse().ok())
                .unwrap_or(500);
            let (tx, rx) = tokio::sync::mpsc::channel::<Result<
                axum::response::sse::Event,
                std::convert::Infallible,
            >>(8);
            tokio::spawn(async move {
                // Compute the expected event count up front
                // so we can log it once on the producer side
                // (helps the operator correlate the test
                // output with the expected count).
                let total = (duration_secs * 1000) / interval_ms + 1;
                tracing::info!(
                    duration_secs,
                    interval_ms,
                    expected_events = total,
                    "long-lived SSE server: starting producer"
                );
                for i in 0..total {
                    let event = sse_event_for(
                        &format!("0x{i:06X}"),
                        i as i32,
                        1_700_000_000_000 + i as i64,
                    );
                    if tx.send(Ok(event)).await.is_err() {
                        // consumer hung up; bail.
                        break;
                    }
                    if i < total - 1 {
                        tokio::time::sleep(Duration::from_millis(interval_ms))
                            .await;
                    }
                }
                // `tx` drops when the task returns, closing
                // the stream.
            });
            let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
            axum::response::sse::Sse::new(stream)
        }

        let app = Router::new().route(
            "/v1/stream/ranking-offsets",
            get(sse_handler),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port for long-lived mock SSE server");
        let local_addr = listener
            .local_addr()
            .expect("read local_addr from ephemeral listener");
        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("long-lived mock SSE server error: {e}");
            }
        });
        (local_addr.to_string(), handle)
    }

    /// **The regression test for the wave-15 bug.**
    ///
    /// Spins up a real axum SSE server that emits one
    /// event every 500ms for 90+ seconds (longer than
    /// the old `.timeout(60s)` on the consumer's
    /// `reqwest::Client`), opens a real connection
    /// through the production-shaped client, runs the
    /// production `consume_once`, and asserts that ALL
    /// events landed in the bus.
    ///
    /// **Pre-fix behavior:** the 60s total-request-timeout
    /// kills the stream at t+60s, the consumer reconnects
    /// with backoff, events emitted between t+60s and
    /// t+90s are lost, the test fails because the bus
    /// has fewer than the expected count (~180 events
    /// emitted, ~120 received).
    ///
    /// **Post-fix behavior:** the client has no
    /// per-request timeout, the stream runs the full
    /// duration, ALL events land, the test passes.
    ///
    /// The test is `#[ignore]`d by default because the
    /// 90s default duration is too slow for the ordinary
    /// `cargo test` loop. Run it explicitly with
    /// `cargo test --bin epsx-analytics-service
    ///   consume_once_survives_long_lived_stream_with_no_per_request_timeout
    ///   -- --ignored`.
    #[tokio::test]
    #[ignore = "90s+ runtime — run with --ignored; CI should run this regularly"]
    async fn consume_once_survives_long_lived_stream_with_no_per_request_timeout() {
        // Duration: 90s default (1.5x the old 60s timeout).
        // Override with `WAVE15_SSE_DURATION_SECS=...` for
        // local dev (e.g. 65 for a minimal-but-failing
        // pre-fix run).
        let duration_secs: u64 = std::env::var("WAVE15_SSE_DURATION_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(90);
        // 500ms cadence: 90s / 0.5s = 180 events + 1
        // (the t=0 event) = 181 events total. Plenty of
        // resolution to catch "stream dies at exactly
        // 60s" as a clear pre-fix failure.
        let interval_ms: u64 = 500;
        let expected_events: u64 = (duration_secs * 1000) / interval_ms + 1;

        eprintln!(
            "[wave15 regression] starting: duration={}s, interval={}ms, \
             expected_events={}",
            duration_secs, interval_ms, expected_events
        );

        let (host_port, server_handle) =
            spin_up_long_lived_sse_server().await;
        let url = format!(
            "http://{host_port}/v1/stream/ranking-offsets?duration_secs={duration_secs}&interval_ms={interval_ms}"
        );

        let bus = LocalRankingOffsetBus::new(2048);
        let mut rx = bus.subscribe();

        // **Construction site — must mirror the production
        // builder in `apps/analytics/src/main.rs`.** Wave
        // 17: the production builder AND this test now both
        // call `sse_consumer_client()`, the shared helper
        // exported from this module. This is the
        // construction-site-parity pattern: a future
        // refactor that re-adds `.timeout(_)` (or any other
        // dangerous knob) to `sse_consumer_client()` will
        // change BOTH the production and test sites at the
        // same time, AND the
        // `construction_site_parity_guards` test pins the
        // builder shape. Pre-wave-17 this test inlined the
        // SAME builder as production; post-wave-17 it
        // calls the shared function — drift between the
        // two sites is now structurally impossible.
        let client = sse_consumer_client()
            .expect("wave-15 regression: sse_consumer_client() must build; \
                     the default builder should not fail");
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

        // Drain all expected events. The per-event timeout
        // is `(duration_secs + 30)s` — generous headroom
        // over the 500ms inter-event interval, well under
        // the 5s slop the inner assertion needs.
        let drain_timeout = Duration::from_secs(duration_secs + 30);
        let mut received_count: u64 = 0;
        let start = std::time::Instant::now();
        while received_count < expected_events {
            let r = tokio::time::timeout(drain_timeout, rx.recv())
                .await
                .unwrap_or_else(|_| {
                    panic!(
                        "[wave15 regression] event {}/{} did not arrive within {:?}; \
                         pre-fix the consumer's 60s reqwest timeout kills the stream at \
                         t+60s, the consumer reconnects with backoff, and events emitted \
                         between t+60s and t+{}s are lost. After the fix all {} events \
                         must land.",
                        received_count, expected_events, drain_timeout,
                        duration_secs, expected_events
                    )
                })
                .expect("event must be received (not lagged)");
            // Sanity-check the event contents. A
            // pre-fix failure that loses some events
            // but the test's race timing lets the
            // reconnected consumer pick up the
            // server's tail might still produce the
            // expected COUNT but with the wrong
            // CONTENT (e.g. duplicate wallet 0x000042
            // because the server restarted its index
            // on the post-timeout reconnect — though
            // our mock doesn't, but a future test
            // variant could). Pin the content too.
            assert_eq!(
                r.wallet,
                format!("0x{:06X}", received_count),
                "[wave15 regression] wallet for event {received_count} is wrong; \
                 the stream order must be preserved end-to-end"
            );
            assert_eq!(r.offset, received_count as i32);
            assert_eq!(r.changed_at_ms, 1_700_000_000_000 + received_count as i64);
            received_count += 1;
            if received_count % 20 == 0 {
                eprintln!(
                    "[wave15 regression] drained {}/{} events ({:.1}s elapsed)",
                    received_count,
                    expected_events,
                    start.elapsed().as_secs_f64()
                );
            }
        }
        eprintln!(
            "[wave15 regression] drained all {received_count} events in {:.1}s",
            start.elapsed().as_secs_f64()
        );
        assert_eq!(
            received_count, expected_events,
            "[wave15 regression] received count must equal expected count"
        );

        // consume_once should return Ok(()) on clean close
        // (the mock server closes the stream after the
        // last event).
        let consume_result =
            tokio::time::timeout(Duration::from_secs(5), done_rx)
                .await
                .expect(
                    "consume_once must return within 5s after the server closes the stream"
                )
                .expect("oneshot sender must not be dropped");
        assert!(
            consume_result.is_ok(),
            "consume_once must return Ok(()) on clean close; got {consume_result:?}"
        );

        consumer_handle.abort();
        server_handle.abort();
    }

    // ========================================================================
    // REGRESSION TEST — wave 16 (warn-once + downgrade for first-connect
    //                   chunked-decoder error)
    // ========================================================================
    //
    // **Hypothesis:** the warn-once flag in
    // `WARN_ONCE_CHUNKED_DECODER` flips exactly once per
    // process, and the `log_sse_hard_error` helper emits
    // 1 `warn!` line + 2 `debug!` lines for 3 consecutive
    // matching errors. Non-matching errors (any error whose
    // Display does NOT contain "error decoding") fall
    // through to the unchanged `warn!` path and never
    // touch the flag.
    //
    // **Why this test does NOT drive a real EventSource:**
    // the actual first-connect chunked-decoder edge case
    // fires at ~42-53s on a fresh consumer process (see
    // `~/.mavis/agents/mavis/memory/rust-sse-over-proxies.md`
    // §"SSE first-connect edge case"). A real round-trip
    // test would take 45-60s and need a backend that emits
    // a malformed `Transfer-Encoding: chunked` response.
    // The heuristic is the load-bearing logic; we exercise
    // it directly with a synthetic
    // `reqwest_eventsource::Error` whose Display contains
    // the substring "error decoding".
    //
    // **Why `Error::InvalidLastEventId`:** it's the only
    // `reqwest_eventsource::Error` variant whose Display
    // we can fully control from outside the crate (its
    // Display is `"Invalid \`Last-Event-ID\`: {s}"`). The
    // other variants either need inner `reqwest::Error`
    // values (opaque) or `nom::error::Error` values (not a
    // direct dep). By passing a string that embeds "error
    // decoding" we synthesize an error that displays as
    // `"Invalid \`Last-Event-ID\`: error decoding response
    // body"`, which the heuristic matches.
    //
    // **Threading note:** the `WARN_ONCE_CHUNKED_DECODER`
    // static is process-lifetime, so this test is NOT
    // reentrant within a single binary. By design the test
    // is the only one in the analytics crate that touches
    // the flag (the production path goes through
    // `consume_once`, which is exercised end-to-end by
    // the other `consume_once_*` tests, but those use real
    // axum servers that emit valid SSE and never hit the
    // `other` arm in the `match e`). If a future test ever
    // needs to verify the un-flipped state, it should be
    // wrapped in a test-binary of its own (a separate
    // `[[test]]` target) so the static is fresh.
    #[test]
    fn consume_once_first_connect_chunked_decoder_warn_once() {
        use std::io::Write;
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::fmt::MakeWriter;

        /// In-memory writer that captures every `write!`
        /// call into a shared `Vec<u8>`. Used as the
        /// `MakeWriter` for a per-test
        /// `tracing_subscriber::fmt` layer so the test can
        /// read back the captured log lines and assert on
        /// the warn-vs-debug ratio.
        #[derive(Clone)]
        struct VecWriter(Arc<Mutex<Vec<u8>>>);
        impl Write for VecWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        impl<'a> MakeWriter<'a> for VecWriter {
            type Writer = VecWriter;
            fn make_writer(&'a self) -> Self::Writer {
                self.clone()
            }
        }

        let buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let writer = VecWriter(buf.clone());
        let subscriber = tracing_subscriber::fmt()
            .with_writer(writer)
            .with_max_level(tracing::Level::TRACE)
            .with_ansi(false)
            .with_target(false)
            .finish();

        // Synthesize a matching-pattern error: its Display
        // is `"Invalid \`Last-Event-ID\`: error decoding
        // response body (synthetic for test)"`, which
        // contains the substring "error decoding".
        fn matching_err() -> reqwest_eventsource::Error {
            reqwest_eventsource::Error::InvalidLastEventId(
                "error decoding response body (synthetic for test)"
                    .to_string(),
            )
        }
        // Non-matching error: Display does NOT contain
        // "error decoding".
        fn non_matching_err() -> reqwest_eventsource::Error {
            reqwest_eventsource::Error::InvalidLastEventId(
                "totally unrelated Last-Event-ID (synthetic for test)"
                    .to_string(),
            )
        }

        tracing::subscriber::with_default(subscriber, || {
            // Sanity: PROCESS_START is uninitialized in
            // the test binary → the heuristic falls into
            // the `None => true` branch, so a
            // matching-pattern error is always treated as
            // first-connect regardless of the test's
            // wall-clock elapsed time. (See
            // `is_first_connect_chunked_decoder_err`.)
            assert!(
                is_first_connect_chunked_decoder_err(&matching_err()),
                "heuristic must match an error whose Display contains \
                 'error decoding' when PROCESS_START is uninitialized"
            );
            assert!(
                !is_first_connect_chunked_decoder_err(&non_matching_err()),
                "heuristic must NOT match an error whose Display does \
                 not contain 'error decoding'"
            );

            // First call: flips the flag → emits 1 warn!.
            log_sse_hard_error(
                "http://test.invalid/v1/stream/ranking-offsets",
                &matching_err(),
            );
            // Second + third calls: flag already flipped →
            // emit 2 debug! lines.
            log_sse_hard_error(
                "http://test.invalid/v1/stream/ranking-offsets",
                &matching_err(),
            );
            log_sse_hard_error(
                "http://test.invalid/v1/stream/ranking-offsets",
                &matching_err(),
            );

            // Non-matching error: must always emit warn!
            // regardless of the flag state, and must NOT
            // touch the flag.
            log_sse_hard_error(
                "http://test.invalid/v1/stream/ranking-offsets",
                &non_matching_err(),
            );
        });

        // Read back the captured log lines and assert.
        let captured = String::from_utf8(buf.lock().unwrap().clone())
            .expect("captured log buffer is valid utf-8");

        let warn_count = captured
            .matches("WARN SSE EventSource first-connect")
            .count()
            + captured
                .matches("WARN SSE EventSource hard error")
                .count();
        let debug_count = captured
            .matches("DEBUG SSE EventSource chunked-decoder")
            .count();

        // 3 matching calls → 1 warn (first) + 2 debug
        // (subsequent) for the chunked-decoder path.
        // 1 non-matching call → 1 warn on the
        // pre-wave-16 path. Total: 2 warn + 2 debug.
        assert_eq!(
            warn_count, 2,
            "expected 2 WARN lines total (1 first-connect + 1 hard-error), \
             got {warn_count}. Captured log:\n{captured}"
        );
        assert_eq!(
            debug_count, 2,
            "expected 2 DEBUG lines (subsequent chunked-decoder \
             occurrences), got {debug_count}. Captured log:\n{captured}"
        );

        // The first-connect WARN must include the
        // one-time marker phrase + the
        // `process_start_to_error_ms` field.
        assert!(
            captured.contains("known recoverable edge case"),
            "first-connect WARN must include the one-time marker \
             phrase 'known recoverable edge case'. Captured:\n{captured}"
        );
        assert!(
            captured.contains("process_start_to_error_ms="),
            "first-connect WARN must include the \
             `process_start_to_error_ms` field. Captured:\n{captured}"
        );

        // The non-matching WARN must use the
        // pre-wave-16 "hard error" message (NOT the
        // first-connect marker).
        assert!(
            captured.contains("returning Err for outer-loop backoff + reconnect"),
            "non-matching error must emit the unchanged 'hard error' \
             WARN. Captured:\n{captured}"
        );

        // The flag must remain flipped after all calls.
        // (The next call to `log_sse_hard_error` with a
        // matching error — outside this test — would
        // emit `debug!` again, confirming the
        // one-shot semantics.)
        assert!(
            WARN_ONCE_CHUNKED_DECODER.load(Ordering::Acquire),
            "WARN_ONCE_CHUNKED_DECODER must remain flipped after the \
             first matching call; it is process-lifetime and not \
             auto-reset"
        );
    }
}

// ============================================================================
// Wave 17: construction-site-parity guards
// ============================================================================
//
// **The drift detectors.** These tests pin the
// `sse_consumer_client()` builder shape so a future refactor
// that re-adds a timer-style knob (the wave-15 bug shape) is
// caught at `cargo test` time, not at production runtime.
//
// **Why two layers of defense:**
//   1. The marker-constant test pins the human-readable
//      "expected builder shape" string. If a future reader
//      intentionally changes the builder, they MUST update
//      this string AND the wave-15 regression test in the
//      same commit, OR this test fails.
//   2. The source-substring test scans the
//      `pub fn sse_consumer_client` function body for
//      forbidden knob patterns (`.timeout(`,
//      `.connect_timeout(`, etc.) at test time. This is
//      a true "structural" check — it sees the actual code,
//      not a string that might have drifted from the code.
//
// **Why not a const-eval / `static_assertions` approach:**
// `reqwest = 0.12`'s `Client` has no public getter for the
// configured timeout (the inner fields are private), so we
// can't introspect the built `Client` at runtime to assert
// "the timeout is `None`". A `static_assertions` const-eval
// check on the builder chain isn't possible because the
// builder is a non-`const` expression. The
// `include_str!`-based substring check is the most
// pragmatic approach for this specific knob.
//
// **The construction-site-parity invariant.** Both the
// production `main.rs` site AND the wave-15 regression
// test now call `sse_consumer_client()`. If a future commit
// re-introduces an inline `reqwest::Client::builder()...`
// chain in either site, the test below still passes (it's
// checking `sse_consumer_client`, not the call sites) —
// the invariant is enforced by code review, not by a test.
// The test's job is narrower: catch a future commit that
// re-adds a dangerous knob to `sse_consumer_client()`
// itself.
//
// **The scope guard.** The forbidden-knob substring check
// is scoped to the function body region (from
// `pub fn sse_consumer_client` to end-of-file), NOT the
// whole file. This avoids false positives from the module
// doc comment at the top of the file (which legitimately
// mentions `.timeout(` and `.connect_timeout(` to explain
// the wave-15 bug).

#[cfg(test)]
mod construction_site_parity_guards {
    /// The marker constant must match the current expected
    /// builder shape. If a future refactor changes the
    /// builder chain in `sse_consumer_client()`, this test
    /// fails UNLESS the marker constant is updated in the
    /// same commit. The marker constant is the
    /// human-readable contract that links the production
    /// site to the wave-15 regression test.
    #[test]
    fn sse_consumer_client_builder_chain_marker_is_current() {
        assert_eq!(
            super::SSE_CONSUMER_CLIENT_BUILDER_CHAIN,
            "reqwest::Client::builder().build()",
            "SSE_CONSUMER_CLIENT_BUILDER_CHAIN has drifted from the actual \
             builder chain. If you intentionally changed the builder (e.g. \
             added a knob), update BOTH this marker constant AND the \
             wave-15 regression test in \
             consume_once_survives_long_lived_stream_with_no_per_request_timeout \
             to assert the new behavior end-to-end. Re-run the wave-15 \
             test with --ignored (default 90s, override with \
             WAVE15_SSE_DURATION_SECS=...) to verify the new builder \
             survives a sustained stream longer than the new timeout."
        );
    }

    /// The forbidden knobs — any re-add to
    /// `sse_consumer_client()` of one of these method calls
    /// will fail this test at `cargo test` time. Scoped to
    /// the **builder chain expression region** (delimited
    /// by the `// wave17-builder-scan-begin` and
    /// `// wave17-builder-scan-end` marker comments
    /// inside the function body), NOT the whole function
    /// body or the whole file. This scoping avoids false
    /// positives from:
    ///   - The module-level doc comment (which legitimately
    ///     explains the wave-15 bug by name).
    ///   - The function-level doc comment (which also
    ///     mentions `.timeout(_)` to explain the
    ///     wave-15 root cause).
    ///   - The `construction_site_parity_guards` test
    ///     itself (which contains `.timeout(` as a string
    ///     literal in the `FORBIDDEN_BUILDER_KNOBS` list
    ///     below).
    /// The markers are a structural contract: a future
    /// refactor that adds a real builder call to the
    /// function MUST place it between the markers, OR
    /// update this test to use a new scoping rule. The
    /// marker strings are NEVER mentioned in any prose
    /// comment in this file (the test holds them as
    /// const strings only) so the `find()` call below
    /// matches the function-body markers exclusively,
    /// never a backticked prose mention.
    #[test]
    fn sse_consumer_client_source_has_no_timer_knobs() {
        // The marker comments that delimit the scan region.
        // These are placed inside the function body
        // immediately around the actual builder call.
        // The exact strings are NEVER mentioned in any
        // prose comment elsewhere in the file (the test
        // holds them as const strings only) so the
        // `find()` call below matches the function-body
        // markers exclusively.
        const SCAN_BEGIN: &str = "// wave17-builder-scan-begin";
        const SCAN_END: &str = "// wave17-builder-scan-end";

        let source = include_str!("sse_consumer.rs");
        let begin_idx = source.find(SCAN_BEGIN).unwrap_or_else(|| {
            panic!(
                "could not find `{SCAN_BEGIN}` marker in sse_consumer.rs; \
                 the source layout has drifted from what the guard expects. \
                 Either restore the marker inside the \
                 `sse_consumer_client()` function body (around the actual \
                 builder call) or update the \
                 `construction_site_parity_guards` test to use a new \
                 scoping rule."
            )
        });
        let begin_after = begin_idx + SCAN_BEGIN.len();
        let end_idx = source[begin_after..].find(SCAN_END).unwrap_or_else(|| {
            panic!(
                "could not find `{SCAN_END}` marker in sse_consumer.rs \
                 after the `{SCAN_BEGIN}` marker; the source layout has \
                 drifted. Restore the marker pair inside the \
                 `sse_consumer_client()` function body."
            )
        }) + begin_after;
        let body_region = &source[begin_after..end_idx];

        // Each entry is a forbidden builder method call —
        // including the open-paren so we only match actual
        // method invocations, not variable names or comments
        // that mention the word alone.
        const FORBIDDEN_BUILDER_KNOBS: &[&str] = &[
            ".timeout(",          // wave-15 root cause
            ".connect_timeout(",  // wrong-fit (TCP-only)
            ".read_timeout(",     // doesn't exist in 0.12, but flag future adds
        ];

        for knob in FORBIDDEN_BUILDER_KNOBS {
            assert!(
                !body_region.contains(knob),
                "sse_consumer_client() body contains forbidden knob \
                 `{knob}` between the {SCAN_BEGIN} / {SCAN_END} markers. \
                 The whole point of construction-site-parity is that the \
                 production and test paths share the same builder. \
                 Re-adding a timer-style knob will bring back the \
                 wave-15 60s-cadence 'error decoding' bug (or a related \
                 timer-bomb shape). See the doc comment on \
                 sse_consumer_client() for the load-bearing rationale. \
                 If you intentionally need this knob, update BOTH the \
                 function body AND SSE_CONSUMER_CLIENT_BUILDER_CHAIN \
                 AND the wave-15 regression test expectations, AND \
                 re-run the wave-15 test with --ignored to verify the \
                 new builder survives a sustained stream."
            );
        }
    }

    /// `sse_consumer_client()` produces a usable client. This
    /// is a smoke test — the function returns
    /// `Result<Client, reqwest::Error>`, mirroring the
    /// original `main.rs` `?` + `.context()` shape. With
    /// the default builder (no overrides), the `Err` arm
    /// is unreachable in practice; the only failure modes
    /// are system-level (out-of-memory, TLS backend init).
    /// If this test fails, the helper itself has a bug;
    /// the structural checks above catch knob-level drift.
    #[test]
    fn sse_consumer_client_builds_a_reqwest_client() {
        let _client = super::sse_consumer_client()
            .expect("sse_consumer_client: default builder should not fail; \
                     the only failure modes are system-level (out-of-memory, \
                     TLS backend init) which are not recoverable");
    }
}
