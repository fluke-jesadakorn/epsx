// Plan Repository Adapter (Infrastructure Layer)
// PostgreSQL implementation of PlanRepositoryPort using sqlx
//
// BIG-BANG: migrated to sqlx (real). All diesel DSL/derive replaced with raw SQL.

use crate::prelude::*;
use sqlx::{PgPool, QueryBuilder};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::error;

use crate::domain::permission_management::{
    repository_ports::{PlanRepositoryPort, PlanSearchCriteria, PlanStatistics},
    PermissionString, Plan, PlanId, PlanSlug,
};
use crate::infrastructure::models::plan::PlanDb;

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

/// Helper to map a PlanDb row + permissions to Plan aggregate
fn row_to_plan(row: PlanDb, permissions: HashSet<PermissionString>) -> Result<Plan, AppError> {
    use crate::domain::permission_management::PlanCategory;
    let plan_id = PlanId::from_uuid(row.id);
    let slug = PlanSlug::new(row.slug)
        .map_err(|e| AppError::validation_error(e.to_string()))?;
    let price_f64 = row
        .price
        .as_ref()
        .and_then(|bd| bd.to_string().parse::<f64>().ok())
        .unwrap_or(0.0);
    Plan::new(
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
    .map_err(|e| AppError::validation_error(format!("Invalid plan: {}", e)))
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

    /// Fetch permissions for a single plan
    async fn fetch_permissions(&self, plan_id: PlanId) -> AppResult<HashSet<PermissionString>> {
        let rows: Vec<PermStringRow> = sqlx::query_as(
            "SELECT p.permission_string FROM plan_permissions pgm \
             JOIN permissions p ON pgm.permission_id = p.id WHERE pgm.plan_id = $1",
        )
        .bind(plan_id.value())
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|r| PermissionString::new(r.permission_string).ok())
            .collect())
    }

    /// Fetch permissions for multiple plans in a single query
    async fn fetch_permissions_batch(
        &self,
        plan_ids: &[uuid::Uuid],
    ) -> AppResult<std::collections::HashMap<uuid::Uuid, Vec<String>>> {
        use std::collections::HashMap;
        if plan_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let rows: Vec<(uuid::Uuid, String)> = sqlx::query_as(
            "SELECT pgm.plan_id, p.permission_string \
             FROM plan_permissions pgm \
             JOIN permissions p ON pgm.permission_id = p.id WHERE pgm.plan_id = ANY($1)",
        )
        .bind(plan_ids)
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let mut map: HashMap<uuid::Uuid, Vec<String>> = HashMap::new();
        for (pid, perm) in rows {
            map.entry(pid).or_default().push(perm);
        }
        Ok(map)
    }
}

