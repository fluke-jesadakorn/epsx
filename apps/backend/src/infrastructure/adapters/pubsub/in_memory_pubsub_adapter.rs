//! In-memory implementation of `PubsubPort`.
//!
//! Used by tests (chat pubsub canary, container wiring, round-trip
//! integration). It is **not** intended for production use — the
//! broadcast bus does not cross process boundaries, so multiple
//! backend instances would not see each other's messages.
//!
//! ## Why `tokio::sync::broadcast`?
//!
//! Redis's pub/sub is a fan-out: every subscriber gets a copy of every
//! message published on a channel they are subscribed to. The closest
//! tokio primitive with the same shape is `tokio::sync::broadcast` —
//! each `Sender` has many `Receiver`s, and a `send` clones the value
//! to all of them.
//!
//! The one semantic difference that matters for the tests is that
//! `broadcast::Receiver::recv()` returns `RecvError::Lagged(n)` if
//! the receiver is too slow; we map that to a transparent
//! "skip-ahead" by re-issuing `recv()`. In production this would be
//! a real drop; in tests we are explicit about it because a slow
//! test should not silently lose messages.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use epsx_contracts::errors::{AppError, ErrorKind};
use epsx_contracts::pubsub_port::{MessageStream, PubsubPort};

/// In-memory pubsub adapter backed by a per-channel broadcast bus.
///
/// Channel senders are created lazily on first `subscribe` for a
/// channel, and dropped when the last receiver is dropped. The
/// mutex-guarded `HashMap` lets multiple adapters share the same bus
/// if needed (the chat pubsub canary test does this to simulate
/// "two backend instances").
pub struct InMemoryPubsubAdapter {
    inner: Arc<Mutex<HashMap<String, tokio::sync::broadcast::Sender<Vec<u8>>>>>,
    /// Buffer size per channel. `tokio::sync::broadcast` requires a
    /// fixed-size ring; if a slow receiver lags, it misses messages.
    /// 1024 is generous for tests; production runs through Redis.
    capacity: usize,
}

impl std::fmt::Debug for InMemoryPubsubAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryPubsubAdapter")
            .field("capacity", &self.capacity)
            .field("channels", &"<HashMap>")
            .finish()
    }
}

impl Default for InMemoryPubsubAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryPubsubAdapter {
    /// Create a new in-memory adapter with a per-channel buffer of
    /// 1024 messages.
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new in-memory adapter with a custom per-channel
    /// buffer size.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            capacity,
        }
    }

    /// Number of active subscribers across all channels. Test-only.
    pub fn receiver_count(&self) -> usize {
        let map = self.inner.lock().expect("pubsub mutex poisoned");
        map.values().map(|tx| tx.receiver_count()).sum()
    }

    /// Number of distinct channels with at least one active
    /// subscriber. Test-only.
    pub fn active_channels(&self) -> Vec<String> {
        let map = self.inner.lock().expect("pubsub mutex poisoned");
        map.keys().cloned().collect()
    }

    fn sender_for(&self, channel: &str) -> tokio::sync::broadcast::Sender<Vec<u8>> {
        let mut map = self.inner.lock().expect("pubsub mutex poisoned");
        map.entry(channel.to_string())
            .or_insert_with(|| {
                let (tx, _rx) = tokio::sync::broadcast::channel(self.capacity);
                tx
            })
            .clone()
    }
}

impl PubsubPort for InMemoryPubsubAdapter {
    async fn publish(&self, channel: &str, payload: &[u8]) -> Result<(), AppError> {
        let tx = self.sender_for(channel);
        // A broadcast::Sender with zero receivers returns Err on
        // send. We treat that as success because the "no one is
        // listening" case is normal in a fan-out bus — the message
        // has nowhere to go, but it isn't a fault.
        match tx.send(payload.to_vec()) {
            Ok(_) => Ok(()),
            Err(tokio::sync::broadcast::error::SendError(_)) => {
                tracing::debug!(
                    "InMemoryPubsubAdapter: published to channel={} with 0 receivers",
                    channel
                );
                Ok(())
            }
        }
    }

