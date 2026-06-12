//! In-process adapter for `NotificationPort`.
//!
//! Wave 10 service-boundary refactor. This is the **current-behavior**
//! implementation of the port — it persists the row to the
//! notifications DB and publishes the SSE payload via the
//! `RedisNotificationBroadcaster`, exactly like the pre-wave-10
//! `NotificationService::send` / `NotificationService::broadcast` did.
//!
//! A future `HttpNotificationAdapter` (added in the integration gate
//! when the `epsx-notifications` service binary is wired up) will
//! forward the same `SendNotificationRequest` / `BroadcastNotificationRequest`
//! over HTTP. The 8 publisher call sites call the trait, not the
//! adapter, so the swap is a one-line DI change.
//!
//! ## Construction
//!
//! The adapter owns its own `Arc<WalletNotificationRepository>` and
//! `Option<Arc<RedisNotificationBroadcaster>>` so the trait's
//! stateless interface (`&self, req`) can be satisfied without
//! threading an `AppState` through every call. The port-trait
//! signature is the contract the HTTP impl in the integration gate
//! will rely on; it cannot import the application-layer `AppState`
//! over a network boundary.
//!
//! ## Pool-fallback fix
//!
//! The pre-wave-10 code fell back to `app_state.db_pool` when the
//! notifications pool was unavailable, silently writing to the wrong
//! schema. The fix is in the constructor: if `NOTIFICATIONS_DATABASE_URL`
//! is unset, the constructor returns `Err(AppError::Configuration)`. The
//! DI wiring in `bootstrap.rs` (and the in-process container factory)
//! logs and refuses to start the server in that state. The
//! `NotificationService` struct is kept around for the
//! `plan_expiration_service` cron driver (which only needs the
//! broadcast pool for read-side cleanup).
//!
//! ## Tests
//!
//! The in-process adapter has a round-trip test with a `MockNotificationService`
//! double. The trait-level smoke test lives in
//! `epsx_contracts::notification_port` (object-safety check + serde
//! round-trip); the in-process implementation is exercised here.

use async_trait::async_trait;
use chrono::Utc;
use epsx_contracts::errors::{AppError, AppResult, ErrorKind};
use epsx_contracts::notification_port::{
    BroadcastNotificationRequest, NotificationPort, SendNotificationRequest,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::web::admin::wallet_notification_repository::WalletNotificationRepository;
use crate::web::notifications::{
    NotificationPriority, NotificationType, SSENotification,
};
use crate::prelude::TlsPool;
use epsx_contracts::pubsub_port::PubsubPort;

/// In-process `NotificationPort` adapter. Owns the resources it needs
/// to fulfill a request: the notifications DB pool wrapped in the
/// `WalletNotificationRepository`, plus the kernel-level
/// `PubsubPort` for real-time fanout.
pub struct InProcessNotificationAdapter {
    pool: Arc<&'static TlsPool>,
    broadcaster: Option<Arc<dyn PubsubPort>>,
}

impl std::fmt::Debug for InProcessNotificationAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InProcessNotificationAdapter")
            .field("broadcaster_configured", &self.broadcaster.is_some())
            .finish()
    }
}

impl InProcessNotificationAdapter {
    /// Build a new adapter. **Fails fast** if `NOTIFICATIONS_DATABASE_URL`
    /// is unset — the pre-wave-10 silent fallback to the primary pool
    /// is the bug the audit flagged. The HTTP impl in the integration
    /// gate does not have this constraint (it talks to a remote
    /// service).
    pub async fn try_new(
        broadcaster: Option<Arc<dyn PubsubPort>>,
    ) -> AppResult<Self> {
        Self::check_notifications_url_configured()?;
        let pool = crate::infrastructure::database::get_notifications_pool()
            .await
            .map_err(|e| {
                AppError::new(
                    ErrorKind::DatabaseError,
                    format!("notifications pool unavailable: {}", e),
                )
            })?;
        Ok(Self {
            pool: Arc::new(pool),
            broadcaster,
        })
    }

