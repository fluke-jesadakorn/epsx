// Plan Repository Adapter (Infrastructure Layer)
// PostgreSQL implementation of PlanRepositoryPort using Diesel
// (Previously PermissionPlanRepositoryAdapter - renamed for clarity)
use crate::prelude::*;
use tracing::{error, info};
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};

use crate::domain::permission_management::{
    Plan, PlanId, PlanSlug, PermissionString,
    repository_ports::{PlanRepositoryPort, PlanSearchCriteria, PlanStatistics},
    aggregates::plan::LoadPlanParams,
};
use crate::infrastructure::models::plan::{PlanDb, NewPlanDb};
use crate::infrastructure::adapters::repositories::database_types::PermissionRow;
use crate::schemas::primary::{plans, plan_permissions};
use std::collections::HashSet;

#[derive(diesel::QueryableByName)]
struct IdResult {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
}

#[derive(diesel::QueryableByName)]
struct PlanStatsRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub total_plans: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub active_plans: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub promoted_plans: i64,
}

#[derive(diesel::QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

/// PostgreSQL implementation of PlanRepositoryPort using Diesel
#[derive(Clone)]
pub struct PlanRepositoryAdapter {
    db_pool: &'static TlsPool,
}

impl PlanRepositoryAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl PlanRepositoryPort for PlanRepositoryAdapter {
    async fn find_by_id(&self, id: &PlanId) -> AppResult<Option<Plan>> {
        let mut conn = self.db_pool.conn().await?;

        let plan_result = plans::table
            .filter(plans::id.eq(id.value()))
            .select(PlanDb::as_select())
            .first::<PlanDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find permission plan by id {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;

        if let Some(row) = plan_result {
            // Get permissions for this plan using raw SQL (JOIN query)
            let query = r#"
                SELECT p.platform, p.resource, p.action
                FROM plan_permissions pgm
                JOIN permissions p ON pgm.permission_id = p.id
                WHERE pgm.plan_id = $1
            "#;

            let permission_rows = diesel::sql_query(query)
                .bind::<diesel::sql_types::Uuid, _>(id.value())
                .load::<PermissionRow>(&mut conn)
                .await
                .map_err(|e| {
                    error!("Failed to fetch permissions for plan {}: {}", id, e);
                    AppError::database_error(e.to_string())
                })?;

            let permissions: HashSet<PermissionString> = permission_rows
                .iter()
                .filter_map(|r| {
                    let perm_str = format!("{}:{}:{}", r.platform, r.resource, r.action);
                    PermissionString::new(perm_str).ok()
                })
                .collect();

            let plan_id = PlanId::from_uuid(row.id);
            let slug = PlanSlug::new(row.slug)
                .map_err(|e| AppError::validation_error(e.to_string()))?;

            // Convert BigDecimal to f64 for domain model
            let price_f64 = row.price
                .and_then(|bd| bd.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);

            let plan = Plan::load(LoadPlanParams {
                id: plan_id,
                name: row.name,
                slug,
                description: row.description,
                plan_type: row.plan_type,
                permissions,
                price: price_f64,
                currency: row.currency.unwrap_or_else(|| "USD".to_string()),
                billing_cycle: row.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
                is_active: row.is_active,
                is_promoted: row.is_promoted,
                tier_level: row.tier_level,
                max_members: row.max_members,
                auto_assign_enabled: row.auto_assign_enabled.unwrap_or(false),
                metadata: row.plan_metadata,
                is_public: row.is_public,
                grace_period_hours: row.grace_period_hours,
                created_at: row.created_at,
                updated_at: row.updated_at,
                version: 1,
            });

            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug(&self, slug: &PlanSlug) -> AppResult<Option<Plan>> {
        let mut conn = self.db_pool.conn().await?;

        let id_result = plans::table
            .filter(plans::slug.eq(slug.as_str()))
            .select(plans::id)
            .first::<uuid::Uuid>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find permission plan by slug {}: {}", slug, e);
                AppError::database_error(e.to_string())
            })?;

        if let Some(id_uuid) = id_result {
            let plan_id = PlanId::from_uuid(id_uuid);
            self.find_by_id(&plan_id).await
        } else {
            Ok(None)
        }
    }

    async fn find_all(&self, criteria: PlanSearchCriteria) -> AppResult<Vec<Plan>> {
        let mut conn = self.db_pool.conn().await?;

        // Build dynamic query using Diesel DSL
        let mut query = plans::table.into_boxed();

        if let Some(plan_type) = &criteria.plan_type {
            query = query.filter(plans::plan_type.eq(plan_type));
        }

        if let Some(is_active) = criteria.is_active {
            query = query.filter(plans::is_active.eq(is_active));
        }

        if let Some(is_promoted) = criteria.is_promoted {
            query = query.filter(plans::is_promoted.eq(is_promoted));
        }

        if let Some(search_term) = &criteria.search_term {
            let pattern = format!("%{}%", search_term);
            let p = pattern.clone();
            query = query.filter(
                plans::name.ilike(pattern)
                    .or(plans::description.ilike(p))
            );
        }

        query = query.order((
            plans::tier_level.asc(),
            plans::created_at.desc(),
        ));

        if let Some(limit_val) = criteria.limit {
            query = query.limit(limit_val);
        }

        if let Some(offset_val) = criteria.offset {
            query = query.offset(offset_val);
        }

        let plan_ids = query
            .select(plans::id)
            .load::<uuid::Uuid>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find permission plans: {}", e);
                AppError::database_error(e.to_string())
            })?;

