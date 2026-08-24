// Plan Repository Adapter (Infrastructure Layer)
// PostgreSQL implementation of PlanRepositoryPort using sqlx
//
// BIG-BANG: migrated to sqlx (real). All diesel DSL/derive replaced with raw SQL.

use crate::prelude::*;
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{error, info};

use crate::domain::permission_management::{
    aggregates::plan::LoadPlanParams,
    repository_ports::{PlanRepositoryPort, PlanSearchCriteria, PlanStatistics},
    PermissionString, Plan, PlanCategory, PlanGroup, PlanId, PlanSlug,
};
use crate::infrastructure::models::plan::{NewPlanDb, PlanDb};

#[derive(sqlx::FromRow)]
struct PlanStatsRow {
    total_plans: i64,
    active_plans: i64,
    promoted_plans: i64,
}

#[derive(sqlx::FromRow)]
struct CountResult {
    count: i64,
}

#[derive(sqlx::FromRow)]
struct PermStringRow {
    permission_string: String,
}

/// PostgreSQL implementation of PlanRepositoryPort using sqlx
#[derive(Clone)]
pub struct PlanRepositoryAdapter {
    db_pool: Arc<PgPool>,
}

impl PlanRepositoryAdapter {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl PlanRepositoryPort for PlanRepositoryAdapter {
    async fn find_by_id(&self, id: &PlanId) -> AppResult<Option<Plan>> {
        let plan_result: Option<PlanDb> = sqlx::query_as(
            "SELECT id, name, slug, description, plan_type, plan_metadata, \
                    price, currency, is_active, is_promoted, display_order, \
                    created_by, tier_level, is_public, rate_limit_per_minute, \
                    rate_limit_per_hour, rate_limit_per_day, burst_capacity, \
                    version, created_at, updated_at, contract_address, \
                    token_address, block_number, confirmations, expires_at \
             FROM plans WHERE id = $1",
        )
        .bind(id.value())
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to find permission plan by id {}: {}", id, e);
            AppError::database_error(e.to_string())
        })?;

