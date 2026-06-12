//! Redis implementation of `PubsubPort`.
//!
//! ## What this file replaces
//!
//! The notification-specific wrapper in
//! `apps/backend/src/web/notifications/redis_broadcaster.rs` is being
//! decomposed. The generic `publish` / `subscribe` primitives
//! (`publish_to_channel` and `subscribe_to_channel`) move here; the
//! notification-typed wrappers (`publish_to_wallet`,
//! `subscribe_to_wallet`, `publish_to_all`) stay in
//! `redis_broadcaster.rs` as thin adapters that call into the
//! `PubsubPort` methods.
//!
//! ## Connection settings
//!
//! The new adapter does **not** duplicate env configuration. It
//! reuses the existing `RedisPool` (in
//! `apps/backend/src/infrastructure/redis/pool.rs`) and the same
//! `redis_url` env var. The only thing the adapter adds is the
//! `redis::Client` clone it needs to call
//! `client.get_async_pubsub()` — that handle is needed because
//! `redis::aio::PubSub` cannot be created from a
//! `ConnectionManager` (which is what the pool caches for ordinary
//! commands).

use std::pin::Pin;
use std::sync::Arc;

use epsx_contracts::errors::{AppError, ErrorKind};
use epsx_contracts::pubsub_port::{MessageStream, PubsubPort};
use futures::StreamExt;
use redis::AsyncCommands;

use crate::infrastructure::redis::RedisPool;

/// The Redis pub/sub adapter.
///
/// Holds:
///   - a `redis::Client` clone so we can mint fresh `PubSub` connections
///     (one per `subscribe` call);
///   - an `Arc<RedisPool>` so the publish path can re-use the shared
///     `ConnectionManager` (the same one the cache and other Redis
///     commands use).
#[derive(Clone)]
pub struct RedisPubsubAdapter {
    client: redis::Client,
    pool: Arc<RedisPool>,
}

impl std::fmt::Debug for RedisPubsubAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisPubsubAdapter")
            .field("client", &"<redis::Client>")
            .field("pool", &"<RedisPool>")
            .finish()
    }
}

impl RedisPubsubAdapter {
    /// Create a new adapter from a Redis URL. The URL is parsed once
    /// and the underlying `Client` is cloned cheaply for the lifetime
    /// of the adapter.
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        // Use the same URL to build a RedisPool for the publish path.
        // We use `block_in_place` to bridge sync `Client::open` with
        // the async pool constructor. The pool internally calls
        // `client.get_connection_manager().await`, so we cannot avoid
        // the runtime hop.
        let pool = futures::executor::block_on(RedisPool::new(redis_url))?;
        Ok(Self {
            client,
            pool: Arc::new(pool),
        })
    }

    /// Wrap an existing `Arc<RedisPool>` together with a redis
    /// `Client` handle. Used by the DI container in
    /// `infrastructure/container/simple_container.rs` which already
    /// has a `RedisPool` Arc in hand.
    pub fn from_pool_and_client(client: redis::Client, pool: Arc<RedisPool>) -> Self {
        Self { client, pool }
    }

    /// Returns the inner pool so existing call sites (the cache,
    /// permission cache, etc.) can share the same connection.
    pub fn pool(&self) -> &Arc<RedisPool> {
        &self.pool
    }

    /// The redis `Client` handle used to mint fresh `PubSub`
    /// connections.
    pub fn client(&self) -> &redis::Client {
        &self.client
    }
}

