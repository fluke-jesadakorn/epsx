//! Wave 10 end-to-end smoke test — port + pubsub runtime integration.
//!
//! This is the integration gate's runtime check. It confirms that
//! Track A's `NotificationPort` trait and Track B's `PubsubPort`
//! trait line up at runtime (not just at compile time). The chat
//! pubsub canary in `apps/backend/src/infrastructure/adapters/pubsub/mod.rs`
//! tests pubsub-only; this test exercises the seam between the
//! notification port's "publish SSE" call and the pubsub port's
//! `publish(channel, payload)` contract.
//!
//! ## What it tests
//!
//! 1. A `PubsubPort` (in-memory adapter) is created.
//! 2. A `NotificationPort` is constructed that publishes through
//!    the same `PubsubPort` (mirroring `InProcessNotificationAdapter`'s
//!    `publish_sse` — channel names `notifications:wallet:<addr>` and
//!    `notifications:all`, JSON payload of an `SSENotification`).
//! 3. A subscriber on the same channels receives the messages that
//!    the port publishes.
//!
//! If this test fails, the integration is broken — the port seam
//! the integration depends on (Track A's port publishes via
//! Track B's pubsub, with a specific channel name + payload
//! convention) does not actually work end-to-end.
//!
//! ## Why a custom port and not `InProcessNotificationAdapter`?
//!
//! `InProcessNotificationAdapter::try_new` requires a real Postgres
//! pool (the env-var check is a hard fail), and `from_pool` requires
//! `Arc<&'static TlsPool>` which is hard to construct in a unit
//! context. The integration truth is the *seam* between the port
//! and the pubsub — not the DB write. A port that mirrors the
//! in-process adapter's `publish_sse` channel-name + payload
//! convention is sufficient to confirm the seam. The
//! `in_process_adapter::tests::port_trait_send_round_trip` unit
//! test confirms the trait-DTO contract; this integration test
//! confirms the runtime channel/payload contract.

use std::sync::Arc;

use async_trait::async_trait;
use epsx::infrastructure::adapters::pubsub::InMemoryPubsubAdapter;
use epsx_contracts::errors::AppResult;
use epsx_contracts::notification_port::{
    BroadcastNotificationRequest, NotificationPort, SendNotificationRequest,
};
use epsx_contracts::pubsub_port::PubsubPort;
use serde::Serialize;
use uuid::Uuid;

/// Channel-name convention used by `InProcessNotificationAdapter`.
/// Public re-statement so the smoke test does not need to import
/// private internals from the in-process adapter.
const CHANNEL_BROADCAST: &str = "notifications:all";
fn channel_for_wallet(wallet: &str) -> String {
    format!("notifications:wallet:{}", wallet.to_lowercase())
}

/// Minimal SSE payload — same shape as
/// `crate::web::notifications::SSENotification` (the production
/// type). We don't import the production type because it lives
/// behind a private module path; the integration truth is "JSON
/// payload with at least these fields" — the actual struct may
/// grow in the future and the subscriber should be tolerant.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
struct SmokeSseNotification {
    id: String,
    wallet_address: String,
    title: String,
    message: String,
    priority: String,
}

/// Minimal in-process `NotificationPort` that mirrors
/// `InProcessNotificationAdapter`'s `publish_sse` channel-name +
/// JSON-payload contract. This is the integration test's stand-in
/// for the production in-process adapter.
struct SmokeNotificationPort {
    pubsub: Arc<dyn PubsubPort>,
}

#[async_trait]
impl NotificationPort for SmokeNotificationPort {
    async fn send(&self, req: SendNotificationRequest) -> AppResult<String> {
        let id = Uuid::new_v4().to_string();
        let wallet = req.recipient_wallet_address.to_lowercase();
        let sse = SmokeSseNotification {
            id: id.clone(),
            wallet_address: wallet.clone(),
            title: req.title,
            message: req.message,
            priority: req.priority,
        };
        let payload = serde_json::to_vec(&sse)
            .map_err(|e| epsx_contracts::errors::AppError::new(
                epsx_contracts::errors::ErrorKind::InternalError,
                format!("smoke port: serialize SSE: {}", e),
            ))?;
        // Fire-and-forget, like the production in-process adapter.
        self.pubsub
            .publish(&channel_for_wallet(&wallet), &payload)
            .await?;
        Ok(id)
    }

    async fn broadcast(&self, req: BroadcastNotificationRequest) -> AppResult<()> {
        let id = Uuid::new_v4().to_string();
        let sse = SmokeSseNotification {
            id: id.clone(),
            wallet_address: "all".to_string(),
            title: req.title,
            message: req.message,
            priority: req.priority,
        };
        let payload = serde_json::to_vec(&sse)
            .map_err(|e| epsx_contracts::errors::AppError::new(
                epsx_contracts::errors::ErrorKind::InternalError,
                format!("smoke port: serialize SSE: {}", e),
            ))?;
        self.pubsub.publish(CHANNEL_BROADCAST, &payload).await?;
        Ok(())
    }
}

