//! Admin Payment Analytics Handlers
//!
//! Wave 11 / Track A: the 4 separate `diesel::sql_query` blocks
//! (daily revenue, plan breakdown, payment methods, trends) +
//! the cross-pool `plans::table` join collapse to a single
//! `PaymentRepositoryPort::get_analytics_rollup(window)` call.
//! The port method runs each of the 4 aggregations as a
//! single SQL query (with the `plans` JOIN done inline on the
//! payments side).

use axum::{
    extract::State,
    response::Json,
};
use tracing::{info, error};

use crate::{
    web::middleware::UnifiedErrorResponse,
};

use super::types::*;

pub async fn admin_get_payment_analytics_handler(
    State(app_state): State<crate::web::auth::AppState>,
) -> Result<Json<PaymentAnalyticsResponse>, Json<UnifiedErrorResponse>> {
    use crate::domain::payment::repository_ports::AnalyticsWindow;

    info!("Admin getting payment analytics");

    // Wave 11 / Track A: collapse the 4 sql_query blocks +
    // the cross-pool `plans::table` lookup to a single port
    // call. The window is the last-30-days default that the
    // previous handler used.
    let payment_repo = app_state.payment_repo.as_ref().ok_or_else(|| {
        error!("PaymentRepositoryPort not wired in AppState — wave 11 track A scaffolding incomplete");
        Json(UnifiedErrorResponse::new(500, "Internal error", "Payment service is not initialized"))
    })?;
    let rollup = payment_repo
        .get_analytics_rollup(AnalyticsWindow::Last30Days)
        .await
        .map_err(|e| {
            error!("Failed to get analytics rollup: {}", e);
            Json(UnifiedErrorResponse::new(500, "Query failed", format!("Failed to load analytics: {}", e)))
        })?;

    // Convert the port DTOs to the response DTOs.
    let daily_revenue: Vec<DailyRevenue> = rollup
        .daily_revenue
        .into_iter()
        .map(|d| DailyRevenue {
            date: d.date,
            revenue: d.revenue,
            payment_count: d.payment_count,
        })
        .collect();
    let plan_breakdown: Vec<PlanBreakdown> = rollup
        .plan_breakdown
        .into_iter()
        .map(|p| PlanBreakdown {
            plan_id: p.plan_id,
            plan_name: p.plan_name,
            subscription_count: p.subscription_count,
            revenue: p.total_revenue,
            average_revenue_per_user: p.average_revenue_per_user,
        })
        .collect();
    let payment_methods: Vec<PaymentMethodStats> = rollup
        .payment_methods
        .into_iter()
        .map(|m| PaymentMethodStats {
            method: m.currency,
            count: m.payment_count,
            revenue: m.total_revenue,
            success_rate: m.success_rate,
        })
        .collect();
    let trends = PaymentTrends {
        growth_rate: rollup.trends.growth_rate,
        churn_rate: rollup.trends.churn_rate,
        average_subscription_length: rollup.trends.average_subscription_length,
        customer_lifetime_value: rollup.trends.customer_lifetime_value,
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
