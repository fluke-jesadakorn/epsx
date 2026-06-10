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
use diesel_async::RunQueryDsl;
use crate::core::constants::*;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct SeedPlansResponse {
    pub plans_inserted: i32,
    pub total_plans: i64,
    pub errors: Vec<String>,
}

/// POST /api/admin/plans/seed
///
/// Requires admin auth. Disabled entirely in production as safety measure.
#[utoipa::path(
    post,
    path = "/api/admin/plans/seed",
    tag = "admin",
    responses(
        (status = 200, description = "Successfully seeded subscription plans", body = ApiResponse<SeedPlansResponse>),
        (status = 403, description = "Forbidden in production"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn seed_subscription_plans(State(app_state): State<AppState>) -> (StatusCode, Json<ApiResponse<SeedPlansResponse>>) {
    if crate::config::env::is_production() {
        tracing::warn!("Seed endpoint called in production — rejecting");
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("FORBIDDEN", "Plan seeding is disabled in production"))
        );
    }

    tracing::info!("Seeding subscription plans...");

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

    // Free Plan (system plan with constant ID)
    let free_meta = json!({
        "permissions": ["epsx:rankings:view:5", "epsx:rankings:offset:100"],
        "features": ["View top 5 stock rankings", "Basic market overview", "Community access"],
        "ranking_offset": FREE_PLAN_RANKING_OFFSET,
        "rankings_limit": 5,
        "limits": { "analytics_queries_per_day": 5, "stocks_tracked": 5, "historical_data_months": 1 }
    });

    let free_res = diesel::sql_query(
        r#"INSERT INTO plans (
            id, name, slug, description, plan_type, plan_metadata,
            price, currency, billing_cycle, is_active, is_promoted, display_order, created_by, tier_level, is_public
        ) VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name, slug = EXCLUDED.slug, description = EXCLUDED.description,
            plan_type = EXCLUDED.plan_type, plan_metadata = EXCLUDED.plan_metadata,
            price = EXCLUDED.price, currency = EXCLUDED.currency, billing_cycle = EXCLUDED.billing_cycle,
            is_active = EXCLUDED.is_active, is_promoted = EXCLUDED.is_promoted,
            display_order = EXCLUDED.display_order, created_by = EXCLUDED.created_by,
            tier_level = EXCLUDED.tier_level, is_public = EXCLUDED.is_public"#
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::parse_str(FREE_PLAN_ID).unwrap())
    .bind::<diesel::sql_types::Text, _>(FREE_PLAN_NAME)
    .bind::<diesel::sql_types::Text, _>(FREE_PLAN_SLUG)
    .bind::<diesel::sql_types::Text, _>(FREE_PLAN_DESCRIPTION)
    .bind::<diesel::sql_types::Text, _>("subscription")
    .bind::<diesel::sql_types::Jsonb, _>(&free_meta)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Numeric>, _>(BigDecimal::from_str("0").ok())
    .bind::<diesel::sql_types::Text, _>("USD")
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
    .bind::<diesel::sql_types::Bool, _>(true)
    .bind::<diesel::sql_types::Bool, _>(true)
    .bind::<diesel::sql_types::Integer, _>(FREE_PLAN_TIER_LEVEL)
    .bind::<diesel::sql_types::Text, _>("0x0000000000000000000000000000000000000000")
    .bind::<diesel::sql_types::Integer, _>(0)
    .bind::<diesel::sql_types::Bool, _>(true)
    .execute(&mut conn)
    .await;

    // Helper: insert/upsert a plan by slug
    async fn upsert_plan(
        conn: &mut diesel_async::AsyncPgConnection,
        name: &str, slug: &str, desc: &str, plan_type: &str,
        meta: &serde_json::Value, price: &str, billing: &str,
        promoted: bool, order: i32, tier: i32,
        rpm: i32, rph: i32, rpd: i32, burst: i32,
    ) -> Result<usize, diesel::result::Error> {
        diesel::sql_query(
            r#"INSERT INTO plans (
                name, slug, description, plan_type, plan_metadata,
                price, currency, billing_cycle, is_active, is_promoted, is_public,
                display_order, tier_level, rate_limit_per_minute, rate_limit_per_hour,
                rate_limit_per_day, burst_capacity, created_by
            ) VALUES ($1, $2, $3, $4, $5::jsonb, $6, 'USD', $7, true, $8, true, $9, $10, $11, $12, $13, $14, '0x0000000000000000000000000000000000000000')
            ON CONFLICT (slug) DO UPDATE SET
                name = EXCLUDED.name, description = EXCLUDED.description,
                plan_type = EXCLUDED.plan_type, plan_metadata = EXCLUDED.plan_metadata,
                price = EXCLUDED.price, billing_cycle = EXCLUDED.billing_cycle,
                is_active = EXCLUDED.is_active, is_promoted = EXCLUDED.is_promoted,
                display_order = EXCLUDED.display_order, tier_level = EXCLUDED.tier_level,
                rate_limit_per_minute = EXCLUDED.rate_limit_per_minute,
                rate_limit_per_hour = EXCLUDED.rate_limit_per_hour,
                rate_limit_per_day = EXCLUDED.rate_limit_per_day,
                burst_capacity = EXCLUDED.burst_capacity, updated_at = NOW()"#
        )
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(slug)
        .bind::<diesel::sql_types::Text, _>(desc)
        .bind::<diesel::sql_types::Text, _>(plan_type)
        .bind::<diesel::sql_types::Jsonb, _>(meta)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Numeric>, _>(BigDecimal::from_str(price).ok())
        .bind::<diesel::sql_types::Text, _>(billing)
        .bind::<diesel::sql_types::Bool, _>(promoted)
        .bind::<diesel::sql_types::Integer, _>(order)
        .bind::<diesel::sql_types::Integer, _>(tier)
        .bind::<diesel::sql_types::Integer, _>(rpm)
        .bind::<diesel::sql_types::Integer, _>(rph)
        .bind::<diesel::sql_types::Integer, _>(rpd)
        .bind::<diesel::sql_types::Integer, _>(burst)
        .execute(conn)
        .await
    }

    // 1. One Day Plan
    let one_day_meta = json!({
        "features": ["Basic analytics view", "Rankings from position 6+", "Basic trading features", "24-hour trial access", "Explore the platform"],
        "ranking_offset": 5, "rankings_limit": 5,
        "promotion": { "enabled": true, "type": "percentage", "value": 80.0, "price": 1.0, "start_date": "", "end_date": "2026-03-25T14:00:00Z" }
    });
    let one_day_res = upsert_plan(&mut conn, "One Day Plan", "one-day", "24-hour trial access to explore the platform", "subscription", &one_day_meta, "5.00", "one_time", false, 1, 0, 60, 1000, 10000, 10).await;

    // 2. Starter Plan
    let starter_meta = json!({
        "features": ["Advanced analytics view", "25 stock rankings", "Basic Analytic features", "Price alerts", "Email support", "30-day access"],
        "ranking_offset": 1, "rankings_limit": 25,
        "promotion": { "enabled": true, "type": "percentage", "value": 90.0, "price": 9.9, "start_date": "", "end_date": "2026-03-25T14:00:00Z" }
    });
    let starter_res = upsert_plan(&mut conn, "Starter Plan", "starter", "Advanced analytics for individual investors and traders", "subscription", &starter_meta, "99.00", "one_time", false, 2, 1, 120, 3000, 50000, 20).await;

    // 3. Life Time
    let lifetime_meta = json!({
        "features": ["Advanced analytics suite", "Full rankings access (Rank 1+)", "API read access", "Basic & Pro trading", "Priority support", "Lifetime access"],
        "ranking_offset": 0, "rankings_limit": -1,
        "promotion": { "enabled": true, "type": "percentage", "value": 50.0, "price": 4999.0, "start_date": "", "end_date": "2026-03-25T14:00:00Z" }
    });
    let lifetime_res = upsert_plan(&mut conn, "Life Time", "lifetime", "Full platform access with lifetime membership", "subscription", &lifetime_meta, "9999.00", "lifetime", true, 3, 3, 300, 10000, 200000, 50).await;

    // 4. Company Plan
    let company_meta = json!({
        "features": ["Advanced analytics suite", "Full trading suite (Basic, Pro & Advanced)", "API read & write access", "Data export", "Notifications management", "365-day corporate access", "Dedicated support"],
        "ranking_offset": 0, "rankings_limit": -1,
        "promotion": { "enabled": true, "type": "percentage", "value": 57.0, "price": 2999.0, "start_date": "", "end_date": "2026-04-04T05:00:00Z" }
    });
    let company_res = upsert_plan(&mut conn, "Company Plan", "company", "Complete solutions for professional teams and institutions", "subscription", &company_meta, "6999.00", "one_time", false, 4, 4, 1000, 50000, 1000000, 200).await;

    // 5. API Personal
    let api_meta = json!({
        "features": ["Analytics view access", "API read access", "Data export capability", "Full developer documentation", "30-day access"],
        "ranking_offset": 1, "rankings_limit": -1,
        "promotion": { "enabled": true, "type": "percentage", "value": 75.0, "price": 999.0, "start_date": "", "end_date": "2026-03-25T14:00:00Z" }
    });
    let api_res = upsert_plan(&mut conn, "API Personal", "api-personal", "Integrate our powerful API into your systems", "subscription", &api_meta, "3999.00", "one_time", false, 5, 2, 300, 10000, 100000, 50).await;

    // 6. Custom
    let custom_meta = json!({
        "features": ["Custom feature set & permissions", "Dedicated support & SLA", "Volume-based pricing", "Custom API rate limits", "White-label options", "Priority onboarding"],
        "contact_sales": true
    });
    let custom_res = upsert_plan(&mut conn, "Custom", "custom", "Tailored solutions for partners, corporate, and enterprise needs", "manual", &custom_meta, "0.00", "pay_per_use", false, 6, 5, 1000, 50000, 1000000, 200).await;

    // Seed plan permissions
    async fn seed_perms(
        conn: &mut diesel_async::AsyncPgConnection,
        slug: &str,
        perms: &[&str],
    ) -> Result<(), String> {
        #[derive(QueryableByName)]
        struct GId { #[diesel(sql_type = diesel::sql_types::Uuid)] id: uuid::Uuid }

        let plan_id = diesel::sql_query("SELECT id FROM plans WHERE slug = $1")
            .bind::<diesel::sql_types::Text, _>(slug)
            .get_result::<GId>(conn)
            .await
            .map_err(|e| format!("Plan {} not found: {}", slug, e))?
            .id;

        for p_str in perms {
            let parts: Vec<&str> = p_str.split(':').collect();
            #[allow(clippy::get_first)]
            let platform = parts.get(0).unwrap_or(&"epsx");
            let resource = parts.get(1).unwrap_or(&"unknown");
            let action = parts.get(2).unwrap_or(&"access");

            diesel::sql_query(
                r#"INSERT INTO permissions (id, permission_string, platform, resource, action, permission_type)
                VALUES (gen_random_uuid(), $1, $2, $3, $4, 'manual')
                ON CONFLICT (permission_string) DO NOTHING"#
            )
            .bind::<diesel::sql_types::Text, _>(p_str)
            .bind::<diesel::sql_types::Text, _>(platform)
            .bind::<diesel::sql_types::Text, _>(resource)
            .bind::<diesel::sql_types::Text, _>(action)
            .execute(conn)
            .await
            .map_err(|e| format!("Failed to insert perm {}: {}", p_str, e))?;

            #[derive(QueryableByName)]
            struct PId { #[diesel(sql_type = diesel::sql_types::Uuid)] id: uuid::Uuid }

            let perm_id = diesel::sql_query("SELECT id FROM permissions WHERE permission_string = $1")
                .bind::<diesel::sql_types::Text, _>(p_str)
                .get_result::<PId>(conn)
                .await
                .map_err(|e| format!("Failed to get ID for perm {}: {}", p_str, e))?
                .id;

            diesel::sql_query(
                r#"INSERT INTO plan_permissions (id, plan_id, permission_id, granted_at)
                VALUES (gen_random_uuid(), $1, $2, NOW())
                ON CONFLICT DO NOTHING"#
            )
            .bind::<diesel::sql_types::Uuid, _>(plan_id)
            .bind::<diesel::sql_types::Uuid, _>(perm_id)
            .execute(conn)
            .await
            .map_err(|e| format!("Failed to link perm {}: {}", p_str, e))?;
        }
        Ok(())
    }

    // Seed permissions per plan (matching CLAUDE.md)
    let free_seed = seed_perms(&mut conn, "free", &["epsx:rankings:view:5", "epsx:rankings:offset:100"]).await;
    let one_day_seed = seed_perms(&mut conn, "one-day", &["epsx:analytics:view", "epsx:trading:basic"]).await;
    let starter_seed = seed_perms(&mut conn, "starter", &["epsx:analytics:view", "epsx:analytics:advanced", "epsx:trading:basic", "epsx:alerts:create"]).await;
    let lifetime_seed = seed_perms(&mut conn, "lifetime", &["epsx:analytics:view", "epsx:analytics:advanced", "epsx:trading:basic", "epsx:trading:pro", "epsx:api:read"]).await;
    let company_seed = seed_perms(&mut conn, "company", &["epsx:analytics:view", "epsx:analytics:advanced", "epsx:trading:basic", "epsx:trading:pro", "epsx:trading:advanced", "epsx:api:read", "epsx:api:write", "epsx:data:export", "epsx:notifications:manage"]).await;
    let api_seed = seed_perms(&mut conn, "api-personal", &["epsx:analytics:view", "epsx:api:read", "epsx:data:export"]).await;

    if let Err(e) = free_seed { tracing::error!("Error seeding Free Plan perms: {}", e); }
    if let Err(e) = one_day_seed { tracing::error!("Error seeding One Day Plan perms: {}", e); }
    if let Err(e) = starter_seed { tracing::error!("Error seeding Starter Plan perms: {}", e); }
    if let Err(e) = lifetime_seed { tracing::error!("Error seeding Lifetime Plan perms: {}", e); }
    if let Err(e) = company_seed { tracing::error!("Error seeding Company Plan perms: {}", e); }
    if let Err(e) = api_seed { tracing::error!("Error seeding API Personal Plan perms: {}", e); }

    // Deactivate old plan slugs that no longer exist
    let _ = diesel::sql_query(
        "UPDATE plans SET is_active = false WHERE slug IN ('pro', 'enterprise', 'api-developer') AND is_active = true"
    ).execute(&mut conn).await;

    // Count results
    let mut inserted = 0;
    let mut errors = Vec::new();

    if free_res.is_ok() { inserted += 1; } else { errors.push("Free Plan".into()); }
    if one_day_res.is_ok() { inserted += 1; } else { errors.push("One Day Plan".into()); }
    if starter_res.is_ok() { inserted += 1; } else { errors.push("Starter Plan".into()); }
    if lifetime_res.is_ok() { inserted += 1; } else { errors.push("Life Time".into()); }
    if company_res.is_ok() { inserted += 1; } else { errors.push("Company Plan".into()); }
    if api_res.is_ok() { inserted += 1; } else { errors.push("API Personal".into()); }
    if custom_res.is_ok() { inserted += 1; } else { errors.push("Custom".into()); }

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let total_plans = diesel::sql_query(
        "SELECT COUNT(*) as count FROM plans WHERE is_active = true"
    )
    .get_result::<CountRow>(&mut conn)
    .await
    .map(|r| r.count)
    .unwrap_or(0);

    tracing::info!("Seeded {} plans. Total active: {}", inserted, total_plans);

    (
        StatusCode::OK,
        Json(ApiResponse::success(SeedPlansResponse {
            plans_inserted: inserted,
            total_plans,
            errors,
        }))
    )
}