#[allow(async_fn_in_trait)]
impl PubsubPort for RedisPubsubAdapter {
    async fn publish(&self, channel: &str, payload: &[u8]) -> Result<(), AppError> {
        let mut conn = self.pool.get_connection();
        let subscriber_count: i32 = conn
            .publish(channel, payload)
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::InternalError,
                    format!("Redis publish to {} failed: {}", channel, e),
                )
            })?;

        tracing::debug!(
            "RedisPubsubAdapter: published to channel={} subscribers={} bytes={}",
            channel,
            subscriber_count,
            payload.len()
        );
        Ok(())
    }

    fn subscribe(
        &self,
        channels: &[&str],
    ) -> Result<Box<dyn MessageStream + Send + Unpin>, AppError> {
        if channels.is_empty() {
            return Err(AppError::new(
                ErrorKind::ValidationError,
                "RedisPubsubAdapter::subscribe requires at least one channel",
            ));
        }

        // Mint a fresh PubSub connection for this subscription. The
        // PubSub cannot be created from a ConnectionManager (it owns
        // its own dedicated connection), so we have to go back to the
        // underlying redis::Client.
        let client = self.client.clone();
        let channels_owned: Vec<String> = channels.iter().map(|c| c.to_string()).collect();

        // Open the PubSub, subscribe to every requested channel, and
        // *consume* the PubSub so we get the concrete `PubSubStream`
        // type back (via `into_on_message()`). The block_on is OK
        // because `subscribe` is not async and the connect+SUBSCRIBE
        // roundtrip is bounded by the redis timeout.
        let stream = futures::executor::block_on(async {
            let mut ps = client
                .get_async_pubsub()
                .await
                .map_err(|e| {
                    AppError::new(
                        ErrorKind::InternalError,
                        format!("Redis pubsub connection failed: {}", e),
                    )
                })?;
            for ch in &channels_owned {
                ps.subscribe(ch.as_str()).await.map_err(|e| {
                    AppError::new(
                        ErrorKind::InternalError,
                        format!("Redis subscribe to {} failed: {}", ch, e),
                    )
                })?;
            }
            Ok::<_, AppError>(ps.into_on_message())
        })?;

        Ok(Box::new(RedisMessageStream { inner: stream }))
    }
}

/// Adapter that wraps `redis::aio::PubSubStream` as a `MessageStream`.
///
/// The redis crate's `PubSubStream` is itself a `Stream<Item = Msg>`,
/// so we can drive `next_message` by `poll_next`-ing it through a
/// `Pin<Box<dyn Future>>` shell.
pub struct RedisMessageStream {
    inner: redis::aio::PubSubStream,
}

impl std::fmt::Debug for RedisMessageStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisMessageStream").finish()
    }
}

impl MessageStream for RedisMessageStream {
    fn next_message(
        &mut self,
    ) -> Pin<Box<dyn std::future::Future<Output = Option<Vec<u8>>> + Send + '_>> {
        let stream = &mut self.inner;
        Box::pin(async move {
            match stream.next().await {
                Some(msg) => match msg.get_payload() {
                    Ok(payload) => Some(payload),
                    Err(e) => {
                        tracing::warn!(
                            "RedisMessageStream: failed to get payload from redis message: {}",
                            e
                        );
                        // Treat a bad payload as an end-of-stream —
                        // the SSE handler skips it and continues.
                        None
                    }
                },
                None => None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    // The Redis adapter test is gated behind the `redis-tests` feature
    // because it requires a live Redis instance. The default CI test
    // path uses the in-memory adapter (see
    // `in_memory_pubsub_adapter.rs`).
    //
    // To run the live-Redis tests:
    //   REDIS_URL=redis://127.0.0.1:6379 cargo test -p epsx --lib \
    //     infrastructure::adapters::pubsub::redis_pubsub_adapter \
    //     --features redis-tests -- --include-ignored

    #[cfg(feature = "redis-tests")]
    use super::*;

    #[cfg(feature = "redis-tests")]
    #[tokio::test]
    #[ignore = "requires REDIS_URL with a live redis instance"]
    async fn round_trip_publish_subscribe() {
        let url = std::env::var("REDIS_URL")
            .expect("REDIS_URL env var must be set to run redis-tests feature tests");
        let adapter = RedisPubsubAdapter::new(&url).expect("failed to create adapter");

        let mut sub = adapter.subscribe(&["wave10:test:round_trip"]).unwrap();

        // Publish in a separate task so the subscriber is ready first.
        let adapter_for_publish = adapter.clone();
        let payload = b"hello redis pubsub".to_vec();
        tokio::spawn(async move {
            adapter_for_publish
                .publish("wave10:test:round_trip", &payload)
                .await
                .unwrap();
        });

        let received = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            sub.next_message(),
        )
        .await
        .expect("timed out waiting for redis pubsub message")
        .expect("subscriber closed before delivering message");

        assert_eq!(received, b"hello redis pubsub");
    }
}
