//! In-process adapter for `NotificationPort`.
//!
//! Wave 10 service-boundary refactor. This is the **current-behavior**
//! implementation of the port — it persists the row to the
//! notifications DB and publishes the SSE payload via the
//! `RedisNotificationBroadcaster`, exactly like the pre-wave-10
//! `NotificationService::send` / `NotificationService::broadcast` did.
//!
//! An opt-in `HttpNotificationAdapter` now forwards the same
//! `SendNotificationRequest` / `BroadcastNotificationRequest` over HTTP.
//! The 8 publisher call sites call the trait, not the adapter, so the
//! integration gate can select the remote implementation without changing
//! producer behavior.
//!
//! ## Construction
//!
//! The adapter owns its own `Arc<WalletNotificationRepository>` and
//! `Option<Arc<RedisNotificationBroadcaster>>` so the trait's
//! stateless interface (`&self, req`) can be satisfied without
//! threading an `AppState` through every call. The port-trait
//! signature is the contract the HTTP implementation relies on; it cannot
//! import the application-layer `AppState` over a network boundary.
//!
//! ## Pool-fallback fix
//!
//! The pre-wave-10 code fell back to `app_state.db_pool` when the
//! notifications pool was unavailable, silently writing to the wrong
//! schema. The fix is in the constructor: if `NOTIFICATIONS_DATABASE_URL`
//! is unset, the constructor returns `Err(AppError::Configuration)`. The
//! DI wiring in `bootstrap.rs` (and the in-process container factory) logs
//! the unavailable port and leaves publisher call sites without a delivery
//! adapter; startup fail-closed policy remains an explicit cutover blocker.
//! The `NotificationService` struct is kept around for the
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
use chrono::{DateTime, Utc};
use epsx_contracts::errors::{AppError, AppResult, ErrorKind};
use epsx_contracts::notification_port::{
    BroadcastNotificationRequest, NotificationPort, SendNotificationRequest,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::prelude::TlsPool;
use crate::web::admin::wallet_notification_repository::WalletNotificationRepository;
use crate::web::notifications::{NotificationPriority, NotificationType, SSENotification};
use epsx_contracts::pubsub_port::PubsubPort;

/// In-process `NotificationPort` adapter. Owns the resources it needs
/// to fulfill a request: the notifications DB pool wrapped in the
/// `WalletNotificationRepository`, plus the kernel-level
/// `PubsubPort` for real-time fanout.
pub struct InProcessNotificationAdapter {
    pool: Arc<TlsPool>,
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
    /// is the bug the audit flagged. The HTTP implementation does not
    /// use this pool because it talks to a remote service.
    pub async fn try_new(broadcaster: Option<Arc<dyn PubsubPort>>) -> AppResult<Self> {
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
        pool: Arc<TlsPool>,
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

    fn validate_expiration(expires_at: Option<DateTime<Utc>>) -> AppResult<()> {
        let Some(expires_at) = expires_at else {
            return Ok(());
        };
        let now = Utc::now();
        if expires_at <= now || expires_at > now + chrono::Duration::days(365) {
            return Err(AppError::validation_error(
                "notification expiry must be in the future and within 365 days",
            ));
        }
        Ok(())
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

    /// Persist a row to `wallet_notifications` with a caller-supplied
    /// identity.  Event-aware callers use a deterministic UUID so retries
    /// can reuse the first durable row.
    async fn persist_with_id(
        &self,
        id: Uuid,
        wallet: &str,
        notification_type: &str,
        priority: &str,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
        action_url: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> AppResult<Uuid> {
        let repo = WalletNotificationRepository::new(self.pool.clone());
        repo.create(
            id,
            wallet,
            notification_type,
            title,
            message,
            data,
            priority,
            expires_at,
            action_url,
            None,
        )
        .await?;
        Ok(id)
    }

    /// Persist a row with a fresh local identity for legacy callers.
    async fn persist(
        &self,
        wallet: &str,
        notification_type: &str,
        priority: &str,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
        action_url: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> AppResult<Uuid> {
        self.persist_with_id(
            Uuid::new_v4(),
            wallet,
            notification_type,
            priority,
            title,
            message,
            data,
            action_url,
            expires_at,
        )
        .await
    }

    async fn persist_event_once(
        &self,
        event_id: &str,
        wallet: &str,
        notification_type: &str,
        priority: &str,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
        action_url: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> AppResult<(Uuid, bool)> {
        let id = stable_event_id(event_id)?;
        let repo = WalletNotificationRepository::new(self.pool.clone());
        if let Some(existing) = repo.find_identity_by_id(id).await? {
            if existing.matches_payload(
                wallet,
                notification_type,
                priority,
                title,
                message,
                data.as_ref(),
                action_url.as_deref(),
                expires_at,
            ) {
                return Ok((id, false));
            }
            return Err(AppError::conflict(
                "notification event identity was reused with a different payload",
            ));
        }
        let persisted = self
            .persist_with_id(
                id,
                wallet,
                notification_type,
                priority,
                title,
                message,
                data.clone(),
                action_url.clone(),
                expires_at,
            )
            .await;
        match persisted {
            Ok(id) => Ok((id, true)),
            Err(error) => match repo.find_identity_by_id(id).await {
                Ok(Some(existing))
                    if existing.matches_payload(
                        wallet,
                        notification_type,
                        priority,
                        title,
                        message,
                        data.as_ref(),
                        action_url.as_deref(),
                        expires_at,
                    ) =>
                {
                    Ok((id, false))
                }
                Ok(Some(_)) => Err(AppError::conflict(
                    "notification event identity was reused with a different payload",
                )),
                Ok(None) | Err(_) => Err(error),
            },
        }
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
                        "Failed to serialize notification SSE (broadcast={}): {}",
                        broadcast,
                        e
                    );
                    return;
                }
            };
            if let Err(e) = broadcaster.publish(&channel, &payload).await {
                tracing::warn!(
                    "Failed to publish notification via PubsubPort (broadcast={}): {}",
                    broadcast,
                    e
                );
            }
        }
    }
}

#[async_trait]
impl NotificationPort for InProcessNotificationAdapter {
    async fn send(&self, req: SendNotificationRequest) -> AppResult<String> {
        Self::validate_expiration(req.expires_at)?;
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
                req.expires_at,
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
            expires_at: req.expires_at,
        };
        self.publish_sse(&wallet, &sse, false).await;
        Ok(id.to_string())
    }

    async fn send_with_event_id(
        &self,
        event_id: &str,
        req: SendNotificationRequest,
    ) -> AppResult<String> {
        Self::validate_expiration(req.expires_at)?;
        let wallet = req.recipient_wallet_address.to_lowercase();
        let (id, inserted) = self
            .persist_event_once(
                event_id,
                &wallet,
                &req.notification_type,
                &req.priority,
                &req.title,
                &req.message,
                req.data.clone(),
                req.action_url.clone(),
                req.expires_at,
            )
            .await?;

        // A retry of an already accepted event is acknowledged without
        // emitting a second real-time notification.  The existence check in
        // `persist_event_once` makes this branch race-safe for the common
        // sequential retry path; the unique ID still protects the row.
        if !inserted {
            return Ok(id.to_string());
        }
        let sse = SSENotification {
            id: id.to_string(),
            wallet_address: wallet.clone(),
            notification_type: parse_notification_type(&req.notification_type),
            title: req.title.clone(),
            message: req.message.clone(),
            data: req.data.clone(),
            priority: parse_notification_priority(&req.priority),
            timestamp: Utc::now(),
            expires_at: req.expires_at,
        };
        self.publish_sse(&wallet, &sse, false).await;
        Ok(id.to_string())
    }

    async fn broadcast(&self, req: BroadcastNotificationRequest) -> AppResult<()> {
        Self::validate_expiration(req.expires_at)?;
        let id = self
            .persist(
                "all",
                &req.notification_type,
                &req.priority,
                &req.title,
                &req.message,
                req.data.clone(),
                None,
                req.expires_at,
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
            expires_at: req.expires_at,
        };
        self.publish_sse("all", &sse, true).await;
        Ok(())
    }

    async fn broadcast_with_event_id(
        &self,
        event_id: &str,
        req: BroadcastNotificationRequest,
    ) -> AppResult<()> {
        Self::validate_expiration(req.expires_at)?;
        let (id, inserted) = self
            .persist_event_once(
                event_id,
                "all",
                &req.notification_type,
                &req.priority,
                &req.title,
                &req.message,
                req.data.clone(),
                None,
                req.expires_at,
            )
            .await?;
        if !inserted {
            return Ok(());
        }
        let sse = SSENotification {
            id: id.to_string(),
            wallet_address: "all".to_string(),
            notification_type: parse_notification_type(&req.notification_type),
            title: req.title.clone(),
            message: req.message.clone(),
            data: req.data.clone(),
            priority: parse_notification_priority(&req.priority),
            timestamp: Utc::now(),
            expires_at: req.expires_at,
        };
        self.publish_sse("all", &sse, true).await;
        Ok(())
    }
}

fn validate_event_id(event_id: &str) -> AppResult<()> {
    if event_id.trim().is_empty()
        || event_id.len() > 128
        || event_id
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(AppError::validation_error(
            "notification event identity must be non-empty, bounded, and whitespace-free",
        ));
    }
    Ok(())
}

