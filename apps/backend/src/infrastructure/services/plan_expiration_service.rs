//! Plan Expiration Background Service
//!
//! Handles three responsibilities:
//! 1. Send notifications for plans expiring in 7, 3, 1 days
//! 2. Deactivate expired plan assignments (respecting grace period)
//! 3. Cleanup old notifications (delegates to offline_queue)
//!
//! ## Wave 10 / R3
//!
//! The plan-expiry notification was the 8th publisher in the audit.
//! Pre-wave-10 it did its own `INSERT INTO wallet_notifications` plus
//! `RedisNotificationBroadcaster::publish_to_wallet`. After the
//! `NotificationPort` lift, the service holds an
//! `Arc<dyn NotificationPort>` and calls `port.send(...)` instead.
//! The dedup-key check still uses the raw SQL because the port does
//! not expose a "check for existing" method (and should not — the
//! audit's R3 scope is *delivery*, not admin / read paths).

use std::sync::Arc;
use diesel_async::RunQueryDsl;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::prelude::TlsPool;
use crate::web::notifications::{
    cleanup_old_notifications,
};
use epsx_contracts::notification_port::{NotificationPort, SendNotificationRequest};
use epsx_contracts::pubsub_port::PubsubPort;

pub struct PlanExpirationConfig {
    pub poll_interval_secs: u64,
    pub notification_days: Vec<i64>,
}

impl Default for PlanExpirationConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 3600, // 1 hour
            notification_days: vec![7, 3, 1],
        }
    }
}

pub struct PlanExpirationService {
    db_pool: Arc<&'static TlsPool>,
    notifications_pool: Option<Arc<&'static TlsPool>>,
    // Wave 10 / R3: the notification port is wired by the
    // container factory. When `None`, the service still runs its
    // cleanup and dedup logic but skips the plan-expiry
    // notification publish (logged as a warning so the misconfig is
    // visible in production).
    notification_port: Option<Arc<dyn NotificationPort>>,
    config: PlanExpirationConfig,
}

impl PlanExpirationService {
    pub fn new(
        db_pool: Arc<&'static TlsPool>,
        notifications_pool: Option<Arc<&'static TlsPool>>,
        // Wave 10 integration gate: the `pubsub` argument is the
        // Track B call-site update (main.rs now passes
        // `container.pubsub` as the 3rd arg). The service itself
        // does not publish notifications directly anymore — the
        // `NotificationPort` adapter owns the broadcaster. We
        // accept the arg to keep the call-site signature stable;
        // the field is no longer stored on the struct.
        _pubsub: Option<Arc<dyn PubsubPort>>,
    ) -> Self {
        Self {
            db_pool,
            notifications_pool,
            notification_port: None,
            config: PlanExpirationConfig::default(),
        }
    }

    /// Attach the `NotificationPort` (called by the container factory
    /// after the in-process adapter is constructed).
    pub fn with_notification_port(mut self, port: Arc<dyn NotificationPort>) -> Self {
        self.notification_port = Some(port);
        self
    }

