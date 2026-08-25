use crate::web::api_response::ApiResponse;
use crate::web::auth::AppState;
use axum::{extract::State, http::StatusCode, Json};
use epsx_contracts::constants::*;
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

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
pub async fn seed_subscription_plans(
    State(app_state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<SeedPlansResponse>>) {
    if crate::config::env::is_production() {
        tracing::warn!("Seed endpoint called in production — rejecting");
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "FORBIDDEN",
                "Plan seeding is disabled in production",
            )),
        );
    }

    tracing::info!("Seeding subscription plans...");

    // Free Plan (system plan with constant ID)
    let free_meta = json!({
        "permissions": ["epsx:rankings:view:5", "epsx:rankings:offset:100"],
        "features": ["View top 5 stock rankings", "Basic market overview", "Community access"],
        "ranking_offset": FREE_PLAN_RANKING_OFFSET,
        "rankings_limit": 5,
        "limits": { "analytics_queries_per_day": 5, "stocks_tracked": 5, "historical_data_months": 1 }
    });

    let free_res = upsert_plan_by_id(
        &app_state.db_pool,
        FREE_PLAN_ID,
        FREE_PLAN_NAME,
        FREE_PLAN_SLUG,
        FREE_PLAN_DESCRIPTION,
        "subscription",
        &free_meta,
        None,
        "USD",
        None,
        true,
        true,
        FREE_PLAN_TIER_LEVEL,
        "0x0000000000000000000000000000000000000000",
        0,
        true,
    )
    .await;

    // Helper: insert/upsert a plan by slug (async, sqlx)
    async fn upsert_plan_by_id(
        db_pool: &sqlx::PgPool,
        id: &str,
        name: &str,
        slug: &str,
        desc: &str,
        plan_type: &str,
        meta: &serde_json::Value,
        price: Option<&str>,
        currency: &str,
        billing: Option<&str>,
        is_active: bool,
        is_promoted: bool,
        tier: i32,
        _created_by: &str,
        display_order: i32,
        is_public: bool,
    ) -> Result<(), String> {
        let id_uuid = uuid::Uuid::parse_str(id).map_err(|e| format!("bad uuid: {}", e))?;
        sqlx::query(
            r#"INSERT INTO plans (
                id, name, slug, description, plan_type, plan_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order, created_by, tier_level, is_public
            ) VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7::numeric, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name, slug = EXCLUDED.slug, description = EXCLUDED.description,
                plan_type = EXCLUDED.plan_type, plan_metadata = EXCLUDED.plan_metadata,
                price = EXCLUDED.price, currency = EXCLUDED.currency, billing_cycle = EXCLUDED.billing_cycle,
                is_active = EXCLUDED.is_active, is_promoted = EXCLUDED.is_promoted,
                display_order = EXCLUDED.display_order, created_by = EXCLUDED.created_by,
                tier_level = EXCLUDED.tier_level, is_public = EXCLUDED.is_public"#,
        )
        .bind(id_uuid)
        .bind(name)
        .bind(slug)
        .bind(desc)
        .bind(plan_type)
        .bind(meta)
        .bind(price.unwrap_or("0"))
        .bind(currency)
        .bind(billing)
        .bind(is_active)
        .bind(is_promoted)
        .bind(display_order)
        .bind(_created_by)
        .bind(tier)
        .bind(is_public)
        .execute(db_pool)
        .await
        .map_err(|e| format!("upsert_plan {}: {}", slug, e))?;
        Ok(())
    }

    async fn upsert_plan_by_slug(
        db_pool: &sqlx::PgPool,
        name: &str,
        slug: &str,
        desc: &str,
        plan_type: &str,
        meta: &serde_json::Value,
        price: &str,
        billing: &str,
        promoted: bool,
        order: i32,
        tier: i32,
        rpm: i32,
        rph: i32,
        rpd: i32,
        burst: i32,
    ) -> Result<(), String> {
        sqlx::query(
            r#"INSERT INTO plans (
                name, slug, description, plan_type, plan_metadata,
                price, currency, billing_cycle, is_active, is_promoted, is_public,
                display_order, tier_level, rate_limit_per_minute, rate_limit_per_hour,
                rate_limit_per_day, burst_capacity, created_by
            ) VALUES ($1, $2, $3, $4, $5::jsonb, $6::numeric, 'USD', $7, true, $8, true, $9, $10, $11, $12, $13, $14, '0x0000000000000000000000000000000000000000')
            ON CONFLICT (slug) DO UPDATE SET
                name = EXCLUDED.name, description = EXCLUDED.description,
                plan_type = EXCLUDED.plan_type, plan_metadata = EXCLUDED.plan_metadata,
                price = EXCLUDED.price, billing_cycle = EXCLUDED.billing_cycle,
                is_active = EXCLUDED.is_active, is_promoted = EXCLUDED.is_promoted,
                display_order = EXCLUDED.display_order, tier_level = EXCLUDED.tier_level,
                rate_limit_per_minute = EXCLUDED.rate_limit_per_minute,
                rate_limit_per_hour = EXCLUDED.rate_limit_per_hour,
                rate_limit_per_day = EXCLUDED.rate_limit_per_day,
                burst_capacity = EXCLUDED.burst_capacity, updated_at = NOW()"#,
        )
        .bind(name)
        .bind(slug)
        .bind(desc)
        .bind(plan_type)
        .bind(meta)
        .bind(price)
        .bind(billing)
        .bind(promoted)
        .bind(order)
        .bind(tier)
        .bind(rpm)
        .bind(rph)
        .bind(rpd)
        .bind(burst)
        .execute(db_pool)
        .await
        .map_err(|e| format!("upsert_plan_by_slug {}: {}", slug, e))?;
        Ok(())
    }

    async fn seed_perms(db_pool: &sqlx::PgPool, slug: &str, perms: &[&str]) -> Result<(), String> {
        #[derive(sqlx::FromRow)]
        struct GId {
            id: uuid::Uuid,
        }

        let plan_id: GId = sqlx::query_as("SELECT id FROM plans WHERE slug = $1")
            .bind(slug)
            .fetch_one(db_pool)
            .await
            .map_err(|e| format!("Plan {} not found: {}", slug, e))?;

        for p_str in perms {
            let parts: Vec<&str> = p_str.split(':').collect();
            let platform = parts.get(0).unwrap_or(&"epsx");
            let resource = parts.get(1).unwrap_or(&"unknown");
            let action = parts.get(2).unwrap_or(&"access");

            sqlx::query(
                r#"INSERT INTO permissions (id, permission_string, platform, resource, action, permission_type)
                VALUES (gen_random_uuid(), $1, $2, $3, $4, 'manual')
                ON CONFLICT (permission_string) DO NOTHING"#,
            )
            .bind(p_str)
            .bind(platform)
            .bind(resource)
            .bind(action)
            .execute(db_pool)
            .await
            .map_err(|e| format!("Failed to insert perm {}: {}", p_str, e))?;

            #[derive(sqlx::FromRow)]
            struct PId {
                id: uuid::Uuid,
            }

            let perm_id: PId = sqlx::query_as("SELECT id FROM permissions WHERE permission_string = $1")
                .bind(p_str)
                .fetch_one(db_pool)
                .await
                .map_err(|e| format!("Failed to get ID for perm {}: {}", p_str, e))?;

            sqlx::query(
                r#"INSERT INTO plan_permissions (id, plan_id, permission_id, granted_at)
                VALUES (gen_random_uuid(), $1, $2, NOW())
                ON CONFLICT DO NOTHING"#,
            )
            .bind(plan_id.id)
            .bind(perm_id.id)
            .execute(db_pool)
            .await
            .map_err(|e| format!("Failed to link perm {}: {}", p_str, e))?;
        }
        Ok(())
    }

    // 1. One Day Plan
    let one_day_meta = json!({
        "features": ["Basic analytics view", "Rankings from position 6+", "Basic trading features", "24-hour trial access", "Explore the platform"],
        "ranking_offset": 5, "rankings_limit": 5,
        "promotion": { "enabled": true, "type": "percentage", "value": 80.0, "price": 1.0, "start_date": "", "end_date": "2026-03-25T14:00:00Z" }
    });
    let one_day_res = upsert_plan_by_slug(
        &app_state.db_pool,
        "One Day Plan",
        "one-day",
        "24-hour trial access to explore the platform",
        "subscription",
        &one_day_meta,
        "5.00",
        "one_time",
        false,
        1,
        0,
        60,
        1000,
        10000,
        10,
    )
    .await;

    // 2. Starter Plan
    let starter_meta = json!({
        "features": ["Advanced analytics view", "25 stock rankings", "Basic Analytic features", "Price alerts", "Email support", "30-day access"],
        "ranking_offset": 1, "rankings_limit": 25,
        "promotion": { "enabled": true, "type": "percentage", "value": 90.0, "price": 9.9, "start_date": "", "end_date": "2026-03-25T14:00:00Z" }
    });
    let starter_res = upsert_plan_by_slug(
        &app_state.db_pool,
        "Starter Plan",
        "starter",
        "Advanced analytics for individual investors and traders",
        "subscription",
        &starter_meta,
        "99.00",
        "one_time",
        false,
        2,
        1,
        120,
        3000,
        50000,
        20,
    )
    .await;

    // 3. Life Time
    let lifetime_meta = json!({
        "features": ["Advanced analytics suite", "Full rankings access (Rank 1+)", "API read access", "Basic & Pro trading", "Priority support", "Lifetime access"],
        "ranking_offset": 0, "rankings_limit": -1,
        "promotion": { "enabled": true, "type": "percentage", "value": 50.0, "price": 4999.0, "start_date": "", "end_date": "2026-03-25T14:00:00Z" }
    });
    let lifetime_res = upsert_plan_by_slug(
        &app_state.db_pool,
        "Life Time",
        "lifetime",
        "Full platform access with lifetime membership",
        "subscription",
        &lifetime_meta,
        "9999.00",
        "lifetime",
        true,
        3,
        3,
        300,
        10000,
        200000,
        50,
    )
    .await;

    // 4. Company Plan
    let company_meta = json!({
        "features": ["Advanced analytics suite", "Full trading suite (Basic, Pro & Advanced)", "API read & write access", "Data export", "Notifications management", "365-day corporate access", "Dedicated support"],
        "ranking_offset": 0, "rankings_limit": -1,
        "promotion": { "enabled": true, "type": "percentage", "value": 57.0, "price": 2999.0, "start_date": "", "end_date": "2026-04-04T05:00:00Z" }
    });
    let company_res = upsert_plan_by_slug(
        &app_state.db_pool,
        "Company Plan",
        "company",
        "Complete solutions for professional teams and institutions",
        "subscription",
        &company_meta,
        "6999.00",
        "one_time",
        false,
        4,
        4,
        1000,
        50000,
        1000000,
        200,
    )
    .await;

    // 5. API Personal
    let api_meta = json!({
        "features": ["Analytics view access", "API read access", "Data export capability", "Full developer documentation", "30-day access"],
        "ranking_offset": 1, "rankings_limit": -1,
        "promotion": { "enabled": true, "type": "percentage", "value": 75.0, "price": 999.0, "start_date": "", "end_date": "2026-03-25T14:00:00Z" }
    });
    let api_res = upsert_plan_by_slug(
        &app_state.db_pool,
        "API Personal",
        "api-personal",
        "Integrate our powerful API into your systems",
        "subscription",
        &api_meta,
        "3999.00",
        "one_time",
        false,
        5,
        2,
        300,
        10000,
        100000,
        50,
    )
    .await;

    // 6. Custom
    let custom_meta = json!({
        "features": ["Custom feature set & permissions", "Dedicated support & SLA", "Volume-based pricing", "Custom API rate limits", "White-label options", "Priority onboarding"],
        "contact_sales": true
    });
    let custom_res = upsert_plan_by_slug(
        &app_state.db_pool,
        "Custom",
        "custom",
        "Tailored solutions for partners, corporate, and enterprise needs",
        "manual",
        &custom_meta,
        "0.00",
        "pay_per_use",
        false,
        6,
        5,
        1000,
        50000,
        1000000,
        200,
    )
    .await;

    // Seed permissions per plan
    let free_seed = seed_perms(
        &app_state.db_pool,
        "free",
        &["epsx:rankings:view:5", "epsx:rankings:offset:100"],
    )
    .await;
    let one_day_seed = seed_perms(
        &app_state.db_pool,
        "one-day",
        &["epsx:analytics:view", "epsx:trading:basic"],
    )
    .await;
    let starter_seed = seed_perms(
        &app_state.db_pool,
        "starter",
        &[
            "epsx:analytics:view",
            "epsx:analytics:advanced",
            "epsx:trading:basic",
            "epsx:alerts:create",
        ],
    )
    .await;
    let lifetime_seed = seed_perms(
        &app_state.db_pool,
        "lifetime",
        &[
            "epsx:analytics:view",
            "epsx:analytics:advanced",
            "epsx:trading:basic",
            "epsx:trading:pro",
            "epsx:api:read",
        ],
    )
    .await;
    let company_seed = seed_perms(
        &app_state.db_pool,
        "company",
        &[
            "epsx:analytics:view",
            "epsx:analytics:advanced",
            "epsx:trading:basic",
            "epsx:trading:pro",
            "epsx:trading:advanced",
            "epsx:api:read",
            "epsx:api:write",
            "epsx:data:export",
            "epsx:notifications:manage",
        ],
    )
    .await;
    let api_seed = seed_perms(
        &app_state.db_pool,
        "api-personal",
        &["epsx:analytics:view", "epsx:api:read", "epsx:data:export"],
    )
    .await;

    if let Err(e) = free_seed {
        tracing::error!("Error seeding Free Plan perms: {}", e);
    }
    if let Err(e) = one_day_seed {
        tracing::error!("Error seeding One Day Plan perms: {}", e);
    }
    if let Err(e) = starter_seed {
        tracing::error!("Error seeding Starter Plan perms: {}", e);
    }
    if let Err(e) = lifetime_seed {
        tracing::error!("Error seeding Lifetime Plan perms: {}", e);
    }
    if let Err(e) = company_seed {
        tracing::error!("Error seeding Company Plan perms: {}", e);
    }
    if let Err(e) = api_seed {
        tracing::error!("Error seeding API Personal Plan perms: {}", e);
    }

    // Deactivate old plan slugs that no longer exist
    let _ = sqlx::query(
        "UPDATE plans SET is_active = false WHERE slug IN ('pro', 'enterprise', 'api-developer') AND is_active = true",
    )
    .execute(app_state.db_pool.as_ref())
    .await;

    let mut inserted = 0;
    let mut errors = Vec::new();

    if free_res.is_ok() {
        inserted += 1;
    } else {
        errors.push("Free Plan".into());
    }
    if one_day_res.is_ok() {
        inserted += 1;
    } else {
        errors.push("One Day Plan".into());
    }
    if starter_res.is_ok() {
        inserted += 1;
    } else {
        errors.push("Starter Plan".into());
    }
    if lifetime_res.is_ok() {
        inserted += 1;
    } else {
        errors.push("Life Time".into());
    }
    if company_res.is_ok() {
        inserted += 1;
    } else {
        errors.push("Company Plan".into());
    }
    if api_res.is_ok() {
        inserted += 1;
    } else {
        errors.push("API Personal".into());
    }
    if custom_res.is_ok() {
        inserted += 1;
    } else {
        errors.push("Custom".into());
    }

    let total_plans: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM plans WHERE is_active = true")
        .fetch_one(app_state.db_pool.as_ref())
        .await
        .unwrap_or((0,));

    tracing::info!(
        "Seeded {} plans. Total active: {}",
        inserted,
        total_plans.0
    );

    (
        StatusCode::OK,
        Json(ApiResponse::success(SeedPlansResponse {
            plans_inserted: inserted,
            total_plans: total_plans.0,
            errors,
        })),
    )
}
