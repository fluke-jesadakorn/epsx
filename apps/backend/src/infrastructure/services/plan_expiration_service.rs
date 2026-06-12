//! Plan Expiration Background Service
//!
//! Handles three responsibilities:
//! 1. Send notifications for plans expiring in 7, 3, 1 days
//! 2. Deactivate expired plan assignments (respecting grace period)
//! 3. Cleanup old notifications (delegates to offline_queue)

use std::sync::Arc;
use chrono::Utc;
use diesel_async::RunQueryDsl;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::prelude::TlsPool;
use crate::web::notifications::{
    SSENotification, NotificationType, NotificationPriority,
    cleanup_old_notifications,
};
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
    pubsub: Option<Arc<dyn PubsubPort>>,
    config: PlanExpirationConfig,
}

impl PlanExpirationService {
    pub fn new(
        db_pool: Arc<&'static TlsPool>,
        notifications_pool: Option<Arc<&'static TlsPool>>,
        pubsub: Option<Arc<dyn PubsubPort>>,
    ) -> Self {
        Self {
            db_pool,
            notifications_pool,
            pubsub,
            config: PlanExpirationConfig::default(),
        }
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

                let title = format!("Plan expiring in {} day{}", row.days_left.max(1), if row.days_left > 1 { "s" } else { "" });
                let message = format!(
                    "Your {} plan expires soon. Renew to keep your access.",
                    row.plan_name
                );

                let notification = SSENotification {
                    id: Uuid::new_v4().to_string(),
                    wallet_address: row.wallet_address.clone(),
                    notification_type: NotificationType::Payment,
                    title: title.clone(),
                    message: message.clone(),
                    data: Some(serde_json::json!({
                        "dedup_key": dedup_key,
                        "plan_id": row.plan_id.to_string(),
                        "plan_name": row.plan_name,
                        "days_remaining": row.days_left,
                        "action": "renew"
                    })),
                    priority: NotificationPriority::High,
                    timestamp: Utc::now(),
                    expires_at: None,
                };

                // Persist to DB
                if let Err(e) = self.insert_notification(notif_pool, &notification, &dedup_key).await {
                    warn!("Failed to persist expiry notification: {}", e);
                    continue;
                }

                // Broadcast via Redis for real-time SSE delivery
                if let Some(pubsub) = &self.pubsub {
                    let channel = format!("notifications:wallet:{}", row.wallet_address.to_lowercase());
                    let payload = match serde_json::to_vec(&notification) {
                        Ok(p) => p,
                        Err(e) => {
                            warn!("Failed to serialize expiry notification: {}", e);
                            continue;
                        }
                    };
                    let _ = pubsub.publish(&channel, &payload).await;
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

    /// Insert notification into wallet_notifications table
    async fn insert_notification(
        &self,
        pool: &TlsPool,
        notif: &SSENotification,
        _dedup_key: &str,
    ) -> Result<(), String> {
        let mut conn = pool.get().await
            .map_err(|e| format!("Notification DB pool error: {}", e))?;

        let id = Uuid::parse_str(&notif.id)
            .unwrap_or_else(|_| Uuid::new_v4());
        let ntype = serde_json::to_value(&notif.notification_type)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "payment".to_string());
        let priority = serde_json::to_value(&notif.priority)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "high".to_string());

        diesel::sql_query(
            r#"
            INSERT INTO wallet_notifications
                (id, recipient_wallet_address, notification_type, title, body, data_payload, priority, created_at, action_url, status)
            VALUES ($1, LOWER($2), $3, $4, $5, $6, $7, NOW(), '/plans', 'created')
            "#
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Text, _>(&notif.wallet_address)
        .bind::<diesel::sql_types::Text, _>(&ntype)
        .bind::<diesel::sql_types::Text, _>(&notif.title)
        .bind::<diesel::sql_types::Text, _>(&notif.message)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(notif.data.as_ref())
        .bind::<diesel::sql_types::Text, _>(&priority)
        .execute(&mut conn)
        .await
        .map_err(|e| format!("Insert notification failed: {}", e))?;

        Ok(())
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