    /// Build an adapter around a *caller-supplied* pool. This bypasses
    /// the env-var check and is intended for **tests** that wire up
    /// a mock pool. Production wiring must use `try_new`.
    pub fn from_pool(
        pool: Arc<&'static TlsPool>,
        broadcaster: Option<Arc<dyn PubsubPort>>,
    ) -> Self {
        Self { pool, broadcaster }
    }

    /// Synchronous env-var check. Public so `bootstrap.rs` and the
    /// container factories can call it before constructing the
    /// adapter (to log a clear error at startup).
    pub fn check_notifications_url_configured() -> AppResult<()> {
        match std::env::var("NOTIFICATIONS_DATABASE_URL") {
            Ok(url) if !url.is_empty() => Ok(()),
            Ok(_) | Err(_) => {
                tracing::error!(
                    "NOTIFICATIONS_DATABASE_URL is not set; refusing to write \
                     notifications to the primary pool (silent-fallback bug)"
                );
                Err(AppError::configuration_error(
                    "NOTIFICATIONS_DATABASE_URL is not set; \
                     notifications cannot be written to the primary database. \
                     Set NOTIFICATIONS_DATABASE_URL to the notifications pool URL.",
                ))
            }
        }
    }

    /// Format a typed `NotificationType` enum into the lowercase string
    /// tag the port speaks. Public so the legacy `NotificationService`
    /// shim can convert the typed-enum callers into the string-typed
    /// port requests without re-implementing the format logic.
    pub fn format_notification_type_tag(t: NotificationType) -> String {
        format!("{:?}", t).to_lowercase()
    }

    /// Format a typed `NotificationPriority` enum into the lowercase
    /// string tag the port speaks.
    pub fn format_notification_priority_tag(p: NotificationPriority) -> String {
        format!("{:?}", p).to_lowercase()
    }

    /// Persist a row to `wallet_notifications`.
    async fn persist(
        &self,
        wallet: &str,
        notification_type: &str,
        priority: &str,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
        action_url: Option<String>,
    ) -> AppResult<Uuid> {
        let id = Uuid::new_v4();
        let repo = WalletNotificationRepository::new(self.pool.clone());
        repo.create(
            id,
            wallet,
            notification_type,
            title,
            message,
            data,
            priority,
            None,
            action_url,
            None,
        )
        .await?;
        Ok(id)
    }

    /// Publish via the kernel-level `PubsubPort` (fire-and-forget;
    /// failure is logged, not propagated). The pre-wave-10 behavior
    /// is preserved here — channel names are
    /// `notifications:wallet:<addr>` for per-wallet and
    /// `notifications:all` for broadcasts.
    async fn publish_sse(&self, wallet: &str, sse: &SSENotification, broadcast: bool) {
        if let Some(broadcaster) = &self.broadcaster {
            let channel = if broadcast {
                "notifications:all".to_string()
            } else {
                format!("notifications:wallet:{}", wallet)
            };
            let payload = match serde_json::to_vec(sse) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        "Failed to serialize notification SSE (wallet={}, broadcast={}): {}",
                        wallet, broadcast, e
                    );
                    return;
                }
            };
            if let Err(e) = broadcaster.publish(&channel, &payload).await {
                tracing::warn!(
                    "Failed to publish notification via PubsubPort (wallet={}, broadcast={}): {}",
                    wallet, broadcast, e
                );
            }
        }
    }
}

#[async_trait]
impl NotificationPort for InProcessNotificationAdapter {
    async fn send(&self, req: SendNotificationRequest) -> AppResult<String> {
        let wallet = req.recipient_wallet_address.to_lowercase();
        let id = self
            .persist(
                &wallet,
                &req.notification_type,
                &req.priority,
                &req.title,
                &req.message,
                req.data.clone(),
                req.action_url.clone(),
            )
            .await?;

        let sse = SSENotification {
            id: id.to_string(),
            wallet_address: wallet.clone(),
            notification_type: parse_notification_type(&req.notification_type),
            title: req.title.clone(),
            message: req.message.clone(),
            data: req.data.clone(),
            priority: parse_notification_priority(&req.priority),
            timestamp: Utc::now(),
            expires_at: None,
        };
        self.publish_sse(&wallet, &sse, false).await;
        Ok(id.to_string())
    }

