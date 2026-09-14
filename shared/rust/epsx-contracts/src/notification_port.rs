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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::{AppResult, ErrorKind};

const PRODUCER_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(25);

fn retryable_notification_error(error: &crate::errors::AppError) -> bool {
    matches!(
        error.kind,
        ErrorKind::NetworkError
            | ErrorKind::DatabaseError
            | ErrorKind::ExternalServiceError
            | ErrorKind::ServiceUnavailable
            | ErrorKind::TimeoutError
            | ErrorKind::ResourceExhausted
    )
}

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
    /// Optional expiry shared by the legacy and extracted notification paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
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
    /// Optional expiry shared by the legacy and extracted notification paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
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
/// 2. `HttpNotificationAdapter` now forwards publisher events over the opt-in
///    remote path to the `epsx-notifications` microservice over HTTP.
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

    /// Persist a single-recipient notification using a producer-owned stable
    /// event identity. Retries of the same logical source event must reuse
    /// this value so the remote inbox/idempotency boundary can deduplicate
    /// them. Implementations that do not cross a network may use the default
    /// behavior because their durable write already owns the local identity.
    async fn send_with_event_id(
        &self,
        _event_id: &str,
        req: SendNotificationRequest,
    ) -> AppResult<String> {
        self.send(req).await
    }

    /// Retry only infrastructure failures with the same producer-owned event
    /// identity. The extracted service treats that identity as idempotent, so
    /// a timeout after remote admission cannot create a second logical row.
    /// Validation, authorization, conflicts, and other permanent failures
    /// return immediately.
    async fn send_with_event_id_retry(
        &self,
        event_id: &str,
        req: SendNotificationRequest,
    ) -> AppResult<String> {
        match self.send_with_event_id(event_id, req.clone()).await {
            Ok(value) => Ok(value),
            Err(error) if retryable_notification_error(&error) => {
                tokio::time::sleep(PRODUCER_RETRY_DELAY).await;
                self.send_with_event_id(event_id, req).await
            }
            Err(error) => Err(error),
        }
    }

    /// Persist + real-time-deliver a broadcast notification.
    ///
    /// The wallet address on the persisted row is conventionally
    /// `"all"`; the HTTP adapter may choose to skip persistence and
    /// rely on a Redis fanout.
    async fn broadcast(&self, req: BroadcastNotificationRequest) -> AppResult<()>;

    /// Broadcast equivalent of [`NotificationPort::send_with_event_id`].
    async fn broadcast_with_event_id(
        &self,
        _event_id: &str,
        req: BroadcastNotificationRequest,
    ) -> AppResult<()> {
        self.broadcast(req).await
    }

    /// Broadcast counterpart to the single-recipient retry helper.
    async fn broadcast_with_event_id_retry(
        &self,
        event_id: &str,
        req: BroadcastNotificationRequest,
    ) -> AppResult<()> {
        match self.broadcast_with_event_id(event_id, req.clone()).await {
            Ok(()) => Ok(()),
            Err(error) if retryable_notification_error(&error) => {
                tokio::time::sleep(PRODUCER_RETRY_DELAY).await;
                self.broadcast_with_event_id(event_id, req).await
            }
            Err(error) => Err(error),
        }
    }
}

// =============================================================================
// SMOKE / OBJECT-SAFETY TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

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
            expires_at: None,
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
            expires_at: None,
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

    struct RetryProbe {
        single_attempts: AtomicUsize,
        broadcast_attempts: AtomicUsize,
    }

    #[async_trait]
    impl NotificationPort for RetryProbe {
        async fn send(&self, _req: SendNotificationRequest) -> AppResult<String> {
            Ok("legacy".into())
        }

        async fn send_with_event_id(
            &self,
            event_id: &str,
            _req: SendNotificationRequest,
        ) -> AppResult<String> {
            let attempt = self.single_attempts.fetch_add(1, Ordering::SeqCst);
            if attempt == 0 {
                Err(crate::errors::AppError::network_error("temporary outage"))
            } else {
                Ok(event_id.into())
            }
        }

        async fn broadcast(&self, _req: BroadcastNotificationRequest) -> AppResult<()> {
            Ok(())
        }

        async fn broadcast_with_event_id(
            &self,
            _event_id: &str,
            _req: BroadcastNotificationRequest,
        ) -> AppResult<()> {
            let attempt = self.broadcast_attempts.fetch_add(1, Ordering::SeqCst);
            if attempt == 0 {
                Err(crate::errors::AppError::new(
                    ErrorKind::TimeoutError,
                    "temporary outage",
                ))
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn producer_retry_reuses_the_same_event_identity_for_transient_failures() {
        let probe = Arc::new(RetryProbe {
            single_attempts: AtomicUsize::new(0),
            broadcast_attempts: AtomicUsize::new(0),
        });
        let single = probe
            .send_with_event_id_retry(
                "payment.completed:retry-1",
                SendNotificationRequest {
                    recipient_wallet_address: "0x1111111111111111111111111111111111111111".into(),
                    notification_type: "payment".into(),
                    priority: "normal".into(),
                    title: "Payment".into(),
                    message: "Accepted".into(),
                    data: None,
                    action_url: None,
                    expires_at: None,
                },
            )
            .await
            .expect("transient single-recipient failure should retry");
        assert_eq!(single, "payment.completed:retry-1");
        assert_eq!(probe.single_attempts.load(Ordering::SeqCst), 2);

        probe
            .broadcast_with_event_id_retry(
                "notification.broadcast:retry-1",
                BroadcastNotificationRequest {
                    notification_type: "system".into(),
                    priority: "normal".into(),
                    title: "Maintenance".into(),
                    message: "Accepted".into(),
                    data: None,
                    expires_at: None,
                },
            )
            .await
            .expect("transient broadcast failure should retry");
        assert_eq!(probe.broadcast_attempts.load(Ordering::SeqCst), 2);
    }
}