        let mut plans = Vec::new();
        for id_uuid in plan_ids {
            let plan_id = PlanId::from_uuid(id_uuid);
            if let Some(plan) = self.find_by_id(&plan_id).await? {
                plans.push(plan);
            }
        }

        Ok(plans)
    }

    async fn save(&self, plan: &Plan) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        let new_plan = NewPlanDb {
            id: *plan.id().value(),
            name: plan.name().to_string(),
            slug: plan.slug().as_str().to_string(),
            description: plan.description().to_string(),
            plan_type: plan.plan_type().to_string(),
            plan_metadata: plan.metadata().clone(),
            price: plan.price().to_string().parse::<bigdecimal::BigDecimal>().ok(),
            currency: Some(plan.currency().to_string()),
            billing_cycle: Some(plan.billing_cycle().to_string()),
            is_active: plan.is_active(),
            is_promoted: plan.is_promoted(),
            max_members: plan.max_members(),
            auto_assign_enabled: Some(plan.auto_assign_enabled()),
            assignment_rules: None,
            created_at: plan.created_at(),
            updated_at: plan.updated_at(),
            created_by: None,
            last_modified_by: None,
            grace_period_hours: plan.grace_period_hours(),
            rate_limit_per_minute: 0,
            rate_limit_per_hour: 0,
            rate_limit_per_day: 0,
            burst_capacity: 0,
            tier_level: plan.tier_level(),
            is_public: plan.is_public(),
        };

        // Upsert permission plan
        diesel::insert_into(plans::table)
            .values(&new_plan)
            .on_conflict(plans::id)
            .do_update()
            .set((
                plans::name.eq(&new_plan.name),
                plans::description.eq(&new_plan.description),
                plans::price.eq(&new_plan.price),
                plans::currency.eq(&new_plan.currency),
                plans::billing_cycle.eq(&new_plan.billing_cycle),
                plans::is_active.eq(new_plan.is_active),
                plans::is_promoted.eq(new_plan.is_promoted),
                plans::tier_level.eq(new_plan.tier_level),
                plans::max_members.eq(&new_plan.max_members),
                plans::auto_assign_enabled.eq(&new_plan.auto_assign_enabled),
                plans::plan_metadata.eq(&new_plan.plan_metadata),
                plans::updated_at.eq(new_plan.updated_at),
                plans::is_public.eq(new_plan.is_public),
                plans::grace_period_hours.eq(new_plan.grace_period_hours),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to save permission plan: {}", e);
                AppError::database_error(e.to_string())
            })?;

        // Delete existing permission associations
        diesel::delete(plan_permissions::table)
            .filter(plan_permissions::plan_id.eq(plan.id().value()))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Insert permission associations
        // Note: Sequential execution instead of transaction (following wallet_user_repository pattern)
        for permission in plan.permissions() {
            let parts: Vec<&str> = permission.as_str().split(':').collect();
            if parts.len() >= 3 {
                // Get or create permission using raw SQL
                let query = r#"
                    INSERT INTO permissions (permission_string, platform, resource, action, permission_type)
                    VALUES ($1, $2, $3, $4, 'manual')
                    ON CONFLICT (permission_string) DO UPDATE
                    SET platform = EXCLUDED.platform
                    RETURNING id
                "#;

                let perm_id = diesel::sql_query(query)
                    .bind::<diesel::sql_types::Text, _>(permission.as_str())
                    .bind::<diesel::sql_types::Text, _>(parts[0])
                    .bind::<diesel::sql_types::Text, _>(parts[1])
                    .bind::<diesel::sql_types::Text, _>(parts[2])
                    .get_result::<IdResult>(&mut conn)
                    .await
                    .map(|result| result.id)
                    .map_err(|e| AppError::database_error(e.to_string()))?;

                // Link permission to plan
                diesel::sql_query(
                    r#"
                    INSERT INTO plan_permissions (plan_id, permission_id)
                    VALUES ($1, $2)
                    "#
                )
                .bind::<diesel::sql_types::Uuid, _>(plan.id().value())
                .bind::<diesel::sql_types::Uuid, _>(perm_id)
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?;
            }
        }

        info!("Permission plan {} saved successfully", plan.id());
        Ok(())
    }

    async fn delete(&self, id: &PlanId) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        diesel::delete(plans::table)
            .filter(plans::id.eq(id.value()))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to delete permission plan {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;

        info!("Permission plan {} deleted successfully", id);
        Ok(())
    }

    async fn count(&self, criteria: PlanSearchCriteria) -> AppResult<i64> {
        let mut conn = self.db_pool.conn().await?;

        let mut query = plans::table.into_boxed();

        if let Some(plan_type) = &criteria.plan_type {
            query = query.filter(plans::plan_type.eq(plan_type));
        }

        if let Some(is_active) = criteria.is_active {
            query = query.filter(plans::is_active.eq(is_active));
        }

        if let Some(is_promoted) = criteria.is_promoted {
            query = query.filter(plans::is_promoted.eq(is_promoted));
        }

        if let Some(search_term) = &criteria.search_term {
            let pattern = format!("%{}%", search_term);
            let p = pattern.clone();
            query = query.filter(
                plans::name.ilike(pattern)
                    .or(plans::description.ilike(p))
            );
        }

        let count = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count permission plans: {}", e);
                AppError::database_error(e.to_string())
            })?;

        Ok(count)
    }

    async fn get_statistics(&self) -> AppResult<PlanStatistics> {
        let mut conn = self.db_pool.conn().await?;

        // Use diesel::sql_query for FILTER clause compatibility
        let query = r#"
            SELECT
                COUNT(*) as total_plans,
                COUNT(*) FILTER (WHERE is_active = true) as active_plans,
                COUNT(*) FILTER (WHERE is_promoted = true) as promoted_plans
            FROM plans
        "#;

        let row = diesel::sql_query(query)
            .get_result::<PlanStatsRow>(&mut conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to get plan statistics: {}", e))
            })?;

        let total_members: i64 = diesel::sql_query(
            "SELECT COUNT(DISTINCT wallet_address) as count FROM wallet_plan_assignments"
        )
        .get_result::<CountResult>(&mut conn)
        .await
        .map(|result| result.count)
        .unwrap_or(0);

        Ok(PlanStatistics {
            total_plans: row.total_plans,
            active_plans: row.active_plans,
            promoted_plans: row.promoted_plans,
            total_members,
        })
    }

    async fn slug_exists(&self, slug: &PlanSlug) -> AppResult<bool> {
        let mut conn = self.db_pool.conn().await?;

        let exists = diesel::select(diesel::dsl::exists(
            plans::table.filter(plans::slug.eq(slug.as_str()))
        ))
        .get_result::<bool>(&mut conn)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(exists)
    }
}