    /// Start the background service loop
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        let svc = Arc::new(self);
        tokio::spawn(async move {
            info!("PlanExpirationService started (poll interval: {}s)", svc.config.poll_interval_secs);
            svc.run_loop().await;
        })
    }

    async fn run_loop(self: Arc<Self>) {
        loop {
            // 1. Check for expiring plans and send notifications
            if let Err(e) = self.check_expiring_plans().await {
                error!("PlanExpirationService: notification check failed: {}", e);
            }

            // 2. Cleanup expired assignments
            if let Err(e) = self.cleanup_expired_assignments().await {
                error!("PlanExpirationService: cleanup failed: {}", e);
            }

            // 3. Cleanup old notifications
            if let Some(pool) = &self.notifications_pool {
                if let Err(e) = cleanup_old_notifications(pool, 90).await {
                    warn!("PlanExpirationService: notification cleanup failed: {}", e);
                }
            }

            sleep(Duration::from_secs(self.config.poll_interval_secs)).await;
        }
    }

    /// Check for plans expiring within configured day windows and send notifications
    async fn check_expiring_plans(&self) -> Result<(), String> {
        let notif_pool = match &self.notifications_pool {
            Some(p) => p,
            None => return Ok(()), // No notification DB, skip
        };

        let mut db_conn = self.db_pool.get().await
            .map_err(|e| format!("DB pool error: {}", e))?;

        for &days in &self.config.notification_days {
            // Find active assignments expiring within this window
            #[derive(diesel::QueryableByName)]
            #[allow(dead_code)]
            struct ExpiringRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                assignment_id: Uuid,
                #[diesel(sql_type = diesel::sql_types::Text)]
                wallet_address: String,
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                plan_id: Uuid,
                #[diesel(sql_type = diesel::sql_types::Text)]
                plan_name: String,
                #[diesel(sql_type = diesel::sql_types::BigInt)]
                days_left: i64,
            }

            let rows: Vec<ExpiringRow> = diesel::sql_query(
                r#"
                SELECT
                    wpa.id as assignment_id,
                    wpa.wallet_address,
                    wpa.plan_id,
                    pl.name as plan_name,
                    EXTRACT(DAY FROM (wpa.expires_at - NOW()))::BIGINT as days_left
                FROM wallet_plan_assignments wpa
                JOIN plans pl ON pl.id = wpa.plan_id
                WHERE wpa.is_active = true
                  AND wpa.expires_at IS NOT NULL
                  AND wpa.expires_at > NOW()
                  AND wpa.expires_at <= NOW() + ($1 || ' days')::INTERVAL
                "#
            )
            .bind::<diesel::sql_types::Text, _>(days.to_string())
            .load(&mut db_conn)
            .await
            .unwrap_or_default();

            for row in &rows {
                let dedup_key = format!("plan_expiry_{}d_{}", days, row.plan_id);

                // Check idempotency: skip if notification already sent
                if self.notification_exists(notif_pool, &row.wallet_address, &dedup_key).await {
                    continue;
                }

                let title = format!(
                    "Plan expiring in {} day{}",
                    row.days_left.max(1),
                    if row.days_left > 1 { "s" } else { "" }
                );
                let message = format!(
                    "Your {} plan expires soon. Renew to keep your access.",
                    row.plan_name
                );

                // Wave 10 / R3: route through the NotificationPort.
                // The port handles DB insert + Redis publish as a
                // single atomic-ish call; the in-process adapter
                // also enforces the no-URL fallback fix
                // (AppError::Configuration when
                // NOTIFICATIONS_DATABASE_URL is unset).
                if let Some(port) = self.notification_port.as_ref() {
                    let res = port
                        .send(SendNotificationRequest {
                            recipient_wallet_address: row.wallet_address.clone(),
                            notification_type: "payment".to_string(),
                            priority: "high".to_string(),
                            title: title.clone(),
                            message: message.clone(),
                            data: Some(serde_json::json!({
                                "dedup_key": dedup_key,
                                "plan_id": row.plan_id.to_string(),
                                "plan_name": row.plan_name,
                                "days_remaining": row.days_left,
                                "action": "renew",
                            })),
                            action_url: Some("/plans".to_string()),
                        })
                        .await;
                    if let Err(e) = res {
                        warn!(
                            "Failed to publish plan-expiry notification via port: {}",
                            e
                        );
                        continue;
                    }
                } else {
                    // No port wired — the service still runs cleanup
                    // and dedup, but cannot publish the notification.
                    // Logged at warn so a misconfig is visible in
                    // production; pre-wave-10 the same call silently
                    // succeeded by writing to the primary pool, which
                    // is the bug the audit flagged.
                    warn!(
                        "notification_port not wired in PlanExpirationService; \
                         skipping plan-expiry notification for wallet={}",
                        row.wallet_address
                    );
                    continue;
                }

                info!(
                    "Sent {}d expiry notification: wallet={}, plan={}",
                    days, row.wallet_address, row.plan_name
                );
            }

            if !rows.is_empty() {
                info!("Checked {}d window: {} expiring assignments", days, rows.len());
            }
        }

        Ok(())
    }

    /// Check if a dedup notification already exists
    async fn notification_exists(&self, pool: &TlsPool, wallet: &str, dedup_key: &str) -> bool {
        let mut conn = match pool.get().await {
            Ok(c) => c,
            Err(_) => return false,
        };

        #[derive(diesel::QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            cnt: i64,
        }

        let result: Option<CountRow> = diesel::sql_query(
            r#"
            SELECT COUNT(*)::BIGINT as cnt
            FROM wallet_notifications
            WHERE LOWER(recipient_wallet_address) = LOWER($1)
              AND notification_type = 'payment'
              AND data_payload->>'dedup_key' = $2
              AND status != 'deleted'
            "#
        )
        .bind::<diesel::sql_types::Text, _>(wallet)
        .bind::<diesel::sql_types::Text, _>(dedup_key)
        .get_result(&mut conn)
        .await
        .ok();

        result.map(|r| r.cnt > 0).unwrap_or(false)
    }

    /// Deactivate expired assignments, respecting grace_period_hours
    async fn cleanup_expired_assignments(&self) -> Result<(), String> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| format!("DB pool error: {}", e))?;

        let affected = diesel::sql_query(
            r#"
            UPDATE wallet_plan_assignments wpa
            SET is_active = false, updated_at = NOW()
            FROM plans pl
            WHERE pl.id = wpa.plan_id
              AND wpa.is_active = true
              AND wpa.expires_at IS NOT NULL
              AND wpa.expires_at + (COALESCE(pl.grace_period_hours, 0) || ' hours')::INTERVAL < NOW()
            "#
        )
        .execute(&mut conn)
        .await
        .map_err(|e| format!("Cleanup expired assignments failed: {}", e))?;

        if affected > 0 {
            info!("Deactivated {} expired plan assignments", affected);
        }

        Ok(())
    }
}
