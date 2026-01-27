//! Payment Analytics Service
//!
//! Calculates and aggregates payment analytics for admin dashboards.
//! Extracted from admin_handlers.rs for better separation of concerns.

use uuid::Uuid;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::prelude::*;
use crate::web::middleware::UnifiedErrorResponse;
use crate::schemas::primary::{plans};
use crate::schemas::payments::subscriptions;
use crate::infrastructure::database::get_payments_pool;

/// Payment analytics data
#[derive(Debug, serde::Serialize)]
pub struct PaymentAnalytics {
    pub daily_revenue: Vec<DailyRevenue>,
    pub plan_breakdown: Vec<PlanBreakdown>,
    pub payment_methods: Vec<PaymentMethodStats>,
    pub trends: PaymentTrends,
}

/// Daily revenue data
#[derive(Debug, serde::Serialize)]
pub struct DailyRevenue {
    pub date: String,
    pub revenue: f64,
    pub payment_count: u32,
}

/// Plan breakdown data
#[derive(Debug, serde::Serialize)]
pub struct PlanBreakdown {
    pub plan_id: Uuid,
    pub plan_name: String,
    pub subscription_count: u32,
    pub revenue: f64,
    pub average_revenue_per_user: f64,
}

/// Payment method statistics
#[derive(Debug, serde::Serialize)]
pub struct PaymentMethodStats {
    pub method: String,
    pub count: u32,
    pub revenue: f64,
    pub success_rate: f64,
}

/// Payment trends
#[derive(Debug, serde::Serialize)]
pub struct PaymentTrends {
    pub growth_rate: f64,
    pub churn_rate: f64,
    pub average_subscription_length: f64,
    pub customer_lifetime_value: f64,
}

/// Service for calculating payment analytics
pub struct PaymentAnalyticsService;

impl PaymentAnalyticsService {
    /// Calculate comprehensive payment analytics
    pub async fn calculate(
        app_state: &crate::web::auth::AppState,
    ) -> Result<PaymentAnalytics, Json<UnifiedErrorResponse>> {
        use crate::schemas::payments::payments;

        // Get PAYMENTS database connection
        let payments_pool = get_payments_pool().await.map_err(|e| {
            tracing::error!("Failed to get payments database pool: {}", e);
            Json(create_error_response(500, "Database connection failed", "Failed to get payments database pool"))
        })?;
        let mut payments_conn = payments_pool.get().await
            .map_err(|e| {
                tracing::error!("Failed to get payments database connection: {}", e);
                Json(create_error_response(500, "Database connection failed", "Failed to establish payments database connection"))
            })?;

        // Get PRIMARY database connection for plan name lookup
        let mut primary_conn = app_state.db_pool.get().await
            .map_err(|e| {
                tracing::error!("Failed to get primary database connection: {}", e);
                Json(create_error_response(500, "Database connection failed", "Failed to establish primary database connection"))
            })?;

        // Calculate all analytics components
        let daily_revenue = Self::calculate_daily_revenue(&mut payments_conn).await;
        let plan_breakdown = Self::calculate_plan_breakdown(&mut payments_conn, &mut primary_conn).await;
        let payment_methods = Self::calculate_payment_methods(&mut payments_conn).await;
        let trends = Self::calculate_trends(&mut payments_conn, &mut primary_conn).await?;

        Ok(PaymentAnalytics {
            daily_revenue,
            plan_breakdown,
            payment_methods,
            trends,
        })
    }