// Additional helper methods for subscription plan management
impl PlanRepositoryAdapter {
    /// Get all subscription plans (database_types.rs compatibility layer)
    pub async fn get_subscription_plans(&self) -> Result<Vec<crate::infrastructure::adapters::repositories::database_types::PermissionPlan>, diesel::result::Error> {
        use crate::infrastructure::adapters::repositories::database_types::PermissionPlan as DbPermissionPlan;
        use crate::schemas::primary::plans;

        let mut conn = self.db_pool.conn().await
            .map_err(|_e| diesel::result::Error::NotFound)?;

        plans::table
            .filter(plans::plan_type.eq("subscription"))
            .order_by((
                plans::tier_level.asc(),
                plans::price.assume_not_null().asc()
            ))
            .select(DbPermissionPlan::as_select())
            .load::<DbPermissionPlan>(&mut conn)
            .await
    }

    /// Get plan by ID (database_types.rs compatibility layer)
    pub async fn get_plan_by_id(&self, plan_id: uuid::Uuid) -> Result<Option<crate::infrastructure::adapters::repositories::database_types::PermissionPlan>, diesel::result::Error> {
        use crate::infrastructure::adapters::repositories::database_types::PermissionPlan as DbPermissionPlan;
        use crate::schemas::primary::plans;

        let mut conn = self.db_pool.conn().await
            .map_err(|_e| diesel::result::Error::NotFound)?;

        plans::table
            .filter(plans::id.eq(plan_id))
            .filter(plans::plan_type.eq("subscription"))
            .select(DbPermissionPlan::as_select())
            .first::<DbPermissionPlan>(&mut conn)
            .await
            .optional()
    }

