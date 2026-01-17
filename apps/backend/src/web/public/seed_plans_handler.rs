use crate::web::api_response::ApiResponse;
use serde::Serialize;
use utoipa::ToSchema;
use diesel::QueryableByName;
use serde_json::json;
use bigdecimal::BigDecimal;
use std::str::FromStr;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use crate::web::auth::AppState;
use diesel_async::RunQueryDsl; // Ensure async execute is used

#[derive(Serialize, ToSchema)]
pub struct SeedPlansResponse {
    pub plans_inserted: i32,
    pub total_plans: i64,
    pub errors: Vec<String>,
}



/// POST /api/public/plans/seed
///
/// SAFETY: Should be disabled in production or require admin auth
#[utoipa::path(
    post,
    path = "/api/public/plans/seed",
    tag = "public",
    responses(
        (status = 200, description = "Successfully seeded subscription plans", body = ApiResponse<SeedPlansResponse>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn seed_subscription_plans(State(app_state): State<AppState>) -> (StatusCode, Json<ApiResponse<SeedPlansResponse>>) {
    tracing::info!("🌱 Seeding subscription plans...");

    let mut conn = match app_state.db_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to get database connection: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DB_CONNECTION_ERROR", "Failed to connect to database"))
            );
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
        "ranking_offset": 100, 
        "rankings_limit": 5,
        "limits": {
            "analytics_queries_per_day": 10,
            "stocks_tracked": 5,
            "historical_data_months": 1
        }
    });


    let free_plan_result = diesel::sql_query(
        r#"
        INSERT INTO groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            group_type = EXCLUDED.group_type,
            group_metadata = EXCLUDED.group_metadata,
            price = EXCLUDED.price,
            currency = EXCLUDED.currency,
            billing_cycle = EXCLUDED.billing_cycle,
            is_active = EXCLUDED.is_active,
            is_promoted = EXCLUDED.is_promoted,
            display_order = EXCLUDED.display_order,
            created_by = EXCLUDED.created_by
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
        "ranking_offset": 1,
        "rankings_limit": 25,
        "limits": {
            "analytics_queries_per_day": 50,
            "stocks_tracked": 25,
            "historical_data_months": 6,
            "alerts": 10
        }
    });

    let starter_plan_result = diesel::sql_query(
        r#"
        INSERT INTO groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            group_type = EXCLUDED.group_type,
            group_metadata = EXCLUDED.group_metadata,
            price = EXCLUDED.price,
            currency = EXCLUDED.currency,
            billing_cycle = EXCLUDED.billing_cycle,
            is_active = EXCLUDED.is_active,
            is_promoted = EXCLUDED.is_promoted,
            display_order = EXCLUDED.display_order,
            created_by = EXCLUDED.created_by
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
        "ranking_offset": 1,
        "rankings_limit": 100,
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
        INSERT INTO groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            group_type = EXCLUDED.group_type,
            group_metadata = EXCLUDED.group_metadata,
            price = EXCLUDED.price,
            currency = EXCLUDED.currency,
            billing_cycle = EXCLUDED.billing_cycle,
            is_active = EXCLUDED.is_active,
            is_promoted = EXCLUDED.is_promoted,
            display_order = EXCLUDED.display_order,
            created_by = EXCLUDED.created_by
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
        INSERT INTO groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            group_type = EXCLUDED.group_type,
            group_metadata = EXCLUDED.group_metadata,
            price = EXCLUDED.price,
            currency = EXCLUDED.currency,
            billing_cycle = EXCLUDED.billing_cycle,
            is_active = EXCLUDED.is_active,
            is_promoted = EXCLUDED.is_promoted,
            display_order = EXCLUDED.display_order,
            created_by = EXCLUDED.created_by
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
        INSERT INTO groups (
            id, name, slug, description, group_type, group_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by
        ) VALUES (
            gen_random_uuid(),
            $1, $2, $3, $4,
            $5::jsonb,
            $6, $7, $8, $9, $10, $11, $12
        )
        ON CONFLICT (slug) DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            group_type = EXCLUDED.group_type,
            group_metadata = EXCLUDED.group_metadata,
            price = EXCLUDED.price,
            currency = EXCLUDED.currency,
            billing_cycle = EXCLUDED.billing_cycle,
            is_active = EXCLUDED.is_active,
            is_promoted = EXCLUDED.is_promoted,
            display_order = EXCLUDED.display_order,
            created_by = EXCLUDED.created_by
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

    // Seed group permissions
    
    // Helper to seed permissions
    async fn seed_permissions_for_group(
        conn: &mut diesel_async::AsyncPgConnection,
        slug: &str,
        perms: &[&str],
        platform_default: &str,
    ) -> Result<(), String> {
        // 1. Get Group ID
        #[derive(QueryableByName)]
        struct GId { #[diesel(sql_type = diesel::sql_types::Uuid)] id: uuid::Uuid }
        
        let group_id = diesel::sql_query("SELECT id FROM groups WHERE slug = $1")
            .bind::<diesel::sql_types::Text, _>(slug)
            .get_result::<GId>(conn)
            .await
            .map_err(|e| format!("Group {} not found: {}", slug, e))?
            .id;

        for p_str in perms {
            // Split parts for metadata (rough approximation)
            let parts: Vec<&str> = p_str.split(':').collect();
            let platform = parts.get(0).unwrap_or(&platform_default);
            let resource = parts.get(1).unwrap_or(&"unknown");
            let action = parts.get(2).unwrap_or(&"access");

            // 2. Insert Permission
            diesel::sql_query(
                r#"
                INSERT INTO permissions (id, permission_string, platform, resource, action, permission_type)
                VALUES (gen_random_uuid(), $1, $2, $3, $4, 'manual')
                ON CONFLICT (permission_string) DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Text, _>(p_str)
            .bind::<diesel::sql_types::Text, _>(platform)
            .bind::<diesel::sql_types::Text, _>(resource)
            .bind::<diesel::sql_types::Text, _>(action)
            .execute(conn)
            .await
            .map_err(|e| format!("Failed to insert perm {}: {}", p_str, e))?;

            // 3. Get Permission ID
            #[derive(QueryableByName)]
            struct PId { #[diesel(sql_type = diesel::sql_types::Uuid)] id: uuid::Uuid }
            
            let perm_id = diesel::sql_query("SELECT id FROM permissions WHERE permission_string = $1")
                .bind::<diesel::sql_types::Text, _>(p_str)
                .get_result::<PId>(conn)
                .await
                .map_err(|e| format!("Failed to get ID for perm {}: {}", p_str, e))?
                .id;

            // 4. Link Group Permission
            diesel::sql_query(
                r#"
                INSERT INTO group_permissions (id, group_id, permission_id, granted_at)
                VALUES (gen_random_uuid(), $1, $2, NOW())
                ON CONFLICT DO NOTHING
                "#
            )
            .bind::<diesel::sql_types::Uuid, _>(group_id)
            .bind::<diesel::sql_types::Uuid, _>(perm_id)
            .execute(conn)
            .await
            .map_err(|e| format!("Failed to link perm {}: {}", p_str, e))?;
        }
        Ok(())
    }

    let free_seed = seed_permissions_for_group(&mut conn, "free", &["epsx:analytics:view:5", "epsx:rankings:view:5"], "epsx").await;
    let starter_seed = seed_permissions_for_group(&mut conn, "starter", &["epsx:analytics:view:25", "epsx:rankings:view:25", "epsx:analytics:export", "epsx:alerts:create", "epsx:rankings:offset:1"], "epsx").await;
    let pro_seed = seed_permissions_for_group(&mut conn, "pro", &["epsx:analytics:view:100", "epsx:rankings:view:100", "epsx:analytics:export", "epsx:analytics:advanced", "epsx:alerts:create", "epsx:alerts:manage", "epsx:portfolio:view", "epsx:portfolio:manage", "epsx:rankings:offset:1"], "epsx").await;
    let enterprise_seed = seed_permissions_for_group(&mut conn, "enterprise", &["epsx:*:*", "epsx:api:access", "epsx:enterprise:*"], "epsx").await;
    let api_developer_seed = seed_permissions_for_group(&mut conn, "api-developer", &["epsx:api:access", "epsx:analytics:view:unlimited", "epsx:rankings:view:unlimited"], "epsx").await;
    
    // Log seeding errors if any
    if let Err(e) = free_seed { tracing::error!("Error seeding Free Plan permissions: {}", e); }
    if let Err(e) = starter_seed { tracing::error!("Error seeding Starter Plan permissions: {}", e); }
    if let Err(e) = pro_seed { tracing::error!("Error seeding Pro Plan permissions: {}", e); }
    if let Err(e) = enterprise_seed { tracing::error!("Error seeding Enterprise Plan permissions: {}", e); }
    if let Err(e) = api_developer_seed { tracing::error!("Error seeding API Developer Plan permissions: {}", e); }
    
    // Check results
    let mut inserted = 0;
    let mut errors = Vec::new();

    if free_plan_result.is_ok() { inserted += 1; } else { errors.push("Free Plan".to_string()); }
    if starter_plan_result.is_ok() { inserted += 1; } else { errors.push("Starter Plan".to_string()); }
    if pro_plan_result.is_ok() { inserted += 1; } else { errors.push("Pro Plan".to_string()); }
    if enterprise_plan_result.is_ok() { inserted += 1; } else { errors.push("Enterprise Plan".to_string()); }
    if api_plan_result.is_ok() { inserted += 1; } else { errors.push("API Developer".to_string()); }

    // Get total count
    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let total_plans = diesel::sql_query(
        "SELECT COUNT(*) as count FROM groups WHERE group_type = 'subscription'"
    )
    .get_result::<CountRow>(&mut conn)
    .await
    .map(|r| r.count)
    .unwrap_or(0);

    tracing::info!("✅ Seeded {} subscription plans. Total in database: {}", inserted, total_plans);

    (
        StatusCode::OK,
        Json(ApiResponse::success(SeedPlansResponse {
            plans_inserted: inserted,
            total_plans,
            errors,
        }))
    )
}

