//! Admin Payment Reprocess Handler
//!
//! Allows admins to manually re-trigger finalize_payment for stuck transactions.
//! Useful when verify_transfer_logs or plan assignment failed silently.
//!
//! BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with raw SQL.

use axum::{
    extract::{Path, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::info;

use crate::{
    infrastructure::blockchain::tx_monitor_service::reprocess_payment_tx,
    infrastructure::database::get_payments_pool,
    prelude::*,
    web::{auth::AppState, middleware::UnifiedErrorResponse},
};

// ============================================================================
// RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ReprocessResponse {
    pub success: bool,
    pub message: String,
    pub data: ReprocessData,
}

#[derive(Debug, Serialize)]
pub struct ReprocessData {
    pub tx_hash: String,
    pub status: String,
    pub error_message: Option<String>,
    pub confirmations: i32,
    pub last_checked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PaymentEventsResponse {
    pub success: bool,
    pub data: Vec<PaymentEventEntry>,
}

#[derive(Debug, Serialize)]
pub struct PaymentEventEntry {
    pub action: String,
    pub old_status: Option<String>,
    pub new_status: Option<String>,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// HANDLERS
// ============================================================================

/// POST /api/payments/admin/tx/:tx_hash/reprocess
#[axum::debug_handler]
pub async fn admin_reprocess_payment_handler(
    State(_app_state): State<AppState>,
    Path(tx_hash): Path<String>,
) -> Result<Json<ReprocessResponse>, Json<UnifiedErrorResponse>> {
    if !tx_hash.starts_with("0x") || tx_hash.len() != 66 {
        return Err(UnifiedErrorResponse::json(
            400,
            "Invalid transaction hash",
            "Must be 66 chars starting with 0x",
        ));
    }

    info!("Admin reprocess requested for tx: {}", tx_hash);

    let result = reprocess_payment_tx(&tx_hash).await;
    let reprocess_err = result.err();

    let payments_pool = get_payments_pool().await.map_err(|e| {
        UnifiedErrorResponse::json(500, "Database error", format!("Cannot connect: {}", e))
    })?;

    #[derive(sqlx::FromRow)]
    struct PaymentStateRow {
        status: String,
        error_message: Option<String>,
        confirmations: Option<i32>,
        last_checked_at: Option<DateTime<Utc>>,
    }

    let state: Option<PaymentStateRow> = sqlx::query_as(
        "SELECT status, error_message, confirmations, last_checked_at FROM payments WHERE transaction_hash = $1 LIMIT 1",
    )
    .bind(&tx_hash)
    .fetch_optional(&payments_pool)
    .await
    .ok()
    .flatten();

    let Some(state) = state else {
        return Err(UnifiedErrorResponse::json(
            404,
            "Transaction not found",
            "No payment record for this tx hash",
        ));
    };

    let success = reprocess_err.is_none() && state.status == "confirmed";
    let message = if state.status == "confirmed" {
        "Payment successfully finalized".to_string()
    } else if let Some(ref err) = reprocess_err {
        format!("Reprocess failed: {}", err)
    } else {
        format!("Reprocessed — current status: {}", state.status)
    };

    Ok(Json(ReprocessResponse {
        success,
        message,
        data: ReprocessData {
            tx_hash,
            status: state.status,
            error_message: state.error_message,
            confirmations: state.confirmations.unwrap_or(0),
            last_checked_at: state.last_checked_at,
        },
    }))
}

/// GET /api/payments/admin/tx/:tx_hash/events
#[axum::debug_handler]
pub async fn admin_payment_events_handler(
    State(_app_state): State<AppState>,
    Path(tx_hash): Path<String>,
) -> Result<Json<PaymentEventsResponse>, Json<UnifiedErrorResponse>> {
    if !tx_hash.starts_with("0x") || tx_hash.len() != 66 {
        return Err(UnifiedErrorResponse::json(
            400,
            "Invalid transaction hash",
            "Must be 66 chars starting with 0x",
        ));
    }

    let payments_pool = get_payments_pool().await.map_err(|e| {
        UnifiedErrorResponse::json(500, "Database error", format!("Cannot connect: {}", e))
    })?;

    #[derive(sqlx::FromRow)]
    struct AuditRow {
        action: String,
        old_status: Option<String>,
        new_status: Option<String>,
        reason: Option<String>,
        performed_by: Option<String>,
        metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
    }

    let rows: Vec<AuditRow> = sqlx::query_as(
        r#"
        SELECT pal.action, pal.old_status, pal.new_status, pal.reason,
               pal.performed_by, pal.metadata, pal.created_at
        FROM payment_audit_log pal
        JOIN payments p ON p.id = pal.payment_id
        WHERE p.transaction_hash = $1
        ORDER BY pal.created_at ASC
        "#,
    )
    .bind(&tx_hash)
    .fetch_all(&payments_pool)
    .await
    .unwrap_or_default();

    let events = rows
        .into_iter()
        .map(|r| PaymentEventEntry {
            action: r.action,
            old_status: r.old_status,
            new_status: r.new_status,
            reason: r.reason,
            performed_by: r.performed_by,
            metadata: r
                .metadata
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(PaymentEventsResponse {
        success: true,
        data: events,
    }))
}
