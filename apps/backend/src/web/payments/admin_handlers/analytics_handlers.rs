//! Admin Payment Analytics Handlers

use axum::{
    extract::State,
    response::Json,
};
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error};

use crate::{
    web::middleware::UnifiedErrorResponse,
    schemas::primary::plans,
};

use super::types::*;

pub async fn admin_get_payment_analytics_handler(
    State(app_state): State<crate::web::auth::AppState>,
) -> Result<Json<PaymentAnalyticsResponse>, Json<UnifiedErrorResponse>> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    use crate::infrastructure::database::get_payments_pool;
    use crate::schemas::payments::subscriptions;

    info!("Admin getting payment analytics");

    // Get PAYMENTS database connection
    let payments_pool = get_payments_pool().await.map_err(|e| {
        error!("Failed to get payments database pool: {}", e);
        Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to get payments database pool"))
    })?;
    let mut payments_conn = payments_pool.get().await
        .map_err(|e| {
            error!("Failed to get payments database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish payments database connection"))
        })?;

    // Get PRIMARY database connection for plan name lookup
    let mut primary_conn = app_state.db_pool.get().await
        .map_err(|e| {
            error!("Failed to get primary database connection: {}", e);
            Json(UnifiedErrorResponse::new(500, "Database connection failed", "Failed to establish primary database connection"))
        })?;

    // === 1. Daily Revenue (last 30 days) ===
    let thirty_days_ago = Utc::now() - chrono::Duration::days(30);

    #[derive(diesel::QueryableByName)]
    struct DailyRevenueRow {
        #[diesel(sql_type = diesel::sql_types::Date)]
        payment_date: chrono::NaiveDate,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        daily_revenue: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        payment_count: i64,
    }

    let daily_revenue_rows = diesel::sql_query(
        r#"
        SELECT
            DATE(created_at) as payment_date,
            SUM(amount) as daily_revenue,
            COUNT(*) as payment_count
        FROM payments
        WHERE created_at >= $1
          AND (status = 'completed' OR status = 'confirmed')
        GROUP BY DATE(created_at)
        ORDER BY payment_date DESC
        LIMIT 30
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(thirty_days_ago)
    .load::<DailyRevenueRow>(&mut payments_conn)
    .await
    .unwrap_or_default();

    let daily_revenue: Vec<DailyRevenue> = daily_revenue_rows.into_iter().map(|row| {
        DailyRevenue {
            date: row.payment_date.format("%Y-%m-%d").to_string(),
            revenue: row.daily_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            payment_count: row.payment_count as u32,
        }
    }).collect();

    // === 2. Plan Breakdown ===
    #[derive(diesel::QueryableByName)]
    struct PlanBreakdownRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        plan_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total_revenue: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        subscription_count: i64,
    }

    let plan_rows = diesel::sql_query(
        r#"
        SELECT
            plan_id,
            SUM(amount) as total_revenue,
            COUNT(*) as subscription_count
        FROM payments
        WHERE status = 'completed' OR status = 'confirmed'
        GROUP BY plan_id
        ORDER BY total_revenue DESC NULLS LAST
        LIMIT 10
        "#
    )
    .load::<PlanBreakdownRow>(&mut payments_conn)
    .await
    .unwrap_or_default();

    let mut plan_breakdown: Vec<PlanBreakdown> = Vec::new();
    for row in plan_rows {
        // Get plan name from plans table
        let plan_name = plans::table
            .filter(plans::id.eq(row.plan_id))
            .select(plans::name)
            .first::<String>(&mut primary_conn)
            .await
            .unwrap_or_else(|_| "Unknown Plan".to_string());

        let revenue = row.total_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
        let count = row.subscription_count as u32;
        let arpu = if count > 0 { revenue / count as f64 } else { 0.0 };

        plan_breakdown.push(PlanBreakdown {
            plan_id: row.plan_id,
            plan_name,
            subscription_count: count,
            revenue,
            average_revenue_per_user: arpu,
        });
    }

    // === 3. Payment Methods (by currency/token) ===
    #[derive(diesel::QueryableByName)]
    struct PaymentMethodRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        currency: String,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        payment_count: i64,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total_revenue: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        successful_count: i64,
    }

    let method_rows = diesel::sql_query(
        r#"
        SELECT
            currency,
            COUNT(*) as payment_count,
            SUM(CASE WHEN status IN ('completed', 'confirmed') THEN amount ELSE 0 END) as total_revenue,
            SUM(CASE WHEN status IN ('completed', 'confirmed') THEN 1 ELSE 0 END) as successful_count
        FROM payments
        GROUP BY currency
        ORDER BY payment_count DESC
        "#
    )
    .load::<PaymentMethodRow>(&mut payments_conn)
    .await
    .unwrap_or_default();

    let payment_methods: Vec<PaymentMethodStats> = method_rows.into_iter().map(|row| {
        let total = row.payment_count as f64;
        let success = row.successful_count as f64;
        PaymentMethodStats {
            method: row.currency,
            count: row.payment_count as u32,
            revenue: row.total_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0),
            success_rate: if total > 0.0 { (success / total) * 100.0 } else { 0.0 },
        }
    }).collect();

    // === 4. Payment Trends ===
    // Calculate growth rate (compare last 30 days vs previous 30 days)
    let sixty_days_ago = Utc::now() - chrono::Duration::days(60);

    #[derive(diesel::QueryableByName)]
    struct PeriodStats {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total_revenue: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        total_count: i64,
    }

    let current_period: PeriodStats = diesel::sql_query(
        r#"
        SELECT COALESCE(SUM(amount), 0) as total_revenue, COUNT(*) as total_count
        FROM payments
        WHERE created_at >= $1 AND (status = 'completed' OR status = 'confirmed')
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(thirty_days_ago)
    .get_result(&mut payments_conn)
    .await
    .unwrap_or(PeriodStats { total_revenue: None, total_count: 0 });

    let previous_period: PeriodStats = diesel::sql_query(
        r#"
        SELECT COALESCE(SUM(amount), 0) as total_revenue, COUNT(*) as total_count
        FROM payments
        WHERE created_at >= $1 AND created_at < $2 AND (status = 'completed' OR status = 'confirmed')
        "#
    )
    .bind::<diesel::sql_types::Timestamptz, _>(sixty_days_ago)
    .bind::<diesel::sql_types::Timestamptz, _>(thirty_days_ago)
    .get_result(&mut payments_conn)
    .await
    .unwrap_or(PeriodStats { total_revenue: None, total_count: 0 });

    let current_rev = current_period.total_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);
    let previous_rev = previous_period.total_revenue.map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);

    let growth_rate = if previous_rev > 0.0 {
        ((current_rev - previous_rev) / previous_rev) * 100.0
    } else if current_rev > 0.0 {
        100.0 // 100% growth if previous was 0
    } else {
        0.0
    };

    // Calculate average subscription length from subscriptions table
    let avg_sub_length: f64 = subscriptions::table
        .filter(subscriptions::status.eq("active"))
        .select(diesel::dsl::avg(diesel::dsl::sql::<diesel::sql_types::Float>("EXTRACT(EPOCH FROM (expires_at - started_at)) / 86400.0")))
        .first::<Option<f64>>(&mut payments_conn)
        .await
        .ok()
        .flatten()
        .unwrap_or(30.0);

    // Churn rate estimate (cancelled in last 30 days / total active)
    let cancelled_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("cancelled"))
        .filter(subscriptions::cancelled_at.ge(thirty_days_ago))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(0);

    let active_count: i64 = subscriptions::table
        .filter(subscriptions::status.eq("active"))
        .count()
        .get_result(&mut payments_conn)
        .await
        .unwrap_or(1); // Avoid division by zero

    let churn_rate = (cancelled_count as f64 / active_count.max(1) as f64) * 100.0;

    // Customer lifetime value estimate (average revenue * average subscription length / 30)
    let avg_payment: f64 = if current_period.total_count > 0 {
        current_rev / current_period.total_count as f64
    } else {
        0.0
    };
    let customer_lifetime_value = avg_payment * (avg_sub_length / 30.0);

    let trends = PaymentTrends {
        growth_rate,
        churn_rate,
        average_subscription_length: avg_sub_length,
        customer_lifetime_value,
    };

    let analytics = PaymentAnalytics {
        daily_revenue,
        plan_breakdown,
        payment_methods,
        trends,
    };

    info!("Successfully retrieved payment analytics");

    Ok(Json(PaymentAnalyticsResponse {
        success: true,
        analytics,
    }))
}