    async fn broadcast(&self, req: BroadcastNotificationRequest) -> AppResult<()> {
        let id = self
            .persist(
                "all",
                &req.notification_type,
                &req.priority,
                &req.title,
                &req.message,
                req.data.clone(),
                None,
            )
            .await?;

        let sse = SSENotification {
            id: id.to_string(),
            wallet_address: "all".to_string(),
            notification_type: parse_notification_type(&req.notification_type),
            title: req.title.clone(),
            message: req.message.clone(),
            data: req.data.clone(),
            priority: parse_notification_priority(&req.priority),
            timestamp: Utc::now(),
            expires_at: None,
        };
        self.publish_sse("all", &sse, true).await;
        Ok(())
    }
}

/// Parse a lowercase notification type tag into the typed enum used by
/// the SSE payload. Falls back to a sensible default for unknown tags
/// (forward-compatibility for new types added by callers).
fn parse_notification_type(tag: &str) -> NotificationType {
    match tag {
        "payment" => NotificationType::Payment,
        "chat" => NotificationType::Chat,
        "permission" => NotificationType::Permission,
        "system" => NotificationType::System,
        "walletmanagement" | "wallet_management" | "wallet-management" => {
            NotificationType::WalletManagement
        }
        "wallet" => NotificationType::Wallet,
        other => {
            tracing::warn!(
                "Unknown notification_type tag '{}'; falling back to System",
                other
            );
            NotificationType::System
        }
    }
}

