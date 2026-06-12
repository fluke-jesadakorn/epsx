//! `NotificationPort` — cross-cutting port for sending wallet notifications.
//!
//! Wave 10 service-boundary refactor. Replaces the 8 direct
//! `NotificationService::send` / `NotificationService::broadcast` publisher
//! call sites with a port that the in-process adapter (current behavior)
//! and the future `epsx-notifications` HTTP service adapter (post-split)
//! both implement.
//!
//! ## Why a port
//!
//! The 8 publishers (payments, chat, admin permissions, plan-expiration)
//! currently call `NotificationService` directly via static methods. After
//! notifications is lifted into a microservice, those calls would have to
//! go over the network. A port lets the in-process implementation be
//! swapped for an HTTP implementation without touching a single caller.
//!
//! ## Stateless design
//!
//! The port takes only `&self, req`. The adapter owns the resources
//! (DB pool, broadcaster, HTTP client) it needs to fulfill the
//! request. This is the contract that the HTTP impl in the
//! integration gate will rely on — it cannot import the
//! application-layer `AppState` over a network boundary.
//!
//! ## DTO design
//!
//! `SendNotificationRequest` / `BroadcastNotificationRequest` are
//! `Serialize` / `Deserialize` so the HTTP impl in the integration gate
//! can deserialize them. The current in-process impl uses them as plain
//! value objects (no serde cost because they never cross a network).
//!
//! ## Object safety
//!
//! The trait is `Send + Sync` and uses `async_trait` so it can be held
//! as `Arc<dyn NotificationPort>` in `AppState` and passed across the
//! async runtime. The `#[cfg(test)]` block in this file includes a
//! compile-time object-safety check.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::AppResult;

/// The string tag for a notification's category (payment, chat, permission, ...).
///
/// String-based on purpose: the notifications DDD layer keeps the typed
/// `NotificationType` enum, but the port speaks across bounded contexts
/// where the calling domain does not import the notifications enum.
pub type NotificationTypeTag = String;

/// The string tag for a notification's priority.
///
/// Same rationale as `NotificationTypeTag`.
pub type NotificationPriorityTag = String;

/// Request body for a single-recipient notification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SendNotificationRequest {
    /// Lower-cased recipient wallet address.
    pub recipient_wallet_address: String,
    /// Notification type tag (e.g. `"payment"`, `"chat"`, `"permission"`).
    pub notification_type: NotificationTypeTag,
    /// Priority tag (e.g. `"low"`, `"normal"`, `"high"`, `"critical"`).
    pub priority: NotificationPriorityTag,
    /// Human-readable title.
    pub title: String,
    /// Human-readable body / message.
    pub message: String,
    /// Optional structured payload (rendered in the admin / client UIs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Optional deep-link / action URL the client should navigate to on click.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_url: Option<String>,
}

/// Request body for a broadcast (all-wallet) notification.
///
/// On the wire this maps to the same `notification_type='all'` row that
/// the current in-process admin broadcast uses. The single-wallet
/// `send` method does NOT cover the "all" case — broadcast is its own
/// method so the in-process adapter can write the row with
/// `wallet_address = "all"` and the HTTP adapter can choose a
/// different broadcast semantics if it wants to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BroadcastNotificationRequest {
    pub notification_type: NotificationTypeTag,
    pub priority: NotificationPriorityTag,
    pub title: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Port that every notifications publisher (payments, chat, admin
/// permissions, plan expiration) calls instead of
/// `NotificationService::send` / `NotificationService::broadcast`.
///
/// Two impls are expected to live in the monorepo:
///
/// 1. `InProcessNotificationAdapter` (in
///    `apps/backend/src/infrastructure/adapters/notification/in_process_adapter.rs`):
///    current behavior — persists to `wallet_notifications` and
///    publishes via the in-process `RedisNotificationBroadcaster`.
/// 2. `HttpNotificationAdapter` (added in the integration gate, not
///    part of this track): forwards to the new `epsx-notifications`
///    microservice over HTTP.
///
/// Implementations are `Send + Sync` so they can be held as
/// `Arc<dyn NotificationPort>` in `AppState` and shared across the
/// async runtime.
#[async_trait]
pub trait NotificationPort: Send + Sync {
    /// Persist + real-time-deliver a notification to a single wallet.
    ///
    /// Returns the notification's UUID string on success. The
    /// in-process adapter returns the freshly-minted ID; the HTTP
    /// adapter returns whatever the remote service returned.
    async fn send(&self, req: SendNotificationRequest) -> AppResult<String>;

    /// Persist + real-time-deliver a broadcast notification.
    ///
    /// The wallet address on the persisted row is conventionally
    /// `"all"`; the HTTP adapter may choose to skip persistence and
    /// rely on a Redis fanout.
    async fn broadcast(&self, req: BroadcastNotificationRequest) -> AppResult<()>;
}

// =============================================================================
// SMOKE / OBJECT-SAFETY TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time guarantee that `NotificationPort` is object-safe.
    /// If a future refactor adds a generic or `Self`-typed method that
    /// breaks object safety, this function will stop compiling.
    #[allow(dead_code)]
    fn _assert_object_safe(_: &dyn NotificationPort) {}

    /// Round-trip the DTOs through serde. The HTTP adapter in the
    /// integration gate will rely on this, so guard it here.
    #[test]
    fn send_request_serde_round_trip() {
        let req = SendNotificationRequest {
            recipient_wallet_address: "0xabc".to_string(),
            notification_type: "payment".to_string(),
            priority: "high".to_string(),
            title: "Credits Received".to_string(),
            message: "You received 100 credits".to_string(),
            data: Some(serde_json::json!({ "amount": 100 })),
            action_url: Some("/plans".to_string()),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let back: SendNotificationRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(req, back);
    }

    #[test]
    fn broadcast_request_serde_round_trip() {
        let req = BroadcastNotificationRequest {
            notification_type: "system".to_string(),
            priority: "critical".to_string(),
            title: "Maintenance".to_string(),
            message: "Down for 10 min".to_string(),
            data: Some(serde_json::json!({ "window": "tonight" })),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let back: BroadcastNotificationRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(req, back);
    }

    /// Guard: the error type stays compatible with `AppError`-based callers.
    #[test]
    fn app_error_is_returned_by_port() {
        let err: crate::errors::AppError = crate::errors::AppError::configuration_error("test");
        let _: AppResult<String> = Err(err);
    }
}