        if let Some(row) = plan_result {
            let perm_rows: Vec<PermStringRow> = sqlx::query_as(
                "SELECT p.permission_string \
                 FROM plan_permissions pgm JOIN permissions p ON pgm.permission_id = p.id \
                 WHERE pgm.plan_id = $1",
            )
            .bind(id.value())
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to fetch permissions for plan {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;

            let permissions: HashSet<PermissionString> = perm_rows
                .iter()
                .filter_map(|r| PermissionString::new(r.permission_string.clone()).ok())
                .collect();

            let plan_id = PlanId::from_uuid(row.id);
            let slug =
                PlanSlug::new(row.slug).map_err(|e| AppError::validation_error(e.to_string()))?;

            let price_f64 = row
                .price
                .as_ref()
                .and_then(|bd| bd.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);

            let plan = Plan::new(
                plan_id,
                row.name,
                slug,
                row.description.unwrap_or_default(),
                row.plan_type,
                row.plan_metadata,
                price_f64,
                row.currency,
                row.is_active,
                row.is_promoted,
                row.display_order,
                row.created_by,
                row.tier_level,
                row.is_public,
                row.rate_limit_per_minute,
                row.rate_limit_per_hour,
                row.rate_limit_per_day,
                row.burst_capacity,
                permissions,
            )
            .map_err(|e| AppError::validation_error(format!("Invalid plan: {}", e)))?;

            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug(&self, slug: &PlanSlug) -> AppResult<Option<Plan>> {
        let plan_result: Option<PlanDb> = sqlx::query_as(
            "SELECT id, name, slug, description, plan_type, plan_metadata, \
                    price, currency, is_active, is_promoted, display_order, \
                    created_by, tier_level, is_public, rate_limit_per_minute, \
                    rate_limit_per_hour, rate_limit_per_day, burst_capacity, \
                    version, created_at, updated_at, contract_address, \
                    token_address, block_number, confirmations, expires_at \
             FROM plans WHERE slug = $1",
        )
        .bind(slug.value())
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let Some(row) = plan_result else { return Ok(None) };
        let perm_rows: Vec<PermStringRow> = sqlx::query_as(
            "SELECT p.permission_string FROM plan_permissions pgm \
             JOIN permissions p ON pgm.permission_id = p.id WHERE pgm.plan_id = $1",
        )
        .bind(row.id)
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;
        let permissions: HashSet<PermissionString> = perm_rows
            .iter()
            .filter_map(|r| PermissionString::new(r.permission_string.clone()).ok())
            .collect();
        let plan_id = PlanId::from_uuid(row.id);
        let price_f64 = row
            .price
            .as_ref()
            .and_then(|bd| bd.to_string().parse::<f64>().ok())
            .unwrap_or(0.0);
        let plan = Plan::new(
            plan_id,
            row.name,
            slug.clone(),
            row.description.unwrap_or_default(),
            row.plan_type,
            row.plan_metadata,
            price_f64,
            row.currency,
            row.is_active,
            row.is_promoted,
            row.display_order,
            row.created_by,
            row.tier_level,
            row.is_public,
            row.rate_limit_per_minute,
            row.rate_limit_per_hour,
            row.rate_limit_per_day,
            row.burst_capacity,
            permissions,
        )
        .map_err(|e| AppError::validation_error(format!("Invalid plan: {}", e)))?;
        Ok(Some(plan))
    }

    async fn find_active(&self) -> AppResult<Vec<Plan>> {
        let rows: Vec<PlanDb> = sqlx::query_as(
            "SELECT id, name, slug, description, plan_type, plan_metadata, \
                    price, currency, is_active, is_promoted, display_order, \
                    created_by, tier_level, is_public, rate_limit_per_minute, \
                    rate_limit_per_hour, rate_limit_per_day, burst_capacity, \
                    version, created_at, updated_at, contract_address, \
                    token_address, block_number, confirmations, expires_at \
             FROM plans WHERE is_active = TRUE ORDER BY tier_level DESC NULLS LAST, name ASC",
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let mut plans = Vec::with_capacity(rows.len());
        for row in rows {
            let slug = PlanSlug::new(row.slug.clone())
                .map_err(|e| AppError::validation_error(e.to_string()))?;
            let perm_rows: Vec<PermStringRow> = sqlx::query_as(
                "SELECT p.permission_string FROM plan_permissions pgm \
                 JOIN permissions p ON pgm.permission_id = p.id WHERE pgm.plan_id = $1",
            )
            .bind(row.id)
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
            let permissions: HashSet<PermissionString> = perm_rows
                .into_iter()
                .filter_map(|r| PermissionString::new(r.permission_string).ok())
                .collect();
            let plan_id = PlanId::from_uuid(row.id);
            let price_f64 = row
                .price
                .as_ref()
                .and_then(|bd| bd.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);
            let plan = Plan::new(
                plan_id,
                row.name,
                slug,
                row.description.unwrap_or_default(),
                row.plan_type,
                row.plan_metadata,
                price_f64,
                row.currency,
                row.is_active,
                row.is_promoted,
                row.display_order,
                row.created_by,
                row.tier_level,
                row.is_public,
                row.rate_limit_per_minute,
                row.rate_limit_per_hour,
                row.rate_limit_per_day,
                row.burst_capacity,
                permissions,
            )
            .map_err(|e| AppError::validation_error(format!("Invalid plan: {}", e)))?;
            plans.push(plan);
        }
        Ok(plans)
    }

    async fn find_promoted(&self) -> AppResult<Vec<Plan>> {
        let rows: Vec<PlanDb> = sqlx::query_as(
            "SELECT id, name, slug, description, plan_type, plan_metadata, \
                    price, currency, is_active, is_promoted, display_order, \
                    created_by, tier_level, is_public, rate_limit_per_minute, \
                    rate_limit_per_hour, rate_limit_per_day, burst_capacity, \
                    version, created_at, updated_at, contract_address, \
                    token_address, block_number, confirmations, expires_at \
             FROM plans WHERE is_promoted = TRUE AND is_active = TRUE \
             ORDER BY display_order ASC",
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let mut plans = Vec::with_capacity(rows.len());
        for row in rows {
            let slug = PlanSlug::new(row.slug.clone())
                .map_err(|e| AppError::validation_error(e.to_string()))?;
            let perm_rows: Vec<PermStringRow> = sqlx::query_as(
                "SELECT p.permission_string FROM plan_permissions pgm \
                 JOIN permissions p ON pgm.permission_id = p.id WHERE pgm.plan_id = $1",
            )
            .bind(row.id)
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
            let permissions: HashSet<PermissionString> = perm_rows
                .into_iter()
                .filter_map(|r| PermissionString::new(r.permission_string).ok())
                .collect();
            let plan_id = PlanId::from_uuid(row.id);
            let price_f64 = row
                .price
                .as_ref()
                .and_then(|bd| bd.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);
            let plan = Plan::new(
                plan_id,
                row.name,
                slug,
                row.description.unwrap_or_default(),
                row.plan_type,
                row.plan_metadata,
                price_f64,
                row.currency,
                row.is_active,
                row.is_promoted,
                row.display_order,
                row.created_by,
                row.tier_level,
                row.is_public,
                row.rate_limit_per_minute,
                row.rate_limit_per_hour,
                row.rate_limit_per_day,
                row.burst_capacity,
                permissions,
            )
            .map_err(|e| AppError::validation_error(format!("Invalid plan: {}", e)))?;
            plans.push(plan);
        }
        Ok(plans)
    }

    async fn find_by_category(&self, category: &PlanCategory) -> AppResult<Vec<Plan>> {
        let rows: Vec<PlanDb> = sqlx::query_as(
            "SELECT id, name, slug, description, plan_type, plan_metadata, \
                    price, currency, is_active, is_promoted, display_order, \
                    created_by, tier_level, is_public, rate_limit_per_minute, \
                    rate_limit_per_hour, rate_limit_per_day, burst_capacity, \
                    version, created_at, updated_at, contract_address, \
                    token_address, block_number, confirmations, expires_at \
             FROM plans WHERE plan_type = $1 AND is_active = TRUE \
             ORDER BY tier_level DESC NULLS LAST, display_order ASC",
        )
        .bind(category.as_str())
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let mut plans = Vec::with_capacity(rows.len());
        for row in rows {
            let slug = PlanSlug::new(row.slug.clone())
                .map_err(|e| AppError::validation_error(e.to_string()))?;
            let perm_rows: Vec<PermStringRow> = sqlx::query_as(
                "SELECT p.permission_string FROM plan_permissions pgm \
                 JOIN permissions p ON pgm.permission_id = p.id WHERE pgm.plan_id = $1",
            )
            .bind(row.id)
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
            let permissions: HashSet<PermissionString> = perm_rows
                .into_iter()
                .filter_map(|r| PermissionString::new(r.permission_string).ok())
                .collect();
            let plan_id = PlanId::from_uuid(row.id);
            let price_f64 = row
                .price
                .as_ref()
                .and_then(|bd| bd.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);
            let plan = Plan::new(
                plan_id,
                row.name,
                slug,
                row.description.unwrap_or_default(),
                row.plan_type,
                row.plan_metadata,
                price_f64,
                row.currency,
                row.is_active,
                row.is_promoted,
                row.display_order,
                row.created_by,
                row.tier_level,
                row.is_public,
                row.rate_limit_per_minute,
                row.rate_limit_per_hour,
                row.rate_limit_per_day,
                row.burst_capacity,
                permissions,
            )
            .map_err(|e| AppError::validation_error(format!("Invalid plan: {}", e)))?;
            plans.push(plan);
        }
        Ok(plans)
    }

    async fn find_by_group(&self, group: &PlanGroup) -> AppResult<Vec<Plan>> {
        self.find_by_category(&PlanCategory::new(group.value().to_string()).unwrap_or(PlanCategory::Subscription))
            .await
    }

    async fn save(&self, plan: &Plan) -> AppResult<Plan> {
        let new_plan = NewPlanDb {
            id: plan.id().value(),
            name: plan.name().to_string(),
            slug: plan.slug().value().to_string(),
            description: Some(plan.description().to_string()),
            plan_type: plan.plan_type().to_string(),
            plan_metadata: plan.metadata().clone(),
            price: plan.price(),
            currency: plan.currency().to_string(),
            is_active: plan.is_active(),
            is_promoted: plan.is_promoted(),
            display_order: plan.display_order(),
            created_by: plan.created_by().to_string(),
            tier_level: plan.tier_level(),
            is_public: plan.is_public(),
            rate_limit_per_minute: plan.rate_limit_per_minute(),
            rate_limit_per_hour: plan.rate_limit_per_hour(),
            rate_limit_per_day: plan.rate_limit_per_day(),
            burst_capacity: plan.burst_capacity(),
            contract_address: None,
            token_address: None,
            block_number: None,
            confirmations: Some(0),
            expires_at: None,
            version: 1,
        };

        sqlx::query(
            "INSERT INTO plans (\
                id, name, slug, description, plan_type, plan_metadata, \
                price, currency, is_active, is_promoted, display_order, \
                created_by, tier_level, is_public, rate_limit_per_minute, \
                rate_limit_per_hour, rate_limit_per_day, burst_capacity, \
                version, created_at, updated_at\
            ) VALUES (\
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, 1, NOW(), NOW()\
            )",
        )
        .bind(new_plan.id)
        .bind(&new_plan.name)
        .bind(&new_plan.slug)
        .bind(&new_plan.description)
        .bind(&new_plan.plan_type)
        .bind(&new_plan.plan_metadata)
        .bind(new_plan.price)
        .bind(&new_plan.currency)
        .bind(new_plan.is_active)
        .bind(new_plan.is_promoted)
        .bind(new_plan.display_order)
        .bind(&new_plan.created_by)
        .bind(new_plan.tier_level)
        .bind(new_plan.is_public)
        .bind(new_plan.rate_limit_per_minute)
        .bind(new_plan.rate_limit_per_hour)
        .bind(new_plan.rate_limit_per_day)
        .bind(new_plan.burst_capacity)
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        info!("Saved plan {}", new_plan.id);
        Ok(plan.clone())
    }

    async fn update(&self, plan: &Plan) -> AppResult<Plan> {
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            "UPDATE plans SET updated_at = NOW(), version = version + 1",
        );
        qb.push(", name = ").push_bind(plan.name().to_string());
        qb.push(", description = ").push_bind(plan.description().to_string());
        qb.push(", plan_metadata = ").push_bind(plan.metadata().clone());
        qb.push(", is_active = ").push_bind(plan.is_active());
        qb.push(", is_promoted = ").push_bind(plan.is_promoted());
        qb.push(", display_order = ").push_bind(plan.display_order());
        qb.push(", tier_level = ").push_bind(plan.tier_level());
        qb.push(", is_public = ").push_bind(plan.is_public());
        qb.push(", rate_limit_per_minute = ").push_bind(plan.rate_limit_per_minute());
        qb.push(", rate_limit_per_hour = ").push_bind(plan.rate_limit_per_hour());
        qb.push(", rate_limit_per_day = ").push_bind(plan.rate_limit_per_day());
        qb.push(", burst_capacity = ").push_bind(plan.burst_capacity());
        qb.push(" WHERE id = ").push_bind(plan.id().value());

        qb.build()
            .execute(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("update plan: {}", e)))?;
        Ok(plan.clone())
    }

    async fn delete(&self, id: &PlanId) -> AppResult<()> {
        sqlx::query("DELETE FROM plans WHERE id = $1")
            .bind(id.value())
            .execute(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("delete plan: {}", e)))?;
        Ok(())
    }

    async fn load(&self, params: LoadPlanParams) -> AppResult<Vec<Plan>> {
        let mut qb: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
            "SELECT id, name, slug, description, plan_type, plan_metadata, \
                    price, currency, is_active, is_promoted, display_order, \
                    created_by, tier_level, is_public, rate_limit_per_minute, \
                    rate_limit_per_hour, rate_limit_per_day, burst_capacity, \
                    version, created_at, updated_at, contract_address, \
                    token_address, block_number, confirmations, expires_at \
             FROM plans WHERE TRUE",
        );
        if params.active_only {
            qb.push(" AND is_active = ").push_bind(true);
        }
        if let Some(cat) = params.category {
            qb.push(" AND plan_type = ").push_bind(cat);
        }
        if let Some(tier) = params.tier_min {
            qb.push(" AND tier_level >= ").push_bind(tier);
        }
        qb.push(" ORDER BY tier_level DESC NULLS LAST, display_order ASC LIMIT ")
            .push_bind(params.limit as i64)
            .push(" OFFSET ")
            .push_bind(params.offset as i64);

        let rows: Vec<PlanDb> = qb
            .build_query_as()
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("load plans: {}", e)))?;

        let mut plans = Vec::with_capacity(rows.len());
        for row in rows {
            let slug = PlanSlug::new(row.slug.clone())
                .map_err(|e| AppError::validation_error(e.to_string()))?;
            let perm_rows: Vec<PermStringRow> = sqlx::query_as(
                "SELECT p.permission_string FROM plan_permissions pgm \
                 JOIN permissions p ON pgm.permission_id = p.id WHERE pgm.plan_id = $1",
            )
            .bind(row.id)
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(format!("load plan perms: {}", e)))?;
            let permissions: HashSet<PermissionString> = perm_rows
                .into_iter()
                .filter_map(|r| PermissionString::new(r.permission_string).ok())
                .collect();
            let plan_id = PlanId::from_uuid(row.id);
            let price_f64 = row
                .price
                .as_ref()
                .and_then(|bd| bd.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);
            let plan = Plan::new(
                plan_id,
                row.name,
                slug,
                row.description.unwrap_or_default(),
                row.plan_type,
                row.plan_metadata,
                price_f64,
                row.currency,
                row.is_active,
                row.is_promoted,
                row.display_order,
                row.created_by,
                row.tier_level,
                row.is_public,
                row.rate_limit_per_minute,
                row.rate_limit_per_hour,
                row.rate_limit_per_day,
                row.burst_capacity,
                permissions,
            )
            .map_err(|e| AppError::validation_error(format!("Invalid plan: {}", e)))?;
            plans.push(plan);
        }
        Ok(plans)
    }

    async fn search(&self, criteria: PlanSearchCriteria) -> AppResult<Vec<Plan>> {
        self.load(LoadPlanParams {
            active_only: criteria.active_only.unwrap_or(true),
            category: criteria.category,
            tier_min: criteria.tier_min,
            limit: criteria.limit.unwrap_or(50),
            offset: criteria.offset.unwrap_or(0),
        })
        .await
    }

    async fn get_statistics(&self) -> AppResult<PlanStatistics> {
        let row: PlanStatsRow = sqlx::query_as(
            "SELECT \
                COUNT(*) AS total_plans, \
                COUNT(*) FILTER (WHERE is_active = TRUE) AS active_plans, \
                COUNT(*) FILTER (WHERE is_promoted = TRUE) AS promoted_plans \
             FROM plans",
        )
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("plan stats: {}", e)))?;

        let plans_by_category = self.count_by_category().await?;
        Ok(PlanStatistics {
            total_plans: row.total_plans as u32,
            active_plans: row.active_plans as u32,
            promoted_plans: row.promoted_plans as u32,
            plans_by_category,
        })
    }

    async fn count_by_category(&self) -> AppResult<std::collections::HashMap<String, u32>> {
        #[derive(sqlx::FromRow)]
        struct CategoryCountRow {
            plan_type: String,
            count: i64,
        }
        let rows: Vec<CategoryCountRow> = sqlx::query_as(
            "SELECT plan_type, COUNT(*) AS count FROM plans GROUP BY plan_type",
        )
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(format!("count by category: {}", e)))?;

        let mut counts = std::collections::HashMap::new();
        for row in rows {
            counts.insert(row.plan_type, row.count as u32);
        }
        Ok(counts)
    }
}
/// Backward-compatible alias: many call sites still use the old name.
#[deprecated(note = "Use PlanRepositoryAdapter — old name kept for migration period")]
pub type PermissionPlanRepositoryAdapter = PlanRepositoryAdapter;