    /// Update plan (database_types.rs compatibility layer)
    pub async fn update_plan(&self, plan: crate::infrastructure::adapters::repositories::database_types::PermissionPlan) -> Result<crate::infrastructure::adapters::repositories::database_types::PermissionPlan, diesel::result::Error> {
        use crate::infrastructure::adapters::repositories::database_types::PermissionPlan as DbPermissionPlan;
        use crate::schemas::primary::plans;

        let mut conn = self.db_pool.conn().await
            .map_err(|_e| diesel::result::Error::NotFound)?;

        diesel::update(plans::table.filter(plans::id.eq(plan.id)))
            .set((
                plans::name.eq(plan.name),
                plans::slug.eq(plan.slug),
                plans::description.eq(plan.description),
                plans::plan_metadata.eq(plan.plan_metadata),
                plans::price.eq(plan.price),
                plans::currency.eq(plan.currency),
                plans::billing_cycle.eq(plan.billing_cycle),
                plans::is_active.eq(plan.is_active.unwrap_or(true)),
                plans::is_promoted.eq(plan.is_promoted.unwrap_or(false)),
                plans::tier_level.eq(plan.tier_level),
                plans::updated_at.eq(diesel::dsl::now),
            ))
            .returning(DbPermissionPlan::as_returning())
            .get_result::<DbPermissionPlan>(&mut conn)
            .await
    }

    /// Create a new permission plan (database_types.rs compatibility layer)
    pub async fn create_plan(&self, new_plan: crate::infrastructure::adapters::repositories::database_types::NewPermissionPlan) -> Result<crate::infrastructure::adapters::repositories::database_types::PermissionPlan, diesel::result::Error> {
        use crate::schemas::primary::plans;
        use crate::infrastructure::adapters::repositories::database_types::PermissionPlan as DbPermissionPlan;

        let mut conn = self.db_pool.conn().await
            .map_err(|_e| diesel::result::Error::NotFound)?;

        diesel::insert_into(plans::table)
            .values(&new_plan)
            .returning(DbPermissionPlan::as_returning())
            .get_result::<DbPermissionPlan>(&mut conn)
            .await
    }
}

// Type alias for backward compatibility
pub type PermissionPlanRepositoryAdapter = PlanRepositoryAdapter;
