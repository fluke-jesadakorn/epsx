//! PubsubPort — generic publish/subscribe primitive for the EPSX kernel.
//!
//! ## Why this trait exists
//!
//! Wave 8's `audit-notifications.md` §3c / §3d / §6a flagged that the
//! chat domain reuses `RedisNotificationBroadcaster` (the notifications
//! module's typed wrapper) for its own pubsub on channels like `chat:new`,
//! `chat:agent:<id>`, and `chat:wallet:<addr>`. Before the notifications
//! service is lifted, the broadcaster has to be hoisted to a kernel-level
//! generic pubsub primitive so chat can keep working without depending on
//! notifications.
//!
//! This file is that primitive. The notifications module's
//! notification-specific wrappers (`publish_to_wallet`,
//! `subscribe_to_wallet`, `publish_to_all`) are retained as thin
//! adapters that delegate to this port. The chat SSE stream and the
//! notification SSE stream both go through the same `PubsubPort` after
//! this refactor — they only differ in which channel names they publish
//! to and subscribe to.
//!
//! ## Design notes
//!
//! - `subscribe` returns a `Box<dyn MessageStream>` rather than a concrete
//!   stream type. The Redis adapter wraps `redis::aio::PubSubStream` and
//!   the in-memory adapter wraps `tokio::sync::broadcast::Receiver`. The
//!   SSE handlers call `next_message().await` in a loop and never touch
//!   the underlying transport.
//! - `subscribe` accepts `&[&str]` rather than a single channel so the
//!   admin chat stream can subscribe to `chat:new` + `chat:agent:<id>`
//!   in one call (matches the existing `redis_broadcaster::subscribe_to_channel`
//!   + extra `ps.subscribe(&agent_channel).await` pattern at
//!   `web/admin/chat_handlers.rs:443-454`).
//! - `publish` returns `AppResult<()>` not `usize`. The number of
//!   subscribers reached is transport-specific (Redis returns the count;
//!   in-memory broadcast has its own semantics) and is not part of the
//!   port contract. Callers that need the count wrap the adapter.
//! - `MessageStream` is `Send` but not necessarily `Sync` (a
//!   `redis::aio::PubSubStream` is not `Sync`). It is `Unpin` because
//!   the SSE handlers `.next().await` it through `async_stream!` which
//!   requires `Unpin`.

use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;

use crate::errors::AppResult;

// `#[async_trait]` makes the trait dyn-compatible (so it can be used
// as `Arc<dyn PubsubPort>` in the DI container) and adds an explicit
// `Send` bound on the returned future. The native `async fn` in
// trait syntax would be more ergonomic but is not object-safe —
// `#[async_trait]` is the standard workaround.
#[async_trait]

/// A channel-agnostic pub/sub port.
///
/// Implementations may publish through Redis (the production adapter),
/// an in-process broadcast bus (tests), or any other transport that
/// honors the contract.
pub trait PubsubPort: Send + Sync {
    /// Publish a payload to a single channel.
    ///
    /// `payload` is a raw byte slice — the port does not interpret it.
    /// Notifications serialize `SSENotification` to JSON before calling
    /// this; chat serializes its own event struct to JSON. Decoding
    /// happens at the subscriber.
    async fn publish(&self, channel: &str, payload: &[u8]) -> AppResult<()>;

    /// Subscribe to one or more channels on a fresh stream.
    ///
    /// Each call returns an *independent* stream that owns its own
    /// underlying subscription. The caller is expected to keep the
    /// returned `Box<dyn MessageStream>` alive for as long as it wants
    /// to receive messages; dropping the stream unsubscribes.
    fn subscribe(
        &self,
        channels: &[&str],
    ) -> AppResult<Box<dyn MessageStream + Send + Unpin>>;
}

/// A stream of raw byte payloads from one or more subscribed channels.
///
/// The trait is intentionally minimal: a single `next_message` future
/// that returns `Some(payload)` for each message received and `None`
/// when the underlying subscription has ended. Adapters may keep
/// channel names alongside the payload (e.g. as a tuple), but the
/// current callers only need the payload.
///
/// `next_message` is dyn-compatible by returning
/// `Pin<Box<dyn Future<...>>>` instead of `impl Trait`.
pub trait MessageStream {
    /// Wait for the next message on the subscribed channels.
    ///
    /// Returns `None` when the subscription is closed by the adapter
    /// (e.g. the Redis connection is dropped, or the in-memory
    /// broadcast sender is dropped).
    fn next_message(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Option<Vec<u8>>> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `PubsubPort` is `dyn`-compatible. The test just checks the trait
    /// object compiles (the compiler rejects non-object-safe methods).
    #[test]
    fn trait_is_object_safe() {
        fn _is_object_safe(_p: &dyn PubsubPort) {}
        fn _is_object_safe2(_s: &dyn MessageStream) {}
    }
}
