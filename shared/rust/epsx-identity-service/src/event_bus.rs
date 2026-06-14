//! `RankingOffsetEventBus` — the in-process broadcast channel that
//! the SSE endpoint reads from and the admin emit handler writes
//! to.
//!
//! Spec: `docs/wave8-service-boundary/ROADMAP.md` §17.2 (this
//! track creates that section). Day 1, the only emitter is the
//! `POST /v1/emit` admin hook; future wave-13c+ will hook the
//! real gRPC `GetWalletRankingOffset` path (or a new
//! `UpdateRankingOffset` gRPC) into the publish path so every
//! offset change fans out automatically. The contract is
//! intentionally simple: any subscriber sees every emitted
//! event; the consumer filters by wallet address on its end.
//!
//! ## Why `tokio::sync::broadcast`?
//!
//! - **Multi-subscriber fan-out.** A 1024-slot broadcast
//!   channel is the right shape: each subscriber gets its own
//!   `Receiver`, slow consumers fall behind (and the
//!   `BroadcastStreamRecvError::Lagged` variant is forwarded
//!   to the SSE handler so the client can re-subscribe).
//! - **No external infra.** A in-process broadcast bus is
//!   trivial to swap for a Redis pubsub later (the wave-10
//!   `PubsubPort` extraction already proved the seam). For
//!   day 1, all the SSE publishers (the admin hook + future
//!   gRPC `UpdateRankingOffset` impls) live in the same
//!   process as the SSE handler, so in-process broadcast is
//!   the simplest correct shape.
//! - **1024 slots.** Plenty for the dev cluster; production
//!   tuning can lift this in wave-14+ if needed. The slot
//!   count is the only knob the constructor takes so the
//!   `main.rs` startup banner can print the live value.

use tokio::sync::broadcast;

use crate::generated::RankingOffsetChange;

/// The pub-sub bus that the SSE endpoint reads from.
///
/// The admin emit handler writes to it; future tier-aware
/// impls (wave 13c+) will write to it on every offset change.
///
/// Day 1: 1024-slot broadcast channel. Plenty for the
/// dev cluster; production tuning can lift this in
/// wave-14+ if needed.
#[derive(Clone)]
pub struct RankingOffsetEventBus {
    tx: broadcast::Sender<RankingOffsetChange>,
}

impl RankingOffsetEventBus {
    /// Construct a new bus with the given broadcast-channel
    /// capacity. The capacity is the ring-buffer depth for
    /// slow consumers — events past the capacity are dropped
    /// for any subscriber that hasn't read them yet (the
    /// `BroadcastStreamRecvError::Lagged` variant is the
    /// SSE handler's signal to emit a `:lagged` comment so
    /// the client knows to re-subscribe).
    pub fn new(capacity: usize) -> Self {
        // `broadcast::channel` requires `capacity > 0`; clamp
        // to 1 to keep the contract safe even if a future
        // caller passes 0. 1 is the worst-case (every event
        // lags every subscriber) but it doesn't panic.
        let cap = capacity.max(1);
        let (tx, _rx) = broadcast::channel(cap);
        Self { tx }
    }

    /// Publish a change. Returns the number of active
    /// subscribers that received it (0 if no SSE clients
    /// are currently connected — the event is dropped).
    ///
    /// The `Result<usize, _>` shape would be appropriate if
    /// a `SendError` were meaningful, but `broadcast::Sender::send`
    /// only returns an error if there are zero receivers, and
    /// "no subscribers" is the normal day-1 state. We collapse
    /// to a `usize` so the admin emit handler's
    /// `Json(EmitResponse { delivered_to: n })` shape is
    /// natural.
    pub fn publish(&self, change: RankingOffsetChange) -> usize {
        // `send` returns `Result<usize, SendError<T>>`; the
        // error is `SendError(value)` when there are no
        // active receivers. We treat that as "0 delivered"
        // and return the value back via the `Ok` arm.
        self.tx.send(change).unwrap_or(0)
    }

    /// Subscribe to the stream. Returns a `Receiver` the SSE
    /// handler can await on for new events. Each `subscribe()`
    /// call produces an independent tail of the ring buffer;
    /// subscribers don't see each other's lagged events.
    pub fn subscribe(&self) -> broadcast::Receiver<RankingOffsetChange> {
        self.tx.subscribe()
    }

    /// The number of active subscribers. Useful for the
    /// startup banner and for the admin emit handler's
    /// "delivered_to" debugging log line.
    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl std::fmt::Debug for RankingOffsetEventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RankingOffsetEventBus")
            .field("receiver_count", &self.tx.receiver_count())
            .finish()
    }
}