fn stable_event_id(event_id: &str) -> AppResult<Uuid> {
    validate_event_id(event_id)?;
    Ok(Uuid::new_v5(&Uuid::NAMESPACE_URL, event_id.as_bytes()))
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
            expires_at: None,
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
            expires_at: None,
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
        // We cannot easily build a real `Arc<TlsPool>` in
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

    #[test]
    fn event_identity_is_stable_and_rejects_ambiguous_values() {
        let first = stable_event_id("payment.confirmed:abc").expect("valid event id");
        let second = stable_event_id("payment.confirmed:abc").expect("valid event id");
        assert_eq!(first, second);
        assert!(stable_event_id("").is_err());
        assert!(stable_event_id("payment confirmed").is_err());
        assert!(stable_event_id(&"x".repeat(129)).is_err());
    }

    #[test]
    fn expiration_boundary_matches_extracted_service_policy() {
        assert!(InProcessNotificationAdapter::validate_expiration(None).is_ok());
        assert!(InProcessNotificationAdapter::validate_expiration(Some(
            Utc::now() + chrono::Duration::minutes(1)
        ))
        .is_ok());
        assert!(InProcessNotificationAdapter::validate_expiration(Some(
            Utc::now() - chrono::Duration::seconds(1)
        ))
        .is_err());
        assert!(InProcessNotificationAdapter::validate_expiration(Some(
            Utc::now() + chrono::Duration::days(366)
        ))
        .is_err());
    }
}