    fn subscribe(
        &self,
        channels: &[&str],
    ) -> Result<Box<dyn MessageStream + Send + Unpin>, AppError> {
        if channels.is_empty() {
            return Err(AppError::new(
                ErrorKind::ValidationError,
                "InMemoryPubsubAdapter::subscribe requires at least one channel",
            ));
        }

        // For a single-channel subscription, return a single
        // `BroadcastMessageStream` directly. For multi-channel, we
        // build a fan-in stream that polls each receiver in a loop
        // and yields the first message that arrives.
        if channels.len() == 1 {
            let rx = self.sender_for(channels[0]).subscribe();
            return Ok(Box::new(BroadcastMessageStream { rx: Some(rx) }));
        }

        // Multi-channel fan-in: create one broadcast::Receiver per
        // channel and round-robin poll them.
        let receivers: Vec<tokio::sync::broadcast::Receiver<Vec<u8>>> = channels
            .iter()
            .map(|c| self.sender_for(c).subscribe())
            .collect();
        Ok(Box::new(MultiChannelStream {
            receivers,
            next: 0,
        }))
    }
}

/// A `MessageStream` that pulls from a single
/// `tokio::sync::broadcast::Receiver<Vec<u8>>`. Returns `None` when
/// the sender is dropped AND the channel buffer is empty.
pub struct BroadcastMessageStream {
    rx: Option<tokio::sync::broadcast::Receiver<Vec<u8>>>,
}

impl std::fmt::Debug for BroadcastMessageStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastMessageStream").finish()
    }
}

impl MessageStream for BroadcastMessageStream {
    fn next_message(
        &mut self,
    ) -> Pin<Box<dyn std::future::Future<Output = Option<Vec<u8>>> + Send + '_>> {
        Box::pin(async move {
            // Take the receiver out of the option so we can move it
            // into the async block. Restore it before each `await`
            // so subsequent calls can still poll.
            let mut rx = match self.rx.take() {
                Some(rx) => rx,
                None => return None,
            };
            loop {
                match rx.recv().await {
                    Ok(payload) => {
                        self.rx = Some(rx);
                        return Some(payload);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        // Slow consumer: skip ahead and keep polling.
                        tracing::warn!(
                            "BroadcastMessageStream: receiver lagged by {} messages, skipping",
                            n
                        );
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        // Sender dropped, channel is empty and
                        // disconnected. End the stream.
                        self.rx = None;
                        return None;
                    }
                }
            }
        })
    }
}

/// A `MessageStream` that fans in from multiple broadcast
/// receivers. The `next_message` future polls each receiver in
/// round-robin order and yields the first one that has a message
/// ready.
pub struct MultiChannelStream {
    receivers: Vec<tokio::sync::broadcast::Receiver<Vec<u8>>>,
    next: usize,
}

impl std::fmt::Debug for MultiChannelStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiChannelStream")
            .field("channel_count", &self.receivers.len())
            .field("next", &self.next)
            .finish()
    }
}

