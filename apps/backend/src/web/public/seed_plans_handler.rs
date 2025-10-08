use axum::{
    response::Json,
    http::StatusCode,
    extract::State,
};
use serde_json::{json, Value};
use crate::web::auth::AppState;
use sqlx::types::BigDecimal;
use std::str::FromStr;

/// Seed subscription plans (development/testing only)
/// POST /api/public/plans/seed
///
/// SAFETY: Should be disabled in production or require admin auth
#[utoipa::path(
    post,
    path = "/api/public/plans/seed",
    tag = "public",
    responses(
        (status = 200, description = "Successfully seeded subscription plans"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn seed_subscription_plans(State(app_state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    tracing::info!("🌱 Seeding subscription plans...");

    let pool = &*app_state.db_pool;

    // Free Plan
    let free_plan_result = sqlx::query!(
        r#"
        INSERT INTO permission_groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO NOTHING
        "#,
        "Free Plan",
        "free",
        "Perfect for getting started with basic analytics",
        "subscription",
        json!({
            "permissions": ["epsx:analytics:view:5", "epsx:rankings:view:5"],
            "features": [
                "Basic analytics view",
                "5 stock rankings limit",
                "Community support",
                "Daily market updates"
            ],
            "limits": {
                "analytics_queries_per_day": 10,
                "stocks_tracked": 5,
                "historical_data_months": 1
            }
        }),
        BigDecimal::from_str("0.00").ok(),
        "USD",
        "monthly",
        true,
        false,
        1,
        "0x0000000000000000000000000000000000000000" // System wallet address
    )
    .execute(pool)
    .await;

    // Starter Plan
    let starter_plan_result = sqlx::query!(
        r#"
        INSERT INTO permission_groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO NOTHING
        "#,
        "Starter Plan",
        "starter",
        "Ideal for individual investors and traders",
        "subscription",
        json!({
            "permissions": ["epsx:analytics:view:25", "epsx:rankings:view:25", "epsx:analytics:export", "epsx:alerts:create"],
            "features": [
                "Advanced analytics",
                "25 stock rankings",
                "Export functionality",
                "Price alerts",
                "Email support"
            ],
            "limits": {
                "analytics_queries_per_day": 50,
                "stocks_tracked": 25,
                "historical_data_months": 6,
                "alerts": 10
            }
        }),
        BigDecimal::from_str("14.99").ok(),
        "USD",
        "monthly",
        true,
        false,
        2,
        "0x0000000000000000000000000000000000000000" // System wallet address
    )
    .execute(pool)
    .await;

    // Pro Plan
    let pro_plan_result = sqlx::query!(
        r#"
        INSERT INTO permission_groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO NOTHING
        "#,
        "Pro Plan",
        "pro",
        "For serious traders who need advanced tools",
        "subscription",
        json!({
            "permissions": ["epsx:analytics:view:100", "epsx:rankings:view:100", "epsx:analytics:export", "epsx:analytics:advanced", "epsx:alerts:create", "epsx:alerts:manage", "epsx:portfolio:view", "epsx:portfolio:manage"],
            "features": [
                "Advanced analytics",
                "100 stock rankings",
                "Export functionality",
                "Advanced charting tools",
                "Portfolio management",
                "Unlimited price alerts",
                "Priority support",
                "Real-time data"
            ],
            "limits": {
                "analytics_queries_per_day": 200,
                "stocks_tracked": 100,
                "historical_data_months": 24,
                "alerts": -1,
                "portfolios": 5
            },
            "highlighted": true
        }),
        BigDecimal::from_str("29.99").ok(),
        "USD",
        "monthly",
        true,
        true,
        3,
        "0x0000000000000000000000000000000000000000" // System wallet address
    )
    .execute(pool)
    .await;

    // Enterprise Plan
    let enterprise_plan_result = sqlx::query!(
        r#"
        INSERT INTO permission_groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO NOTHING
        "#,
        "Enterprise Plan",
        "enterprise",
        "Complete solution for professional teams and institutions",
        "subscription",
        json!({
            "permissions": ["epsx:*:*", "epsx:api:access", "epsx:enterprise:*"],
            "features": [
                "Unlimited stock analysis",
                "Unlimited rankings access",
                "Full API access",
                "Premium analytics suite",
                "Advanced portfolio tools",
                "Custom integrations",
                "Dedicated account manager",
                "24/7 priority support",
                "White-label options"
            ],
            "limits": {
                "analytics_queries_per_day": -1,
                "stocks_tracked": -1,
                "historical_data_months": -1,
                "alerts": -1,
                "portfolios": -1,
                "api_calls_per_month": 1000000
            },
            "highlighted": false,
            "contact_sales": true
        }),
        BigDecimal::from_str("99.99").ok(),
        "USD",
        "monthly",
        true,
        false,
        4,
        "0x0000000000000000000000000000000000000000" // System wallet address
    )
    .execute(pool)
    .await;

    // API Developer Plan
    let api_plan_result = sqlx::query!(
        r#"
        INSERT INTO permission_groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO NOTHING
        "#,
        "API Developer",
        "api-developer",
        "For developers building on EPSX platform",
        "subscription",
        json!({
            "permissions": ["epsx:api:access", "epsx:analytics:view:unlimited", "epsx:rankings:view:unlimited"],
            "features": [
                "Full REST API access",
                "Unlimited analytics queries",
                "Unlimited rankings access",
                "WebSocket support",
                "API documentation",
                "Developer support",
                "100k API calls/month"
            ],
            "limits": {
                "api_calls_per_month": 100000,
                "websocket_connections": 5,
                "rate_limit_per_second": 10
            },
            "plan_type": "api"
        }),
        BigDecimal::from_str("49.99").ok(),
        "USD",
        "monthly",
        true,
        false,
        5,
        "0x0000000000000000000000000000000000000000" // System wallet address
    )
    .execute(pool)
    .await;

    // Check results
    let mut inserted = 0;
    let mut errors = Vec::new();

    if free_plan_result.is_ok() { inserted += 1; } else { errors.push("Free Plan"); }
    if starter_plan_result.is_ok() { inserted += 1; } else { errors.push("Starter Plan"); }
    if pro_plan_result.is_ok() { inserted += 1; } else { errors.push("Pro Plan"); }
    if enterprise_plan_result.is_ok() { inserted += 1; } else { errors.push("Enterprise Plan"); }
    if api_plan_result.is_ok() { inserted += 1; } else { errors.push("API Developer"); }

    // Get total count
    let total_plans = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM permission_groups WHERE group_type = 'subscription'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    tracing::info!("✅ Seeded {} subscription plans. Total in database: {}", inserted, total_plans);

    Ok(Json(json!({
        "success": true,
        "message": format!("Seeded {} subscription plans (Total: {})", inserted, total_plans),
        "data": {
            "plans_inserted": inserted,
            "total_plans": total_plans,
            "errors": errors
        }
    })))
}
