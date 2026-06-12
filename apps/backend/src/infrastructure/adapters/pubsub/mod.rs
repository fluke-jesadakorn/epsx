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

// ============================================================================
// Chat pubsub canary test
// ============================================================================
//
// `docs/wave8-service-boundary/audit-notifications.md` §3c / §3d flagged
// that the chat SSE stream at `/api/chat/stream` and the admin chat
// stream at `/api/admin/chat/stream` both depend on
// `RedisNotificationBroadcaster` for fanout on channels like `chat:new`,
// `chat:agent:<id>`, `chat:wallet:<addr>`. If the broadcaster hoist
// (wave-10 R2) is wrong, chat goes down.
//
// This canary test simulates the canary scenario at the port level:
//   1. Stand up an in-memory PubsubPort (the test double for the
//      production Redis adapter).
//   2. Subscribe to `chat:new` (the channel the chat handler
//      `web/user/chat_handlers.rs:77-90` publishes to when a new
//      conversation is created).
//   3. Publish a "new_conversation" event on `chat:new` — exactly the
//      publish the chat handler does (modulo the wallet address).
//   4. Assert the subscriber receives the same payload, with all the
//      fields a real admin SSE client would need (conversation_id,
//      type, wallet_address, subject).
//
// This is the test the audit said the refactor must pass. If
// the PubsubPort's `publish` / `subscribe` round-trip breaks for
// chat, this test breaks.
#[cfg(test)]
mod chat_pubsub_canary_tests {
    use super::*;
    use epsx_contracts::pubsub_port::{MessageStream, PubsubPort};
    use std::sync::Arc;

    /// The audit-flagged canary: publish on `chat:new`, subscribe
    /// on `chat:new`, assert the message round-trips.
    #[tokio::test]
    async fn chat_new_round_trip_via_pubsub_port() {
        let port: Arc<dyn PubsubPort> = Arc::new(InMemoryPubsubAdapter::new());

        // Simulate a chat SSE client connecting to /api/chat/stream
        // — it subscribes to `chat:new` to receive new-conversation
        // events from any backend instance.
        let mut subscriber = port
            .subscribe(&["chat:new"])
            .expect("subscribe to chat:new");

        // Simulate a backend instance publishing a new-conversation
        // event, exactly as web/user/chat_handlers.rs:77-90 does.
        let event = serde_json::json!({
            "type": "new_conversation",
            "conversation_id": "0d4f8a90-1234-5678-9abc-def012345678",
            "wallet_address": "0xabcdef0123456789abcdef0123456789abcdef01",
            "subject": "Cannot access my wallet",
        });
        let payload = serde_json::to_vec(&event).expect("serialize event");

        port.publish("chat:new", &payload)
            .await
            .expect("publish to chat:new");

        let received = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            subscriber.next_message(),
        )
        .await
        .expect("timed out waiting for chat:new event")
        .expect("subscriber closed before delivering message");

        // Decode and assert the round-trip preserves the chat event
        // shape — this is what a real SSE client would receive.
        let decoded: serde_json::Value =
            serde_json::from_slice(&received).expect("decode chat event JSON");
        assert_eq!(decoded["type"], "new_conversation");
        assert_eq!(
            decoded["conversation_id"],
            "0d4f8a90-1234-5678-9abc-def012345678"
        );
        assert_eq!(
            decoded["wallet_address"],
            "0xabcdef0123456789abcdef0123456789abcdef01"
        );
        assert_eq!(decoded["subject"], "Cannot access my wallet");
    }

    /// Multi-channel canary: an admin SSE client subscribes to both
    /// `chat:new` (broadcast) and `chat:agent:<wallet>` (per-agent).
    /// A backend instance publishes on both channels; both messages
    /// must arrive.
    #[tokio::test]
    async fn admin_chat_multi_channel_round_trip() {
        let port: Arc<dyn PubsubPort> = Arc::new(InMemoryPubsubAdapter::new());

        let mut subscriber = port
            .subscribe(&["chat:new", "chat:agent:0xagent1"])
            .expect("subscribe to multi-channel");

        // Publish a new-conversation broadcast (admin channel).
        let broadcast = serde_json::json!({
            "type": "new_conversation",
            "conversation_id": "conv-1",
        });
        port.publish("chat:new", &serde_json::to_vec(&broadcast).unwrap())
            .await
            .unwrap();

        // Publish a per-agent notification.
        let agent_msg = serde_json::json!({
            "type": "new_message",
            "conversation_id": "conv-1",
            "message": "Hello agent",
        });
        port.publish(
            "chat:agent:0xagent1",
            &serde_json::to_vec(&agent_msg).unwrap(),
        )
        .await
        .unwrap();

        // Read two messages — order is not guaranteed.
        let mut received = Vec::new();
        for _ in 0..2 {
            let msg = tokio::time::timeout(
                std::time::Duration::from_secs(1),
                subscriber.next_message(),
            )
            .await
            .expect("timeout")
            .expect("stream closed");
            received.push(serde_json::from_slice::<serde_json::Value>(&msg).unwrap());
        }
        received.sort_by_key(|v| v["type"].as_str().unwrap_or("").to_string());
        assert_eq!(received[0]["type"], "new_conversation");
        assert_eq!(received[1]["type"], "new_message");
    }
}
