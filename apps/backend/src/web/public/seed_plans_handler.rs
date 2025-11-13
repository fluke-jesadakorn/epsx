use axum::{
    response::Json,
    http::StatusCode,
    extract::State,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde_json::{json, Value};
use crate::web::auth::AppState;
use bigdecimal::BigDecimal;
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

    let mut conn = match app_state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Free Plan
    let free_plan_metadata = json!({
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
    });

    let free_plan_result = diesel::sql_query(
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
        "#
    )
    .bind::<diesel::sql_types::Text, _>("Free Plan")
    .bind::<diesel::sql_types::Text, _>("free")
    .bind::<diesel::sql_types::Text, _>("Perfect for getting started with basic analytics")
    .bind::<diesel::sql_types::Text, _>("subscription")
    .bind::<diesel::sql_types::Jsonb, _>(&free_plan_metadata)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Numeric>, _>(BigDecimal::from_str("0.00").ok())
    .bind::<diesel::sql_types::Text, _>("USD")
    .bind::<diesel::sql_types::Text, _>("monthly")
    .bind::<diesel::sql_types::Bool, _>(true)
    .bind::<diesel::sql_types::Bool, _>(false)
    .bind::<diesel::sql_types::Integer, _>(1)
    .bind::<diesel::sql_types::Text, _>("0x0000000000000000000000000000000000000000")
    .execute(&mut conn)
    .await;

    // Starter Plan
    let starter_plan_metadata = json!({
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
    });

    let starter_plan_result = diesel::sql_query(
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
        "#
    )
    .bind::<diesel::sql_types::Text, _>("Starter Plan")
    .bind::<diesel::sql_types::Text, _>("starter")
    .bind::<diesel::sql_types::Text, _>("Ideal for individual investors and traders")
    .bind::<diesel::sql_types::Text, _>("subscription")
    .bind::<diesel::sql_types::Jsonb, _>(&starter_plan_metadata)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Numeric>, _>(BigDecimal::from_str("14.99").ok())
    .bind::<diesel::sql_types::Text, _>("USD")
    .bind::<diesel::sql_types::Text, _>("monthly")
    .bind::<diesel::sql_types::Bool, _>(true)
    .bind::<diesel::sql_types::Bool, _>(false)
    .bind::<diesel::sql_types::Integer, _>(2)
    .bind::<diesel::sql_types::Text, _>("0x0000000000000000000000000000000000000000")
    .execute(&mut conn)
    .await;

    // Pro Plan
    let pro_plan_metadata = json!({
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
    });

    let pro_plan_result = diesel::sql_query(
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
        "#
    )
    .bind::<diesel::sql_types::Text, _>("Pro Plan")
    .bind::<diesel::sql_types::Text, _>("pro")
    .bind::<diesel::sql_types::Text, _>("For serious traders who need advanced tools")
    .bind::<diesel::sql_types::Text, _>("subscription")
    .bind::<diesel::sql_types::Jsonb, _>(&pro_plan_metadata)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Numeric>, _>(BigDecimal::from_str("29.99").ok())
    .bind::<diesel::sql_types::Text, _>("USD")
    .bind::<diesel::sql_types::Text, _>("monthly")
    .bind::<diesel::sql_types::Bool, _>(true)
    .bind::<diesel::sql_types::Bool, _>(true)
    .bind::<diesel::sql_types::Integer, _>(3)
    .bind::<diesel::sql_types::Text, _>("0x0000000000000000000000000000000000000000")
    .execute(&mut conn)
    .await;

    // Enterprise Plan
    let enterprise_plan_metadata = json!({
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
    });

    let enterprise_plan_result = diesel::sql_query(
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
        "#
    )
    .bind::<diesel::sql_types::Text, _>("Enterprise Plan")
    .bind::<diesel::sql_types::Text, _>("enterprise")
    .bind::<diesel::sql_types::Text, _>("Complete solution for professional teams and institutions")
    .bind::<diesel::sql_types::Text, _>("subscription")
    .bind::<diesel::sql_types::Jsonb, _>(&enterprise_plan_metadata)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Numeric>, _>(BigDecimal::from_str("99.99").ok())
    .bind::<diesel::sql_types::Text, _>("USD")
    .bind::<diesel::sql_types::Text, _>("monthly")
    .bind::<diesel::sql_types::Bool, _>(true)
    .bind::<diesel::sql_types::Bool, _>(false)
    .bind::<diesel::sql_types::Integer, _>(4)
    .bind::<diesel::sql_types::Text, _>("0x0000000000000000000000000000000000000000")
    .execute(&mut conn)
    .await;

    // API Developer Plan
    let api_plan_metadata = json!({
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
    });

    let api_plan_result = diesel::sql_query(
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
        "#
    )
    .bind::<diesel::sql_types::Text, _>("API Developer")
    .bind::<diesel::sql_types::Text, _>("api-developer")
    .bind::<diesel::sql_types::Text, _>("For developers building on EPSX platform")
    .bind::<diesel::sql_types::Text, _>("subscription")
    .bind::<diesel::sql_types::Jsonb, _>(&api_plan_metadata)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Numeric>, _>(BigDecimal::from_str("49.99").ok())
    .bind::<diesel::sql_types::Text, _>("USD")
    .bind::<diesel::sql_types::Text, _>("monthly")
    .bind::<diesel::sql_types::Bool, _>(true)
    .bind::<diesel::sql_types::Bool, _>(false)
    .bind::<diesel::sql_types::Integer, _>(5)
    .bind::<diesel::sql_types::Text, _>("0x0000000000000000000000000000000000000000")
    .execute(&mut conn)
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
    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let total_plans = diesel::sql_query(
        "SELECT COUNT(*) as count FROM permission_groups WHERE group_type = 'subscription'"
    )
    .get_result::<CountRow>(&mut conn)
    .await
    .map(|r| r.count)
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
