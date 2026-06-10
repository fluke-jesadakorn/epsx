//! System Admin Plans Seeder
//!
//! Seeds constant system admin plans on startup.
//! Uses ON CONFLICT to safely re-run (idempotent).

use diesel_async::RunQueryDsl;
use tracing::{info, error};
use uuid::Uuid;

use crate::prelude::TlsPool;
use crate::core::constants::{
    SUPER_ADMIN_PLAN_ID, SUPER_ADMIN_PLAN_NAME, SUPER_ADMIN_PLAN_SLUG,
    MODERATOR_PLAN_ID, MODERATOR_PLAN_NAME, MODERATOR_PLAN_SLUG,
    SUPPORT_PLAN_ID, SUPPORT_PLAN_NAME, SUPPORT_PLAN_SLUG,
    SUPER_ADMIN_WALLET,
};

struct AdminPlanDef {
    id: &'static str,
    name: &'static str,
    slug: &'static str,
    description: &'static str,
    permissions: &'static [&'static str],
}

const ADMIN_PLANS: &[AdminPlanDef] = &[
    AdminPlanDef {
        id: SUPER_ADMIN_PLAN_ID,
        name: SUPER_ADMIN_PLAN_NAME,
        slug: SUPER_ADMIN_PLAN_SLUG,
        description: "Full administrative access to all platform features",
        permissions: &["*:*:*", "admin:*:*"],
    },
    AdminPlanDef {
        id: MODERATOR_PLAN_ID,
        name: MODERATOR_PLAN_NAME,
        slug: MODERATOR_PLAN_SLUG,
        description: "Manage users, view audit logs and analytics",
        permissions: &[
            "admin:users:manage",
            "admin:users:read",
            "admin:permissions:view",
            "admin:audit:read",
            "admin:analytics:view",
            "admin:notifications:manage",
            "admin:security:read",
            "admin:chat:manage",
        ],
    },
    AdminPlanDef {
        id: SUPPORT_PLAN_ID,
        name: SUPPORT_PLAN_NAME,
        slug: SUPPORT_PLAN_SLUG,
        description: "Read-only access for support operations",
        permissions: &[
            "admin:users:read",
            "admin:audit:read",
            "admin:analytics:view",
            "admin:security:read",
        ],
    },
];

/// Seed system admin plans into the database.
/// Safe to call multiple times (idempotent via ON CONFLICT).
pub async fn seed_system_admin_plans(pool: &TlsPool) {
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get DB connection for admin plan seeding: {}", e);
            return;
        }
    };

    for def in ADMIN_PLANS {
        let plan_id = match Uuid::parse_str(def.id) {
            Ok(id) => id,
            Err(e) => {
                error!("Invalid UUID for admin plan {}: {}", def.name, e);
                continue;
            }
        };

        // Upsert plan - only update description/permissions on conflict, never name/slug
        let result = diesel::sql_query(
            r#"INSERT INTO plans (
                id, name, slug, description, plan_type, plan_metadata,
                is_active, is_promoted, is_public, is_system,
                plan_category, plan_group, tier_level, grace_period_hours,
                rate_limit_per_minute, rate_limit_per_hour, rate_limit_per_day, burst_capacity
            ) VALUES (
                $1, $2, $3, $4, 'admin', '{}',
                true, false, false, true,
                'system', 'custom', 99, 0,
                0, 0, 0, 0
            )
            ON CONFLICT (id) DO UPDATE SET
                description = EXCLUDED.description,
                is_system = true,
                plan_type = 'admin',
                plan_category = 'system',
                updated_at = NOW()"#
        )
        .bind::<diesel::sql_types::Uuid, _>(plan_id)
        .bind::<diesel::sql_types::Text, _>(def.name)
        .bind::<diesel::sql_types::Text, _>(def.slug)
        .bind::<diesel::sql_types::Text, _>(def.description)
        .execute(&mut conn)
        .await;

        if let Err(e) = result {
            error!("Failed to seed admin plan {}: {}", def.name, e);
            continue;
        }

        // Seed permissions: ensure each permission exists, then link
        for perm_str in def.permissions {
            let parts: Vec<&str> = perm_str.splitn(3, ':').collect();
            if parts.len() < 3 {
                continue;
            }

            // Ensure permission record exists
            let _ = diesel::sql_query(
                r#"INSERT INTO permissions (permission_string, platform, resource, action, permission_type)
                VALUES ($1, $2, $3, $4, 'system')
                ON CONFLICT (permission_string) DO NOTHING"#
            )
            .bind::<diesel::sql_types::Text, _>(*perm_str)
            .bind::<diesel::sql_types::Text, _>(parts[0])
            .bind::<diesel::sql_types::Text, _>(parts[1])
            .bind::<diesel::sql_types::Text, _>(parts[2])
            .execute(&mut conn)
            .await;

            // Link permission to plan
            let _ = diesel::sql_query(
                r#"INSERT INTO plan_permissions (plan_id, permission_id)
                SELECT $1, p.id FROM permissions p WHERE p.permission_string = $2
                ON CONFLICT (plan_id, permission_id) DO NOTHING"#
            )
            .bind::<diesel::sql_types::Uuid, _>(plan_id)
            .bind::<diesel::sql_types::Text, _>(*perm_str)
            .execute(&mut conn)
            .await;
        }

        info!("Seeded system admin plan: {} ({})", def.name, def.id);
    }

    info!("System admin plans seeding complete");

    // Seed Super Admin wallet assignment
    seed_super_admin_wallet(pool).await;
}

