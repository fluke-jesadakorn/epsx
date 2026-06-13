//! Admin Subscription Handlers

use axum::{
    extract::{State, Query},
    response::Json,
};
use chrono::{Datelike, Utc};
use tracing::{info, error};
use uuid::Uuid;

use crate::{
    web::{
        middleware::UnifiedErrorResponse,
        pagination::Pagination,
    },
};

use super::types::*;

/// Get all subscriptions
///
/// Wave 11 / Track A: the two-conn
/// `subscriptions::table` (payments pool) +
/// `plans::table` (primary pool) reacharound is collapsed
/// to a single `PaymentRepositoryPort::list_admin_subscriptions_with_plan_names`
/// call. The port's LEFT JOIN against the plans table
/// runs against the payments pool today (both pools
/// share a schema until the wave-11 integration gate
/// replicates `plans` into the payments schema).
///
/// The 5 sub-aggregation queries (active / expired /
/// cancelled counts, new_today, expiring_soon,
/// monthly_revenue) stay on the payments pool — they
/// are single-table queries, no cross-pool join.
pub async fn admin_list_subscriptions_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Query(params): Query<AdminPaymentListParams>, // Reuse same params
) -> Result<Json<AdminSubscriptionListResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::subscriptions;
    use crate::domain::payment::repository_ports::SubscriptionFilters;

    info!("Admin listing subscriptions with params: {:?}", params);

    // Wave 11 / Track A: pull subscriptions + plan names
    // through the port (single LEFT JOIN).
    let payment_repo = app_state.payment_repo.as_ref().ok_or_else(|| {
        error!("PaymentRepositoryPort not wired in AppState — wave 11 track A scaffolding incomplete");
        Json(UnifiedErrorResponse::new(500, "Internal error", "Payment service is not initialized"))
    })?;

    // Apply pagination
    let pg = Pagination::large(params.page, params.limit);

    let filters = SubscriptionFilters {
        wallet_address: params.wallet_address.clone(),
        plan_id: params.plan_id,
        status: params.status.clone(),
    };
    let (rows, total_count) = payment_repo
        .list_admin_subscriptions_with_plan_names_paginated(
            filters.clone(),
            pg.page,
            pg.limit as u32,
        )
        .await
        .map_err(|e| {
            error!("Failed to list subscriptions: {}", e);
            Json(UnifiedErrorResponse::new(500, "Query failed", format!("Failed to load subscriptions: {}", e)))
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
    // (single-table queries, no cross-pool join)
    let mut payments_conn = {
        let payments_pool = get_payments_pool().await.map_err(|e| {
            error!("Failed to get payments database pool: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get payments database pool"))
        })?;
        payments_pool.get().await.map_err(|e| {
            error!("Failed to get payments database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish payments database connection"))
        })?
    };

    let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0)
        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);
    let seven_days_from_now = Utc::now() + chrono::Duration::days(7);
    let month_start = Utc::now().date_naive()
        .with_day(1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| chrono::DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or_else(Utc::now);

    // Get counts from database
    let active_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("active"))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let expired_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("expired"))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let cancelled_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("cancelled"))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    // New subscriptions today (started_at >= today_start)
    let new_today: i64 = subscriptions::table
        .filter(subscriptions::started_at.ge(today_start))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    // Expiring soon (active and expires_at <= 7 days from now)
    let expiring_soon_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("active"))
        .filter(subscriptions::expires_at.le(seven_days_from_now))
        .filter(subscriptions::expires_at.ge(Utc::now()))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    // Monthly revenue from payments (this month, completed/confirmed)
    use crate::schemas::payments::payments;
    let monthly_revenue_bd: Option<bigdecimal::BigDecimal> = payments::table
        .filter(payments::created_at.ge(month_start))
        .filter(payments::status.eq("completed")
            .or(payments::status.eq("confirmed")))
        .select(diesel::dsl::sum(payments::amount))
        .first(&mut payments_conn)
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

    info!("Found {} subscriptions (page {} of {})", subscriptions_resp.len(), pg.page, total_pages);

    Ok(Json(AdminSubscriptionListResponse {
        success: true,
        subscriptions: subscriptions_resp,
        pagination,
        summary,
    }))
}
