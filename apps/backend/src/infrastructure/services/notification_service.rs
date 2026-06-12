//! Reusable notification service (legacy shim for pre-wave-10 callers)
//!
//! Wave 10 service-boundary refactor. This file now contains a thin
//! shim around `InProcessNotificationAdapter` to keep the
//! `NotificationService::send` / `NotificationService::broadcast`
//! static API working for any code that still imports it
//! (defensive — the 7 publisher call sites are being migrated to the
//! port in the same wave). New code MUST go through
//! `NotificationPort` directly.
//!
//! ## Migration status
//!
//! The 7 publisher call sites (in `web/payments/credit_handlers.rs`,
//! `web/payments/submit_tx_handler.rs`, `web/user/chat_handlers.rs`,
//! `web/admin/chat_handlers.rs`,
//! `web/admin/permissions/assignments/create.rs`,
//! `web/admin/permissions/assignments/remove.rs`, and
//! `infrastructure/services/plan_expiration_service.rs`) now call
//! `AppState::notification_port` (the `Arc<dyn NotificationPort>`),
//! not `NotificationService::send` / `broadcast`. The static
//! functions here are kept as a defensive fallback for any code path
//! that still uses them; they will be removed in wave 11 once the
//! migration is verified.

use epsx_contracts::errors::{AppError, AppResult};
use epsx_contracts::notification_port::{
    BroadcastNotificationRequest, SendNotificationRequest,
};

use crate::infrastructure::adapters::notification::in_process_adapter::InProcessNotificationAdapter;
use crate::web::auth::AppState;

/// Legacy shim around `InProcessNotificationAdapter`. The struct is
/// kept so `use crate::infrastructure::services::NotificationService;`
/// still resolves and so the (now deprecated) static methods
/// `NotificationService::send` / `NotificationService::broadcast`
/// keep compiling. New code MUST go through `NotificationPort`.
#[deprecated(
    since = "0.1.0",
    note = "Go through `AppState::notification_port` (`Arc<dyn NotificationPort>`) instead. The static methods on this shim will be removed in wave 11."
)]
pub struct NotificationService;

impl NotificationService {
    /// Construct a fresh in-process adapter. Sugar for the
    /// `InProcessNotificationAdapter::try_new` constructor.
    pub async fn new() -> AppResult<InProcessNotificationAdapter> {
        InProcessNotificationAdapter::try_new(None).await
    }

    /// Synchronous env-var check. Re-exported so existing callers
    /// that already import it keep working.
    pub fn check_notifications_url_configured() -> AppResult<()> {
        InProcessNotificationAdapter::check_notifications_url_configured()
    }

    /// Legacy static API. **Deprecated.** Routes through the
    /// in-process adapter; equivalent to calling
    /// `AppState::notification_port.send(req)`. Returns
    /// `AppError::Configuration` if `NOTIFICATIONS_DATABASE_URL` is
    /// unset (the pool-fallback fix).
    #[allow(deprecated)]
    pub async fn send(
        app_state: &AppState,
        recipient_wallet_address: &str,
        notification_type: crate::web::notifications::NotificationType,
        priority: crate::web::notifications::NotificationPriority,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
        action_url: Option<String>,
    ) -> Result<String, AppError> {
        let port = app_state
            .notification_port
            .as_ref()
            .ok_or_else(|| AppError::configuration_error("notification_port not initialized in AppState"))?;
        port.send(SendNotificationRequest {
            recipient_wallet_address: recipient_wallet_address.to_string(),
            notification_type: InProcessNotificationAdapter::format_notification_type_tag(notification_type),
            priority: InProcessNotificationAdapter::format_notification_priority_tag(priority),
            title: title.to_string(),
            message: message.to_string(),
            data,
            action_url,
        })
        .await
    }

    /// Legacy static API. **Deprecated.** Routes through the
    /// in-process adapter. Returns `AppError::Configuration` if
    /// `NOTIFICATIONS_DATABASE_URL` is unset.
    #[allow(deprecated)]
    pub async fn broadcast(
        app_state: &AppState,
        notification_type: crate::web::notifications::NotificationType,
        priority: crate::web::notifications::NotificationPriority,
        title: &str,
        message: &str,
        data: Option<serde_json::Value>,
    ) -> Result<String, AppError> {
        let port = app_state
            .notification_port
            .as_ref()
            .ok_or_else(|| AppError::configuration_error("notification_port not initialized in AppState"))?;
        port.broadcast(BroadcastNotificationRequest {
            notification_type: InProcessNotificationAdapter::format_notification_type_tag(notification_type),
            priority: InProcessNotificationAdapter::format_notification_priority_tag(priority),
            title: title.to_string(),
            message: message.to_string(),
            data,
        })
        .await
        .map(|_| "ok".to_string())
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// The env-var check is the regression test for the
    /// pool-fallback fix. When `NOTIFICATIONS_DATABASE_URL` is unset,
    /// it must return `AppError::ConfigurationError` rather than
    /// silently allowing a fallback to the primary pool.
    #[test]
    fn notifications_pool_returns_error_when_unset() {
        // Snapshot the prior value (if any) and unset.
        let prior = std::env::var("NOTIFICATIONS_DATABASE_URL").ok();
        std::env::remove_var("NOTIFICATIONS_DATABASE_URL");

        // The unset case.
        let result = NotificationService::check_notifications_url_configured();
        assert!(
            result.is_err(),
            "expected Err when NOTIFICATIONS_DATABASE_URL is unset"
        );
        let err = result.unwrap_err();
        assert_eq!(err.kind, epsx_contracts::errors::ErrorKind::ConfigurationError);
        assert!(
            err.message.contains("NOTIFICATIONS_DATABASE_URL"),
            "error message should name the missing env var, got: {}",
            err.message
        );

        // The empty-string case.
        std::env::set_var("NOTIFICATIONS_DATABASE_URL", "");
        let result = NotificationService::check_notifications_url_configured();
        assert!(
            result.is_err(),
            "expected Err when NOTIFICATIONS_DATABASE_URL is empty"
        );
        assert_eq!(
            result.unwrap_err().kind,
            epsx_contracts::errors::ErrorKind::ConfigurationError
        );

        // The configured case — should pass.
        std::env::set_var("NOTIFICATIONS_DATABASE_URL", "postgres://user:pw@host/db");
        let result = NotificationService::check_notifications_url_configured();
        assert!(
            result.is_ok(),
            "expected Ok when NOTIFICATIONS_DATABASE_URL is set, got: {:?}",
            result
        );

        // Restore the prior value (best-effort).
        match prior {
            Some(v) => std::env::set_var("NOTIFICATIONS_DATABASE_URL", v),
            None => std::env::remove_var("NOTIFICATIONS_DATABASE_URL"),
        }
    }
}