#[async_trait]
impl PlanRepositoryPort for PlanRepositoryAdapter {
    async fn find_by_id(&self, id: &PlanId) -> AppResult<Option<Plan>> {
        let row: Option<PlanDb> = sqlx::query_as(
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
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let Some(row) = row else { return Ok(None) };
        let permissions = self.fetch_permissions(*id).await?;
        Ok(Some(row_to_plan(row, permissions)?))
    }

    async fn find_by_slug(&self, slug: &PlanSlug) -> AppResult<Option<Plan>> {
        let row: Option<PlanDb> = sqlx::query_as(
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

        let Some(row) = row else { return Ok(None) };
        let permissions = self.fetch_permissions(PlanId::from_uuid(row.id)).await?;
        Ok(Some(row_to_plan(row, permissions)?))
    }

    async fn find_all(&self, criteria: PlanSearchCriteria) -> AppResult<Vec<Plan>> {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, name, slug, description, plan_type, plan_metadata, \
                    price, currency, is_active, is_promoted, display_order, \
                    created_by, tier_level, is_public, rate_limit_per_minute, \
                    rate_limit_per_hour, rate_limit_per_day, burst_capacity, \
                    version, created_at, updated_at, contract_address, \
                    token_address, block_number, confirmations, expires_at \
             FROM plans WHERE TRUE",
        );
        if let Some(ref plan_type) = criteria.plan_type {
            qb.push(" AND plan_type = ").push_bind(plan_type.clone());
        }
        if let Some(is_active) = criteria.is_active {
            qb.push(" AND is_active = ").push_bind(is_active);
        }
        if let Some(is_promoted) = criteria.is_promoted {
            qb.push(" AND is_promoted = ").push_bind(is_promoted);
        }
        if let Some(ref plan_group) = criteria.plan_group {
            qb.push(" AND plan_type = ").push_bind(plan_group.clone());
        }
        if let Some(ref search_term) = criteria.search_term {
            let pattern = format!("%{}%", search_term);
            qb.push(" AND (name ILIKE ").push_bind(pattern.clone());
            qb.push(" OR description ILIKE ").push_bind(pattern);
            qb.push(")");
        }
        let limit = criteria.limit.unwrap_or(50);
        let offset = criteria.offset.unwrap_or(0);
        qb.push(" ORDER BY tier_level DESC NULLS LAST, name ASC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows: Vec<PlanDb> = qb
            .build_query_as()
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let plan_ids: Vec<uuid::Uuid> = rows.iter().map(|r| r.id).collect();
        let mut perms_map = self.fetch_permissions_batch(&plan_ids).await?;

        let mut plans = Vec::with_capacity(rows.len());
        for row in rows {
            let perms: HashSet<PermissionString> = perms_map
                .remove(&row.id)
                .unwrap_or_default()
                .into_iter()
                .filter_map(PermissionString::new)
                .collect();
            plans.push(row_to_plan(row, perms)?);
        }
        Ok(plans)
    }

    async fn save(&self, plan: &Plan) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO plans (
                id, name, slug, description, plan_type, plan_metadata,
                price, currency, is_active, is_promoted, display_order,
                created_by, tier_level, is_public, rate_limit_per_minute,
                rate_limit_per_hour, rate_limit_per_day, burst_capacity, version,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, NOW(), NOW()
            )
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                slug = EXCLUDED.slug,
                description = EXCLUDED.description,
                plan_type = EXCLUDED.plan_type,
                plan_metadata = EXCLUDED.plan_metadata,
                price = EXCLUDED.price,
                currency = EXCLUDED.currency,
                is_active = EXCLUDED.is_active,
                is_promoted = EXCLUDED.is_promoted,
                display_order = EXCLUDED.display_order,
                created_by = EXCLUDED.created_by,
                tier_level = EXCLUDED.tier_level,
                is_public = EXCLUDED.is_public,
                rate_limit_per_minute = EXCLUDED.rate_limit_per_minute,
                rate_limit_per_hour = EXCLUDED.rate_limit_per_hour,
                rate_limit_per_day = EXCLUDED.rate_limit_per_day,
                burst_capacity = EXCLUDED.burst_capacity,
                version = EXCLUDED.version,
                updated_at = NOW()
            "#,
        )
        .bind(plan.id().value())
        .bind(plan.name().to_string())
        .bind(plan.slug().value().to_string())
        .bind(plan.description().to_string())
        .bind(plan.plan_type().to_string())
        .bind(plan.metadata().clone())
        .bind(plan.price())
        .bind(plan.currency().to_string())
        .bind(plan.is_active())
        .bind(plan.is_promoted())
        .bind(plan.display_order())
        .bind(plan.created_by().to_string())
        .bind(plan.tier_level())
        .bind(plan.is_public())
        .bind(plan.rate_limit_per_minute())
        .bind(plan.rate_limit_per_hour())
        .bind(plan.rate_limit_per_day())
        .bind(plan.burst_capacity())
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &PlanId) -> AppResult<()> {
        sqlx::query("DELETE FROM plans WHERE id = $1")
            .bind(id.value())
            .execute(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(())
    }

    async fn count(&self, criteria: PlanSearchCriteria) -> AppResult<i64> {
        let mut qb: QueryBuilder<sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) as count FROM plans WHERE TRUE");
        if let Some(ref plan_type) = criteria.plan_type {
            qb.push(" AND plan_type = ").push_bind(plan_type.clone());
        }
        if let Some(is_active) = criteria.is_active {
            qb.push(" AND is_active = ").push_bind(is_active);
        }
        if let Some(is_promoted) = criteria.is_promoted {
            qb.push(" AND is_promoted = ").push_bind(is_promoted);
        }
        if let Some(ref plan_group) = criteria.plan_group {
            qb.push(" AND plan_type = ").push_bind(plan_group.clone());
        }
        if let Some(ref search_term) = criteria.search_term {
            let pattern = format!("%{}%", search_term);
            qb.push(" AND (name ILIKE ").push_bind(pattern.clone());
            qb.push(" OR description ILIKE ").push_bind(pattern);
            qb.push(")");
        }

        let row: CountResult = qb
            .build_query_as()
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(row.count)
    }

    async fn get_statistics(&self) -> AppResult<PlanStatistics> {
        let row: PlanStatsRow = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total_plans,
                COUNT(*) FILTER (WHERE is_active = TRUE) as active_plans,
                COUNT(*) FILTER (WHERE is_promoted = TRUE) as promoted_plans,
                COUNT(*) as total_members
            FROM plans
            "#,
        )
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(PlanStatistics {
            total_plans: row.total_plans as u32,
            active_plans: row.active_plans as u32,
            promoted_plans: row.promoted_plans as u32,
            total_members: row.total_plans as u32, // No separate members table, reuse count
        })
    }

    async fn slug_exists(&self, slug: &PlanSlug) -> AppResult<bool> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM plans WHERE slug = $1")
            .bind(slug.value())
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        Ok(row.0 > 0)
    }
}