#[tokio::test]
async fn wave10_send_publishes_to_wallet_channel_via_pubsub() {
    // 1. Stand up the in-memory pubsub.
    let pubsub: Arc<dyn PubsubPort> = Arc::new(InMemoryPubsubAdapter::new());

    // 2. Stand up the notification port pointing at it.
    let port: Arc<dyn NotificationPort> =
        Arc::new(SmokeNotificationPort { pubsub: pubsub.clone() });

    // 3. Subscribe to the wallet channel before publishing.
    let mut stream = pubsub
        .subscribe(&["notifications:wallet:0xabc"])
        .expect("subscribe to wallet channel");

    // 4. Call send() on the port.
    let id = port
        .send(SendNotificationRequest {
            recipient_wallet_address: "0xAbC".to_string(), // mixed case — channel is lowercased
            notification_type: "payment".to_string(),
            priority: "high".to_string(),
            title: "Credits Received".to_string(),
            message: "You received 100 credits".to_string(),
            data: Some(serde_json::json!({ "amount": 100 })),
            action_url: None,
        })
        .await
        .expect("port.send should succeed");
    assert!(!id.is_empty(), "port.send should return a non-empty id");

    // 5. The subscriber receives the JSON payload.
    let payload = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next_message())
        .await
        .expect("subscriber must receive a message within 2s")
        .expect("subscriber stream returned Some(payload)");

    let sse: SmokeSseNotification = serde_json::from_slice(&payload)
        .expect("subscriber must be able to deserialize the JSON SSE payload");
    assert_eq!(sse.wallet_address, "0xabc", "wallet should be lowercased");
    assert_eq!(sse.title, "Credits Received");
    assert_eq!(sse.priority, "high");
    assert_eq!(sse.id, id, "the SSE id should match the port's return value");
}

#[tokio::test]
async fn wave10_broadcast_publishes_to_all_channel_via_pubsub() {
    let pubsub: Arc<dyn PubsubPort> = Arc::new(InMemoryPubsubAdapter::new());
    let port: Arc<dyn NotificationPort> =
        Arc::new(SmokeNotificationPort { pubsub: pubsub.clone() });

    let mut stream = pubsub
        .subscribe(&[CHANNEL_BROADCAST])
        .expect("subscribe to broadcast channel");

    port.broadcast(BroadcastNotificationRequest {
        notification_type: "system".to_string(),
        priority: "normal".to_string(),
        title: "Maintenance Window".to_string(),
        message: "Service will be down for 10 minutes".to_string(),
        data: Some(serde_json::json!({ "window": "tonight" })),
    })
    .await
    .expect("port.broadcast should succeed");

    let payload = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next_message())
        .await
        .expect("subscriber must receive a message within 2s")
        .expect("subscriber stream returned Some(payload)");

    let sse: SmokeSseNotification = serde_json::from_slice(&payload)
        .expect("subscriber must be able to deserialize the JSON SSE payload");
    assert_eq!(sse.wallet_address, "all");
    assert_eq!(sse.title, "Maintenance Window");
}

#[tokio::test]
async fn wave10_wallet_subscription_does_not_receive_broadcasts() {
    // Two subscribers on different channels must not see each
    // other's messages — this is the chat-pubsub-canary-equivalent
    // for the notification port.
    let pubsub: Arc<dyn PubsubPort> = Arc::new(InMemoryPubsubAdapter::new());
    let port: Arc<dyn NotificationPort> =
        Arc::new(SmokeNotificationPort { pubsub: pubsub.clone() });

    let mut wallet_stream = pubsub
        .subscribe(&["notifications:wallet:0xdef"])
        .expect("subscribe to wallet");
    let mut broadcast_stream = pubsub
        .subscribe(&[CHANNEL_BROADCAST])
        .expect("subscribe to broadcast");

    port.send(SendNotificationRequest {
        recipient_wallet_address: "0xdef".to_string(),
        notification_type: "chat".to_string(),
        priority: "normal".to_string(),
        title: "New chat message".to_string(),
        message: "Hello!".to_string(),
        data: None,
        action_url: None,
    })
    .await
    .expect("send");

    // The wallet subscriber gets the message.
    let _wallet_payload = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        wallet_stream.next_message(),
    )
    .await
    .expect("wallet subscriber must receive")
    .expect("wallet stream is open");

    // The broadcast subscriber does NOT — it has its own channel.
    // We give the broadcast subscriber a short window to demonstrate
    // that no message arrives. (2s is generous; the in-memory
    // adapter is synchronous from the publisher's perspective.)
    let res = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        broadcast_stream.next_message(),
    )
    .await;
    assert!(
        res.is_err(),
        "broadcast subscriber must NOT receive wallet-channel messages (timeout means no message arrived, which is what we want)"
    );
}
