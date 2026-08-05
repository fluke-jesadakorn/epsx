//! Durable channel-job worker primitives.
//!
//! These functions are deliberately provider-agnostic. They claim and
//! transition jobs in PostgreSQL; provider calls are performed by a future
//! configured worker, never by request admission.

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use thiserror::Error;

pub const DEFAULT_MAX_ATTEMPTS: i32 = 5;
pub const MAX_BACKOFF_SECONDS: i64 = 3600;

const CLAIM_NEXT_SQL: &str = "SELECT j.id FROM public.notification_channel_jobs j LEFT JOIN public.notification_expirations x ON x.notification_id = j.notification_id WHERE j.state IN ('queued', 'retry_wait', 'leased') AND j.available_at <= NOW() AND (j.lease_until IS NULL OR j.lease_until <= NOW()) AND (x.expires_at IS NULL OR x.expires_at > NOW()) ORDER BY j.available_at ASC, j.created_at ASC, j.id ASC FOR UPDATE OF j SKIP LOCKED LIMIT 1";
const CLAIM_UPDATE_SQL: &str = "UPDATE public.notification_channel_jobs SET state = 'leased', lease_until = NOW() + ($2 * INTERVAL '1 second'), updated_at = NOW() WHERE id = $1 AND available_at <= NOW() AND (state IN ('queued', 'retry_wait') OR (state = 'leased' AND (lease_until IS NULL OR lease_until <= NOW()))) RETURNING id, notification_id, channel, recipient, attempt_count, lease_until";
const REDRIVE_SQL: &str = "UPDATE public.notification_channel_jobs SET state = 'queued', available_at = NOW(), lease_until = NULL, updated_at = NOW() WHERE id = $1 AND state = 'dead_lettered' AND NOT EXISTS (SELECT 1 FROM public.notification_expirations x WHERE x.notification_id = notification_channel_jobs.notification_id AND x.expires_at <= NOW())";
const EXPIRE_DUE_SQL: &str = "WITH due AS (SELECT j.id FROM public.notification_channel_jobs j JOIN public.notification_expirations x ON x.notification_id = j.notification_id JOIN public.notifications n ON n.id = j.notification_id WHERE x.expires_at <= NOW() AND j.state IN ('queued', 'retry_wait', 'leased', 'attempting') AND n.status IN ('pending', 'suppressed') ORDER BY x.expires_at ASC, j.id ASC FOR UPDATE OF j SKIP LOCKED LIMIT 100), expired_jobs AS (UPDATE public.notification_channel_jobs j SET state = 'terminal_failed', lease_until = NULL, updated_at = NOW() FROM due WHERE j.id = due.id RETURNING j.notification_id) UPDATE public.notifications n SET status = 'expired', error = 'notification_expired' FROM (SELECT DISTINCT notification_id FROM expired_jobs) expired WHERE n.id = expired.notification_id AND n.status IN ('pending', 'suppressed')";
const EXPIRE_NOTIFICATIONS_SQL: &str = "WITH due AS (SELECT n.id FROM public.notifications n JOIN public.notification_expirations x ON x.notification_id = n.id WHERE x.expires_at <= NOW() AND n.status IN ('pending', 'suppressed') ORDER BY x.expires_at ASC, n.id ASC FOR UPDATE OF n SKIP LOCKED LIMIT 100) UPDATE public.notifications n SET status = 'expired', error = 'notification_expired' FROM due WHERE n.id = due.id AND n.status IN ('pending', 'suppressed')";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryErrorClass {
    Transient,
    Permanent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryResult {
    Accepted,
    Retry,
    TerminalFailure,
}

#[derive(Debug, Error)]
pub enum DeliveryWorkerError {
    #[error("delivery database operation failed")]
    Database(#[source] sqlx::Error),
    #[error("delivery worker transition rejected")]
    Transition,
}

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ClaimedChannelJob {
    pub id: String,
    pub notification_id: String,
    pub channel: String,
    pub recipient: String,
    pub attempt_count: i32,
    pub lease_until: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct DeliveryWorker {
    db: PgPool,
    max_attempts: i32,
}

impl DeliveryWorker {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
        }
    }

    pub fn with_max_attempts(mut self, max_attempts: i32) -> Self {
        self.max_attempts = max_attempts.clamp(1, 100);
        self
    }

    pub async fn claim_next(
        &self,
        worker_id: &str,
        lease_seconds: i64,
    ) -> Result<Option<ClaimedChannelJob>, DeliveryWorkerError> {
        if worker_id.trim().is_empty() || lease_seconds <= 0 || lease_seconds > 3600 {
            return Err(DeliveryWorkerError::Transition);
        }
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(DeliveryWorkerError::Database)?;
        let candidate = sqlx::query_as::<_, (String,)>(CLAIM_NEXT_SQL)
            .fetch_optional(&mut *tx)
            .await
            .map_err(DeliveryWorkerError::Database)?;
        let Some((job_id,)) = candidate else {
            tx.commit().await.map_err(DeliveryWorkerError::Database)?;
            return Ok(None);
        };
        let Some(job) = sqlx::query_as::<_, ClaimedChannelJob>(CLAIM_UPDATE_SQL)
            .bind(&job_id)
            .bind(lease_seconds)
            .fetch_optional(&mut *tx)
            .await
            .map_err(DeliveryWorkerError::Database)?
        else {
            tx.commit().await.map_err(DeliveryWorkerError::Database)?;
            return Ok(None);
        };
        tracing::debug!(worker_id, job_id = %job.id, "notification channel job leased");
        tx.commit().await.map_err(DeliveryWorkerError::Database)?;
        Ok(Some(job))
    }

    /// Sweep due queued/retry/leased jobs in a bounded batch so filtering them
    /// out of `claim_next` cannot leave expired work permanently queued.
    pub async fn expire_due_jobs(&self) -> Result<u64, DeliveryWorkerError> {
        let jobs = sqlx::query(EXPIRE_DUE_SQL)
            .execute(&self.db)
            .await
            .map_err(DeliveryWorkerError::Database)?;
        let notifications = sqlx::query(EXPIRE_NOTIFICATIONS_SQL)
            .execute(&self.db)
            .await
            .map_err(DeliveryWorkerError::Database)?;
        Ok(jobs
            .rows_affected()
            .saturating_add(notifications.rows_affected()))
    }

    pub async fn begin_attempt(&self, job_id: &str) -> Result<(), DeliveryWorkerError> {
        let result = sqlx::query(
            "UPDATE public.notification_channel_jobs SET state = 'attempting', attempt_count = attempt_count + 1, updated_at = NOW() WHERE id = $1 AND state = 'leased'",
        )
        .bind(job_id)
        .execute(&self.db)
        .await
        .map_err(DeliveryWorkerError::Database)?;
        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(DeliveryWorkerError::Transition)
        }
    }

    /// Mark a claimed job and its pending notification expired before any
    /// provider call. Expiration is an additive projection, so legacy rows
    /// without an expiration remain eligible for delivery.
    pub async fn expire_if_due(&self, job_id: &str) -> Result<bool, DeliveryWorkerError> {
        if job_id.trim().is_empty() || job_id.len() > 128 {
            return Err(DeliveryWorkerError::Transition);
        }
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(DeliveryWorkerError::Database)?;
        let expired = sqlx::query_as::<_, (String,)>(
            "UPDATE public.notification_channel_jobs j SET state = 'terminal_failed', lease_until = NULL, updated_at = NOW() FROM public.notification_expirations x WHERE j.id = $1 AND j.notification_id = x.notification_id AND x.expires_at <= NOW() AND j.state IN ('leased', 'attempting') RETURNING j.notification_id",
        )
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(DeliveryWorkerError::Database)?;
        let Some((notification_id,)) = expired else {
            tx.commit().await.map_err(DeliveryWorkerError::Database)?;
            return Ok(false);
        };
        sqlx::query(
            "UPDATE public.notifications SET status = 'expired', error = 'notification_expired' WHERE id = $1 AND status IN ('pending', 'suppressed')",
        )
        .bind(notification_id)
        .execute(&mut *tx)
        .await
        .map_err(DeliveryWorkerError::Database)?;
        tx.commit().await.map_err(DeliveryWorkerError::Database)?;
        Ok(true)
    }

    pub async fn record_result(
        &self,
        job_id: &str,
        outcome: DeliveryResult,
        provider_message_id: Option<&str>,
        error_code: Option<&str>,
    ) -> Result<(), DeliveryWorkerError> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(DeliveryWorkerError::Database)?;
        let job = sqlx::query_as::<_, (i32,)>(
            "SELECT attempt_count FROM public.notification_channel_jobs WHERE id = $1 AND state = 'attempting' FOR UPDATE",
        )
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(DeliveryWorkerError::Database)?
        .ok_or(DeliveryWorkerError::Transition)?;
        let attempt_no = job.0;
        let class = match outcome {
            DeliveryResult::Accepted => "accepted",
            DeliveryResult::Retry => "transient_failure",
            DeliveryResult::TerminalFailure => "permanent_failure",
        };
        sqlx::query(
            "INSERT INTO public.notification_delivery_attempts (job_id, attempt_no, outcome, provider_message_id, error_code) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(job_id)
        .bind(attempt_no)
        .bind(class)
        .bind(provider_message_id)
        .bind(error_code)
        .execute(&mut *tx)
        .await
        .map_err(DeliveryWorkerError::Database)?;

        let (state, available_at): (&str, Option<DateTime<Utc>>) = match outcome {
            DeliveryResult::Accepted => ("provider_accepted", None),
            DeliveryResult::Retry if attempt_no < self.max_attempts => (
                "retry_wait",
                Some(Utc::now() + retry_delay_with_jitter(attempt_no)),
            ),
            DeliveryResult::Retry | DeliveryResult::TerminalFailure => ("terminal_failed", None),
        };
        let updated = sqlx::query(
            "UPDATE public.notification_channel_jobs SET state = $2, provider_message_id = COALESCE($3, provider_message_id), available_at = COALESCE($4, available_at), lease_until = NULL, updated_at = NOW() WHERE id = $1",
        )
        .bind(job_id)
        .bind(state)
        .bind(provider_message_id)
        .bind(available_at)
        .execute(&mut *tx)
        .await
        .map_err(DeliveryWorkerError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(DeliveryWorkerError::Transition);
        }
        tx.commit().await.map_err(DeliveryWorkerError::Database)
    }

    pub async fn dead_letter(
        &self,
        job_id: &str,
        reason: &str,
        payload: Value,
    ) -> Result<(), DeliveryWorkerError> {
        if reason.trim().is_empty() || reason.len() > 255 {
            return Err(DeliveryWorkerError::Transition);
        }
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(DeliveryWorkerError::Database)?;
        let updated = sqlx::query(
            "UPDATE public.notification_channel_jobs SET state = 'dead_lettered', lease_until = NULL, updated_at = NOW() WHERE id = $1 AND state = 'terminal_failed'",
        )
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .map_err(DeliveryWorkerError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(DeliveryWorkerError::Transition);
        }
        sqlx::query(
            "INSERT INTO public.notification_dead_letters (job_id, reason, payload) VALUES ($1, $2, $3)",
        )
        .bind(job_id)
        .bind(reason)
        .bind(payload)
        .execute(&mut *tx)
        .await
        .map_err(DeliveryWorkerError::Database)?;
        tx.commit().await.map_err(DeliveryWorkerError::Database)
    }

    /// Requeue one dead-lettered job after an authorized operator decision.
    /// The dead-letter row is retained for audit and its redrive counter is
    /// incremented; no provider call occurs in this state transition.
    pub async fn redrive(&self, job_id: &str) -> Result<(), DeliveryWorkerError> {
        if job_id.trim().is_empty() || job_id.len() > 128 {
            return Err(DeliveryWorkerError::Transition);
        }
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(DeliveryWorkerError::Database)?;
        let updated = sqlx::query(REDRIVE_SQL)
            .bind(job_id)
            .execute(&mut *tx)
            .await
            .map_err(DeliveryWorkerError::Database)?;
        if updated.rows_affected() != 1 {
            return Err(DeliveryWorkerError::Transition);
        }
        sqlx::query(
            "UPDATE public.notification_dead_letters SET redrive_count = redrive_count + 1, last_redriven_at = NOW(), resolved_at = NULL WHERE job_id = $1",
        )
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .map_err(DeliveryWorkerError::Database)?;
        tx.commit().await.map_err(DeliveryWorkerError::Database)
    }
}

