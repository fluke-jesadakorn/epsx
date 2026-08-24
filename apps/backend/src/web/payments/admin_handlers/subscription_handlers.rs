//! Admin Subscription Handlers
//!
//! BIG-BANG: migrated to sqlx (real). All diesel DSL replaced with raw SQL.

use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::{Datelike, Utc};
use tracing::{error, info};
use uuid::Uuid;

use crate::web::{middleware::UnifiedErrorResponse, pagination::Pagination};

use super::types::*;

/// Get all subscriptions
pub async fn admin_list_subscriptions_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Query(params): Query<AdminPaymentListParams>,
) -> Result<Json<AdminSubscriptionListResponse>, Json<UnifiedErrorResponse>> {
    use crate::domain::payment::repository_ports::SubscriptionFilters;
    use crate::infrastructure::database::get_payments_pool;

    info!("Admin listing subscriptions with params: {:?}", params);

    let payment_repo = app_state.payment_repo.as_ref().ok_or_else(|| {
        error!(
            "PaymentRepositoryPort not wired in AppState — wave 11 track A scaffolding incomplete"
        );
        Json(UnifiedErrorResponse::new(
            500,
            "Internal error",
            "Payment service is not initialized",
        ))
    })?;

    let pg = Pagination::large(params.page, params.limit);

    let filters = SubscriptionFilters {
        wallet_address: params.wallet_address.clone(),
        plan_id: params.plan_id,
        status: params.status.clone(),
    };
    let (rows, total_count) = payment_repo
        .list_admin_subscriptions_with_plan_names_paginated(filters.clone(), pg.page, pg.limit)
        .await
        .map_err(|e| {
            error!("Failed to list subscriptions: {}", e);
            Json(UnifiedErrorResponse::new(
                500,
                "Query failed",
                format!("Failed to load subscriptions: {}", e),
            ))
        })?;

    let subscriptions_resp: Vec<AdminSubscriptionInfo> = rows
        .into_iter()
        .map(|(sub, plan_name)| AdminSubscriptionInfo {
            id: sub.id,
            wallet_address: sub.wallet_address,
            plan_id: sub.plan_id,
            plan_name: plan_name.unwrap_or_else(|| "Unknown Plan".to_string()),
            status: sub.status,
            payment_id: sub.payment_id.unwrap_or(Uuid::nil()),
            started_at: sub.started_at.unwrap_or_else(Utc::now),
            expires_at: sub.expires_at,
            cancelled_at: sub.cancelled_at,
            auto_renew: sub.auto_renew,
            metadata: sub.metadata,
        })
        .collect();

    let total_pages = pg.total_pages(total_count);
    let pagination = PaginationInfo {
        page: pg.page,
        limit: pg.limit,
        total_count,
        total_pages,
        has_next: pg.has_next(total_count),
        has_prev: pg.has_prev(),
    };

    // Calculate summary statistics with real database queries
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(
            500,
            "Database connection failed",
            "Failed to get payments database pool",
        ))
    })?;

    let today_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);
    let seven_days_from_now = Utc::now() + chrono::Duration::days(7);
    let month_start = Utc::now()
        .date_naive()
        .with_day(1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);

    // Active count
    let active_count: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM subscriptions WHERE status = 'active'",
    )
    .fetch_one(payments_pool.as_ref())
    .await
    .unwrap_or(0);

    // Expired count
    let expired_count: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM subscriptions WHERE status = 'expired'",
    )
    .fetch_one(payments_pool.as_ref())
    .await
    .unwrap_or(0);

    // Cancelled count
    let cancelled_count: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM subscriptions WHERE status = 'cancelled'",
    )
    .fetch_one(payments_pool.as_ref())
    .await
    .unwrap_or(0);

    // New subscriptions today
    let new_today: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM subscriptions WHERE started_at >= $1",
    )
    .bind(today_start)
    .fetch_one(payments_pool.as_ref())
    .await
    .unwrap_or(0);

    // Expiring soon (active and expires_at <= 7 days from now)
    let expiring_soon_count: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM subscriptions \
         WHERE status = 'active' AND expires_at <= $1 AND expires_at >= $2",
    )
    .bind(seven_days_from_now)
    .bind(Utc::now())
    .fetch_one(payments_pool.as_ref())
    .await
    .unwrap_or(0);

    // Monthly revenue from payments
    let monthly_revenue_bd: Option<bigdecimal::BigDecimal> = sqlx::query_scalar::<_, Option<bigdecimal::BigDecimal>>(
        "SELECT SUM(amount) FROM payments \
         WHERE created_at >= $1 AND (status = 'completed' OR status = 'confirmed')",
    )
    .bind(month_start)
    .fetch_one(payments_pool.as_ref())
    .await
    .unwrap_or(None);

    let monthly_revenue = monthly_revenue_bd
        .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
        .unwrap_or(0.0);

    let summary = SubscriptionSummary {
        total_subscriptions: total_count,
        active_subscriptions: active_count as u64,
        expired_subscriptions: expired_count as u64,
        cancelled_subscriptions: cancelled_count as u64,
        new_subscriptions_today: new_today as u64,
        expiring_soon: expiring_soon_count as u64,
        monthly_revenue,
    };

    info!(
        "Found {} subscriptions (page {} of {})",
        subscriptions_resp.len(),
        pg.page,
        total_pages
    );

    Ok(Json(AdminSubscriptionListResponse {
        success: true,
        subscriptions: subscriptions_resp,
        pagination,
        summary,
    }))
}