/// Assign SUPER_ADMIN_WALLET to the Super Admin plan.
/// Ensures wallet_users row exists (FK), then upserts assignment.
async fn seed_super_admin_wallet(pool: &TlsPool) {
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get DB connection for super admin wallet seeding: {}", e);
            return;
        }
    };

    let plan_id = match Uuid::parse_str(SUPER_ADMIN_PLAN_ID) {
        Ok(id) => id,
        Err(e) => {
            error!("Invalid Super Admin plan UUID: {}", e);
            return;
        }
    };

    // Use environment variable if provided, otherwise fallback to constant
    let env_wallet = std::env::var("SUPER_ADMIN_WALLET").unwrap_or_else(|_| SUPER_ADMIN_WALLET.to_string());
    
    // In production, we MUST require the ENV var. If it matches the hardcoded dev wallet and we're in prod, warn deeply.
    let is_prod = std::env::var("NODE_ENV").unwrap_or_default() == "production" || 
                  std::env::var("ENVIRONMENT").unwrap_or_default() == "production";
                  
    if is_prod && env_wallet == SUPER_ADMIN_WALLET {
        tracing::warn!("⚠️ SECURITY WARNING: Using default hardcoded SUPER_ADMIN_WALLET in production! Set SUPER_ADMIN_WALLET env var to override.");
    }

    // Ensure wallet_users entry exists (FK constraint)
    if let Err(e) = diesel::sql_query(
        r#"INSERT INTO wallet_users (wallet_address, is_active, tier_level, wallet_metadata)
        VALUES ($1, true, 'Bronze', '{}')
        ON CONFLICT (wallet_address) DO NOTHING"#
    )
    .bind::<diesel::sql_types::Text, _>(&env_wallet)
    .execute(&mut conn)
    .await
    {
        error!("Failed to ensure wallet_users entry for super admin: {}", e);
        return;
    }

    // Upsert wallet_plan_assignments
    match diesel::sql_query(
        r#"INSERT INTO wallet_plan_assignments (id, wallet_address, plan_id, is_active, assigned_at, assigned_by, assignment_source)
        VALUES (gen_random_uuid(), $1, $2, true, NOW(), 'system:seed', 'system_seed')
        ON CONFLICT (wallet_address, plan_id) DO UPDATE SET
            is_active = true,
            assignment_source = 'system_seed'"#
    )
    .bind::<diesel::sql_types::Text, _>(&env_wallet)
    .bind::<diesel::sql_types::Uuid, _>(plan_id)
    .execute(&mut conn)
    .await
    {
        Ok(_) => info!("Seeded Super Admin wallet assignment: {}", env_wallet),
        Err(e) => error!("Failed to seed Super Admin wallet assignment: {}", e),
    }
}
