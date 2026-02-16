//! Admin Subscription Handlers

use axum::{
    extract::{State, Query},
    response::Json,
};
use chrono::{Datelike, Utc};
use tracing::{info, error};

use crate::{
    web::{
        middleware::UnifiedErrorResponse,
        pagination::Pagination,
    },
    schemas::primary::plans,
};

use super::types::*;

/// Get all subscriptions
pub async fn admin_list_subscriptions_handler(
    State(app_state): State<crate::web::auth::AppState>,
    Query(params): Query<AdminPaymentListParams>, // Reuse same params
) -> Result<Json<AdminSubscriptionListResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::models::payment::SubscriptionDb;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::{payments, subscriptions};

    info!("Admin listing subscriptions with params: {:?}", params);

    // Get PAYMENTS database connection (subscriptions table is in payments DB)
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get payments database pool"))
    })?;
    let mut payments_conn = payments_pool.get().await
        .map_err(|e| {
            error!("Failed to get payments database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish payments database connection"))
        })?;

    // Get PRIMARY database connection (plans table is in primary DB)
    let mut primary_conn = app_state.db_pool.get().await
        .map_err(|e| {
            error!("Failed to get primary database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish primary database connection"))
        })?;

    // Build query
    let mut query = subscriptions::table.into_boxed();

    // Apply wallet_address filter if provided
    if let Some(ref wallet_addr) = params.wallet_address {
        query = query.filter(subscriptions::wallet_address.eq(wallet_addr));
    }

    // Apply plan_id filter if provided
    if let Some(ref plan_id) = params.plan_id {
        query = query.filter(subscriptions::plan_id.eq(plan_id));
    }

    // Apply status filter if provided
    if let Some(ref status) = params.status {
        query = query.filter(subscriptions::status.eq(status));
    }

    // Apply pagination
    let pg = Pagination::large(params.page, params.limit);

    // Get total count (before pagination) from PAYMENTS DB
    let total_count: i64 = subscriptions::table
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    // Fetch subscriptions with pagination from PAYMENTS DB
    let subscription_rows = query
        .order(subscriptions::started_at.desc().nulls_last())
        .limit(pg.limit as i64)
        .offset(pg.offset)
        .load::<SubscriptionDb>(&mut payments_conn)
        .await
        .map_err(|e| {
            error!("Failed to query subscriptions: {}", e);
            Json(UnifiedErrorResponse::new(500, "Query failed", format!("Failed to load subscriptions: {}", e)))
        })?;

    // Map to response format with plan name lookup from PRIMARY DB
    let mut subscriptions_resp: Vec<AdminSubscriptionInfo> = Vec::new();
    for sub_db in subscription_rows {
        // Try to get plan name from plans table (PRIMARY DB)
        let plan_name = plans::table
            .filter(plans::id.eq(sub_db.plan_id))
            .select(plans::name)
            .first::<String>(&mut primary_conn)
            .await
            .unwrap_or_else(|_| "Unknown Plan".to_string());

        subscriptions_resp.push(AdminSubscriptionInfo {
            id: sub_db.id,
            wallet_address: sub_db.wallet_address,
            plan_id: sub_db.plan_id,
            plan_name,
            status: sub_db.status,
            payment_id: sub_db.payment_id.unwrap_or(uuid::Uuid::nil()),
            started_at: sub_db.started_at.unwrap_or_else(Utc::now),
            expires_at: sub_db.expires_at,
            cancelled_at: sub_db.cancelled_at,
            auto_renew: sub_db.auto_renew.unwrap_or(false),
            metadata: sub_db.metadata.unwrap_or(serde_json::json!({})),
        });
    }

    let total_pages = pg.total_pages(total_count as u64);
    let pagination = PaginationInfo {
        page: pg.page,
        limit: pg.limit,
        total_count: total_count as u64,
        total_pages,
        has_next: pg.has_next(total_count as u64),
        has_prev: pg.has_prev(),
    };

    // Calculate summary statistics with real database queries
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
        total_subscriptions: total_count as u64,
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