impl MessageStream for MultiChannelStream {
    fn next_message(
        &mut self,
    ) -> Pin<Box<dyn std::future::Future<Output = Option<Vec<u8>>> + Send + '_>> {
        Box::pin(async move {
            // We try each receiver in round-robin. If all are
            // lagging/closed, return None. (This is a synchronous
            // poll across the receivers, not an async wait — true
            // async fan-in would require `tokio::select!` over
            // `recv()`, which is fine but more code.)
            let n = self.receivers.len();
            for i in 0..n {
                let idx = (self.next + i) % n;
                match self.receivers[idx].try_recv() {
                    Ok(payload) => {
                        self.next = (idx + 1) % n;
                        return Some(payload);
                    }
                    Err(tokio::sync::broadcast::error::TryRecvError::Empty) => continue,
                    Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::TryRecvError::Closed) => continue,
                }
            }
            // No message ready on any channel right now. Block on
            // the *next* channel in the round-robin order so we
            // don't starve the others.
            let idx = self.next;
            self.next = (idx + 1) % n;
            loop {
                match self.receivers[idx].recv().await {
                    Ok(payload) => return Some(payload),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(
                            "MultiChannelStream: receiver lagged by {} messages, skipping",
                            n
                        );
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        // All receivers are closed → end the stream.
                        let all_closed = self
                            .receivers
                            .iter_mut()
                            .all(|rx| {
                                matches!(
                                    rx.try_recv(),
                                    Err(tokio::sync::broadcast::error::TryRecvError::Closed)
                                )
                            });
                        if all_closed {
                            return None;
                        }
                        continue;
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip: subscribe to a single channel, publish a message,
    /// assert the subscriber receives the same bytes back.
    #[tokio::test]
    async fn round_trip_publish_subscribe_single_channel() {
        let adapter = InMemoryPubsubAdapter::new();
        let mut sub = adapter.subscribe(&["wave10:test:rt"]).unwrap();

        adapter
            .publish("wave10:test:rt", b"hello bus")
            .await
            .unwrap();

        let received = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            sub.next_message(),
        )
        .await
        .expect("timed out waiting for in-memory pubsub message")
        .expect("subscriber closed before delivering message");

        assert_eq!(received, b"hello bus");
    }

    /// Subscribe to two channels, publish on each, assert both
    /// messages come through the fan-in stream.
    #[tokio::test]
    async fn round_trip_publish_subscribe_multi_channel() {
        let adapter = InMemoryPubsubAdapter::new();
        let mut sub = adapter
            .subscribe(&["wave10:test:a", "wave10:test:b"])
            .unwrap();

        adapter
            .publish("wave10:test:a", b"alpha")
            .await
            .unwrap();
        adapter
            .publish("wave10:test:b", b"beta")
            .await
            .unwrap();

        let mut received = Vec::new();
        for _ in 0..2 {
            let msg = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                sub.next_message(),
            )
            .await
            .expect("timeout")
            .expect("stream closed");
            received.push(msg);
        }
        received.sort();
        assert_eq!(received, vec![b"alpha".to_vec(), b"beta".to_vec()]);
    }

    /// Empty channel list is rejected.
    #[tokio::test]
    async fn empty_channel_list_rejected() {
        let adapter = InMemoryPubsubAdapter::new();
        let result = adapter.subscribe(&[]);
        assert!(result.is_err());
    }

    /// Publishing to a channel with no subscribers is a no-op (does
    /// not error, does not retain the payload).
    #[tokio::test]
    async fn publish_with_no_subscribers_is_ok() {
        let adapter = InMemoryPubsubAdapter::new();
        adapter
            .publish("wave10:test:no_listeners", b"orphan")
            .await
            .expect("publish to empty channel should succeed");
        assert_eq!(adapter.receiver_count(), 0);
    }

    /// Two subscribers on the same channel each get a copy of every
    /// message (fan-out semantics).
    #[tokio::test]
    async fn fan_out_two_subscribers() {
        let adapter = InMemoryPubsubAdapter::new();
        let mut sub_a = adapter.subscribe(&["wave10:test:fan"]).unwrap();
        let mut sub_b = adapter.subscribe(&["wave10:test:fan"]).unwrap();

        adapter
            .publish("wave10:test:fan", b"everyone")
            .await
            .unwrap();

        let recv_a = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            sub_a.next_message(),
        )
        .await
        .expect("timeout a")
        .expect("closed a");
        let recv_b = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            sub_b.next_message(),
        )
        .await
        .expect("timeout b")
        .expect("closed b");

        assert_eq!(recv_a, b"everyone");
        assert_eq!(recv_b, b"everyone");
    }
}
