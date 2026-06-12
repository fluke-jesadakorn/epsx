// Pub/sub adapters — implementations of `epsx_contracts::pubsub_port::PubsubPort`.
//
// Two implementations live here:
//
//   - `redis_pubsub_adapter` (production): wraps the existing
//     `redis::Client` + `redis::aio::PubSub` pair. The same connection
//     settings as `apps/backend/src/infrastructure/redis/pool.rs` are
//     reused — the new adapter does *not* duplicate env config.
//
//   - `in_memory_pubsub_adapter` (tests): wraps a `tokio::sync::broadcast`
//     bus so chat-handler tests and the round-trip integration test
//     can exercise the port without a Redis instance.
//
// The 8+ chat call sites and the notification SSE handlers in
// `web/notifications/sse_handlers.rs` and `web/user/chat_handlers.rs`
// and `web/admin/chat_handlers.rs` consume these adapters through
// `Arc<dyn PubsubPort>`. See `web/auth/app_state.rs` for the wiring.

pub mod in_memory_pubsub_adapter;
pub mod redis_pubsub_adapter;

pub use in_memory_pubsub_adapter::InMemoryPubsubAdapter;
pub use redis_pubsub_adapter::RedisPubsubAdapter;