pub fn classify_provider_status(status: u16) -> DeliveryErrorClass {
    match status {
        408 | 425 | 429 | 500..=599 => DeliveryErrorClass::Transient,
        _ => DeliveryErrorClass::Permanent,
    }
}

pub fn retry_delay(attempt_no: i32) -> Duration {
    let exponent = attempt_no.saturating_sub(1).clamp(0, 10) as u32;
    let seconds = 2_i64.saturating_pow(exponent).min(MAX_BACKOFF_SECONDS);
    Duration::seconds(seconds)
}

/// Add bounded jitter so concurrent workers do not retry a provider in lock
/// step. The deterministic base delay remains separately testable and the
/// final value is always capped by `MAX_BACKOFF_SECONDS`.
pub fn retry_delay_with_jitter(attempt_no: i32) -> Duration {
    let base = retry_delay(attempt_no).num_seconds();
    let max_jitter = (base / 4).min(60);
    let jitter = if max_jitter == 0 {
        0
    } else {
        rand::rng().random_range(0..=max_jitter)
    };
    Duration::seconds((base + jitter).min(MAX_BACKOFF_SECONDS))
}

/// Return whether the next retry would consume the final permitted attempt.
/// The caller can convert that retry into a terminal/dead-letter transition
/// while retaining the provider error code and durable attempt record.
pub fn retry_will_exhaust_attempts(attempt_count: i32, max_attempts: i32) -> bool {
    attempt_count.saturating_add(1) >= max_attempts.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_statuses_have_explicit_retry_classification() {
        assert_eq!(classify_provider_status(429), DeliveryErrorClass::Transient);
        assert_eq!(classify_provider_status(503), DeliveryErrorClass::Transient);
        assert_eq!(classify_provider_status(400), DeliveryErrorClass::Permanent);
        assert_eq!(classify_provider_status(200), DeliveryErrorClass::Permanent);
    }

    #[test]
    fn retry_backoff_is_bounded_and_monotonic() {
        assert_eq!(retry_delay(1), Duration::seconds(1));
        assert_eq!(retry_delay(2), Duration::seconds(2));
        assert!(retry_delay(20) <= Duration::seconds(MAX_BACKOFF_SECONDS));
        assert!(retry_delay(3) >= retry_delay(2));
    }

    #[test]
    fn jittered_retry_backoff_stays_within_the_bounded_window() {
        let base = retry_delay(4).num_seconds();
        let max = (base + (base / 4).min(60)).min(MAX_BACKOFF_SECONDS);
        for _ in 0..32 {
            let actual = retry_delay_with_jitter(4).num_seconds();
            assert!((base..=max).contains(&actual));
        }
    }

    #[test]
    fn retry_exhaustion_is_guarded_at_the_next_attempt_boundary() {
        assert!(!retry_will_exhaust_attempts(0, 5));
        assert!(!retry_will_exhaust_attempts(3, 5));
        assert!(retry_will_exhaust_attempts(4, 5));
        assert!(retry_will_exhaust_attempts(100, 5));
        assert!(retry_will_exhaust_attempts(0, 0));
    }

    #[tokio::test]
    #[ignore = "requires a disposable migrated local notification database"]
    async fn runtime_retry_dead_letter_and_redrive_are_durable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database_url = std::env::var("NOTIFICATION_RUNTIME_DATABASE_URL")?;
        let db = PgPool::connect(&database_url).await?;
        let suffix = format!(
            "runtime-delivery-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        );
        let notification_id = format!("{suffix}-notification");
        let event_id = format!("{suffix}-event");
        let job_id = format!("{suffix}-job");

        sqlx::query(
            "INSERT INTO public.notifications (id, user_id, channel, recipient, body, status) VALUES ($1, $2, 'email', 'runtime@example.test', 'runtime delivery audit', 'pending')",
        )
        .bind(&notification_id)
        .bind("0x1111111111111111111111111111111111111111")
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_outbox (event_id, event_type, aggregate_id, payload) VALUES ($1, 'runtime.delivery', $2, '{}'::jsonb)",
        )
        .bind(&event_id)
        .bind(&notification_id)
        .execute(&db)
        .await?;
        sqlx::query(
            "INSERT INTO public.notification_channel_jobs (id, source_event_id, notification_id, channel, recipient, idempotency_key) VALUES ($1, $2, $3, 'email', 'runtime@example.test', $4)",
        )
        .bind(&job_id)
        .bind(&event_id)
        .bind(&notification_id)
        .bind(&format!("{suffix}-key"))
        .execute(&db)
        .await?;

        let worker = DeliveryWorker::new(db.clone()).with_max_attempts(1);
        let claimed = worker
            .claim_next("runtime-delivery-audit", 30)
            .await?
            .ok_or("job was not claimable")?;
        assert_eq!(claimed.id, job_id);
        worker.begin_attempt(&job_id).await?;
        worker
            .record_result(
                &job_id,
                DeliveryResult::Retry,
                None,
                Some("provider_send_failed"),
            )
            .await?;
        worker
            .dead_letter(
                &job_id,
                "provider_send_failed",
                serde_json::json!({"audit": true}),
            )
            .await?;

        let state: String =
            sqlx::query_scalar("SELECT state FROM public.notification_channel_jobs WHERE id = $1")
                .bind(&job_id)
                .fetch_one(&db)
                .await?;
        assert_eq!(state, "dead_lettered");
        let attempts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM public.notification_delivery_attempts WHERE job_id = $1",
        )
        .bind(&job_id)
        .fetch_one(&db)
        .await?;
        assert_eq!(attempts, 1);

        worker.redrive(&job_id).await?;
        let (state, redrive_count): (String, i32) = sqlx::query_as(
            "SELECT j.state, d.redrive_count FROM public.notification_channel_jobs j JOIN public.notification_dead_letters d ON d.job_id = j.id WHERE j.id = $1",
        )
        .bind(&job_id)
        .fetch_one(&db)
        .await?;
        assert_eq!(state, "queued");
        assert_eq!(redrive_count, 1);

        // Simulate a worker crash after leasing: the next worker must reclaim
        // the expired lease instead of leaving the durable job stranded.
        sqlx::query(
            "UPDATE public.notification_channel_jobs SET state = 'leased', lease_until = NOW() - INTERVAL '1 second', available_at = NOW() WHERE id = $1",
        )
        .bind(&job_id)
        .execute(&db)
        .await?;
        let reclaimed = worker
            .claim_next("runtime-delivery-recovery", 30)
            .await?
            .ok_or("expired lease was not reclaimed")?;
        assert_eq!(reclaimed.id, job_id);
        worker.begin_attempt(&job_id).await?;
        worker
            .record_result(
                &job_id,
                DeliveryResult::Accepted,
                Some("runtime-recovery-provider"),
                None,
            )
            .await?;
        let state: String =
            sqlx::query_scalar("SELECT state FROM public.notification_channel_jobs WHERE id = $1")
                .bind(&job_id)
                .fetch_one(&db)
                .await?;
        assert_eq!(state, "provider_accepted");

        sqlx::query("DELETE FROM public.notification_dead_letters WHERE job_id = $1")
            .bind(&job_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_delivery_attempts WHERE job_id = $1")
            .bind(&job_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_channel_jobs WHERE id = $1")
            .bind(&job_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notification_outbox WHERE event_id = $1")
            .bind(&event_id)
            .execute(&db)
            .await?;
        sqlx::query("DELETE FROM public.notifications WHERE id = $1")
            .bind(&notification_id)
            .execute(&db)
            .await?;
        db.close().await;
        Ok(())
    }

    #[test]
    fn claim_query_reclaims_expired_leases_without_claiming_live_work() {
        assert!(CLAIM_NEXT_SQL.contains("state IN ('queued', 'retry_wait', 'leased')"));
        assert!(CLAIM_NEXT_SQL.contains("j.lease_until IS NULL OR j.lease_until <= NOW()"));
        assert!(CLAIM_NEXT_SQL.contains("FOR UPDATE OF j SKIP LOCKED"));
        assert!(CLAIM_NEXT_SQL.contains("notification_expirations"));
        assert!(CLAIM_NEXT_SQL.contains("x.expires_at IS NULL OR x.expires_at > NOW()"));
        assert!(EXPIRE_DUE_SQL.contains("LIMIT 100"));
        assert!(EXPIRE_DUE_SQL.contains("status = 'expired'"));
        assert!(EXPIRE_NOTIFICATIONS_SQL.contains("FOR UPDATE OF n SKIP LOCKED"));
        assert!(CLAIM_UPDATE_SQL.contains("state IN ('queued', 'retry_wait')"));
        assert!(CLAIM_UPDATE_SQL.contains("state = 'leased'"));
        assert!(CLAIM_UPDATE_SQL.contains("lease_until IS NULL OR lease_until <= NOW()"));
    }

    #[test]
    fn redrive_query_cannot_requeue_expired_notifications() {
        // Expiry is authoritative even after an operator has dead-lettered a
        // job; a redrive must never create provider work past its deadline.
        assert!(REDRIVE_SQL.contains("state = 'dead_lettered'"));
        assert!(REDRIVE_SQL.contains("NOT EXISTS"));
        assert!(REDRIVE_SQL.contains("x.expires_at <= NOW()"));
    }
}