    /// Calculate daily revenue for last 30 days
    async fn calculate_daily_revenue(conn: &mut async_pg::PgConnection) -> Vec<DailyRevenue> {
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

        let rows = diesel::sql_query(
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
        .load::<DailyRevenueRow>(conn)
        .await
        .unwrap_or_default();

        rows.into_iter().map(|row| {
            DailyRevenue {
                date: row.payment_date.format("%Y-%m-%d").to_string(),
                revenue: row.daily_revenue
                    .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
                    .unwrap_or(0.0),
                payment_count: row.payment_count as u32,
            }
        }).collect()
    }

    /// Calculate plan breakdown
    async fn calculate_plan_breakdown(
        payments_conn: &mut async_pg::PgConnection,
        primary_conn: &mut async_pg::PgConnection,
    ) -> Vec<PlanBreakdown> {
        #[derive(diesel::QueryableByName)]
        struct PlanBreakdownRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            plan_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
            total_revenue: Option<bigdecimal::BigDecimal>,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            subscription_count: i64,
        }

        let rows = diesel::sql_query(
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
        .load::<PlanBreakdownRow>(payments_conn)
        .await
        .unwrap_or_default();

        let mut result = Vec::new();
        for row in rows {
            let plan_name = plans::table
                .filter(plans::id.eq(row.plan_id))
                .select(plans::name)
                .first::<String>(primary_conn)
                .await
                .unwrap_or_else(|_| "Unknown Plan".to_string());

            let revenue = row.total_revenue
                .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
                .unwrap_or(0.0);
            let count = row.subscription_count as u32;
            let arpu = if count > 0 { revenue / count as f64 } else { 0.0 };

            result.push(PlanBreakdown {
                plan_id: row.plan_id,
                plan_name,
                subscription_count: count,
                revenue,
                average_revenue_per_user: arpu,
            });
        }
        result
    }

    /// Calculate payment method statistics
    async fn calculate_payment_methods(conn: &mut async_pg::PgConnection) -> Vec<PaymentMethodStats> {
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

        let rows = diesel::sql_query(
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
        .load::<PaymentMethodRow>(conn)
        .await
        .unwrap_or_default();

        rows.into_iter().map(|row| {
            let total = row.payment_count as f64;
            let success = row.successful_count as f64;
            PaymentMethodStats {
                method: row.currency,
                count: row.payment_count as u32,
                revenue: row.total_revenue
                    .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
                    .unwrap_or(0.0),
                success_rate: if total > 0.0 { (success / total) * 100.0 } else { 0.0 },
            }
        }).collect()
    }

    /// Calculate payment trends
    async fn calculate_trends(
        payments_conn: &mut async_pg::PgConnection,
        primary_conn: &mut async_pg::PgConnection,
    ) -> Result<PaymentTrends, Json<UnifiedErrorResponse>> {
        let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
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
        .get_result(payments_conn)
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
        .get_result(payments_conn)
        .await
        .unwrap_or(PeriodStats { total_revenue: None, total_count: 0 });

        let current_rev = current_period.total_revenue
            .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);
        let previous_rev = previous_period.total_revenue
            .map(|bd| bd.to_string().parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        let growth_rate = if previous_rev > 0.0 {
            ((current_rev - previous_rev) / previous_rev) * 100.0
        } else if current_rev > 0.0 {
            100.0
        } else {
            0.0
        };

        // Calculate average subscription length
        let avg_sub_length: f64 = subscriptions::table
            .filter(subscriptions::status.eq("active"))
            .select(diesel::dsl::avg(diesel::dsl::sql::<diesel::sql_types::Float>(
                "EXTRACT(EPOCH FROM (expires_at - started_at)) / 86400.0"
            )))
            .first::<Option<f64>>(payments_conn)
            .await
            .ok()
            .flatten()
            .unwrap_or(30.0);

        // Churn rate estimate
        let cancelled_count: i64 = subscriptions::table
            .filter(subscriptions::status.eq("cancelled"))
            .filter(subscriptions::cancelled_at.ge(thirty_days_ago))
            .count()
            .get_result(payments_conn)
            .await
            .unwrap_or(0);

        let active_count: i64 = subscriptions::table
            .filter(subscriptions::status.eq("active"))
            .count()
            .get_result(payments_conn)
            .await
            .unwrap_or(1);

        let churn_rate = (cancelled_count as f64 / active_count.max(1) as f64) * 100.0;

        // Customer lifetime value
        let avg_payment = if current_period.total_count > 0 {
            current_rev / current_period.total_count as f64
        } else {
            0.0
        };
        let customer_lifetime_value = avg_payment * (avg_sub_length / 30.0);

        Ok(PaymentTrends {
            growth_rate,
            churn_rate,
            average_subscription_length: avg_sub_length,
            customer_lifetime_value,
        })
    }
}

/// Helper function to create UnifiedErrorResponse
fn create_error_response(code: u16, message: &str, reason: &str) -> UnifiedErrorResponse {
    UnifiedErrorResponse {
        success: false,
        error: crate::web::middleware::bearer_middleware::ErrorDetails {
            code,
            message: message.to_string(),
            reason: reason.to_string(),
        },
    }
}