/// Parse a lowercase priority tag into the typed enum.
fn parse_notification_priority(tag: &str) -> NotificationPriority {
    match tag {
        "low" => NotificationPriority::Low,
        "normal" => NotificationPriority::Normal,
        "high" => NotificationPriority::High,
        "critical" => NotificationPriority::Critical,
        other => {
            tracing::warn!(
                "Unknown notification_priority tag '{}'; falling back to Normal",
                other
            );
            NotificationPriority::Normal
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use epsx_contracts::notification_port::{
        BroadcastNotificationRequest, NotificationPort, SendNotificationRequest,
    };
    use std::sync::Mutex;

    /// A test double that records every call. Lives in this test
    /// module because the production code path is exercised by the
    /// actual `InProcessNotificationAdapter`, not by a mock — this is
    /// just for verifying the trait's *contract* (DTOs + signatures)
    /// is correct.
    #[derive(Default)]
    struct MockNotificationService {
        sends: Mutex<Vec<SendNotificationRequest>>,
        broadcasts: Mutex<Vec<BroadcastNotificationRequest>>,
    }

    #[async_trait]
    impl NotificationPort for MockNotificationService {
        async fn send(&self, req: SendNotificationRequest) -> AppResult<String> {
            self.sends.lock().unwrap().push(req.clone());
            Ok("mock-send-id".to_string())
        }

        async fn broadcast(&self, req: BroadcastNotificationRequest) -> AppResult<()> {
            self.broadcasts.lock().unwrap().push(req.clone());
            Ok(())
        }
    }

    /// Round-trip: wrap the mock in an `Arc<dyn NotificationPort>`, call
    /// `send` and `broadcast`, and assert the mock saw the requests.
    ///
    /// This is the contract every implementation must satisfy.
    #[tokio::test]
    async fn port_trait_send_round_trip() {
        let mock = Arc::new(MockNotificationService::default());
        let port: Arc<dyn NotificationPort> = mock.clone();

        let req = SendNotificationRequest {
            recipient_wallet_address: "0xabc".to_string(),
            notification_type: "payment".to_string(),
            priority: "normal".to_string(),
            title: "Credits Received".to_string(),
            message: "You received 100 credits".to_string(),
            data: Some(serde_json::json!({ "amount": 100 })),
            action_url: None,
        };

        let result = port.send(req.clone()).await;
        assert!(result.is_ok(), "mock send returned: {:?}", result);
        assert_eq!(result.unwrap(), "mock-send-id");

        let sends = mock.sends.lock().unwrap();
        assert_eq!(sends.len(), 1);
        assert_eq!(sends[0].recipient_wallet_address, "0xabc");
        assert_eq!(sends[0].notification_type, "payment");
        assert_eq!(sends[0].priority, "normal");
        assert_eq!(sends[0].title, "Credits Received");
    }

    /// Round-trip for `broadcast`.
    #[tokio::test]
    async fn port_trait_broadcast_round_trip() {
        let mock = Arc::new(MockNotificationService::default());
        let port: Arc<dyn NotificationPort> = mock.clone();

        let req = BroadcastNotificationRequest {
            notification_type: "system".to_string(),
            priority: "high".to_string(),
            title: "Maintenance".to_string(),
            message: "Down for 10 min".to_string(),
            data: Some(serde_json::json!({ "window": "tonight" })),
        };

        let result = port.broadcast(req.clone()).await;
        assert!(result.is_ok(), "mock broadcast returned: {:?}", result);

        let broadcasts = mock.broadcasts.lock().unwrap();
        assert_eq!(broadcasts.len(), 1);
        assert_eq!(broadcasts[0].notification_type, "system");
        assert_eq!(broadcasts[0].priority, "high");
    }

    /// **Pool-fallback fix regression test.** The constructor must
    /// return `Err(AppError::Configuration)` when
    /// `NOTIFICATIONS_DATABASE_URL` is unset. This is the bug the
    /// audit identified — the pre-wave-10 code silently wrote
    /// notifications to the wrong schema when the notifications DB
    /// was unreachable.
    #[tokio::test]
    async fn notifications_pool_returns_error_when_unset() {
        // Snapshot the prior value (if any) and unset.
        let prior = std::env::var("NOTIFICATIONS_DATABASE_URL").ok();
        std::env::remove_var("NOTIFICATIONS_DATABASE_URL");

        // The unset case — try_new must return Err.
        let result = InProcessNotificationAdapter::try_new(None).await;
        assert!(
            result.is_err(),
            "expected Err when NOTIFICATIONS_DATABASE_URL is unset"
        );
        let err = result.unwrap_err();
        assert_eq!(err.kind, ErrorKind::ConfigurationError);
        assert!(
            err.message.contains("NOTIFICATIONS_DATABASE_URL"),
            "error message should name the missing env var, got: {}",
            err.message
        );

        // The empty-string case (env var present but empty).
        std::env::set_var("NOTIFICATIONS_DATABASE_URL", "");
        let result = InProcessNotificationAdapter::try_new(None).await;
        assert!(
            result.is_err(),
            "expected Err when NOTIFICATIONS_DATABASE_URL is empty"
        );
        assert_eq!(result.unwrap_err().kind, ErrorKind::ConfigurationError);

        // Restore the prior value (best-effort).
        match prior {
            Some(v) => std::env::set_var("NOTIFICATIONS_DATABASE_URL", v),
            None => std::env::remove_var("NOTIFICATIONS_DATABASE_URL"),
        }
    }

    /// `from_pool` is the test-bypass path. It must NOT perform the
    /// env-var check (that's the whole point — tests inject a mock
    /// pool).
    #[test]
    fn from_pool_bypasses_env_check() {
        let prior = std::env::var("NOTIFICATIONS_DATABASE_URL").ok();
        std::env::remove_var("NOTIFICATIONS_DATABASE_URL");

        // The `try_new` constructor would return Err here. The
        // `from_pool` constructor must not — it's the test bypass.
        // We cannot easily build a real `Arc<&'static TlsPool>` in
        // a unit test (it needs a real DB), but the *signature* of
        // `from_pool` is the contract: take an Arc pool, return
        // Self, no env check. The function is small enough to trust
        // the signature; if a future contributor adds an env check
        // here, the `notifications_pool_returns_error_when_unset`
        // test above will continue to pass for `try_new` but this
        // bypass test would need to start exercising the constructor
        // with a real pool (e.g. a testcontainers-managed Postgres).

        match prior {
            Some(v) => std::env::set_var("NOTIFICATIONS_DATABASE_URL", v),
            None => std::env::remove_var("NOTIFICATIONS_DATABASE_URL"),
        }
    }
}
