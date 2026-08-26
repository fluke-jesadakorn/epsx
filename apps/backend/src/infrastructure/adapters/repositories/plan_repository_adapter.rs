// Plan Repository Adapter (sqlx)
// Implements PlanRepositoryPort using sqlx + PostgreSQL
// Maps 'Plan' aggregate to 'plans' table (where plan_type = 'subscription')
//
// BIG-BANG: migrated to sqlx (real).

use crate::domain::subscription_management::Price;
use crate::prelude::*;
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use sqlx::QueryBuilder;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::domain::subscription_management::{
    aggregates::Plan,
    repository_ports::{PlanRepositoryPort, PlanSearchCriteria},
    value_objects::PlanId,
};
use crate::infrastructure::models::plan::{NewPlanDb, PlanDb};

#[derive(Clone)]
pub struct PostgresPlanRepositoryAdapter {
    db_pool: Arc<PgPool>,
}

impl PostgresPlanRepositoryAdapter {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }

    /// Batch-fetch permissions for multiple plans in a single query
    async fn fetch_permissions_batch(
        &self,
        plan_ids: &[Uuid],
    ) -> AppResult<HashMap<Uuid, Vec<String>>> {
        if plan_ids.is_empty() {
            return Ok(HashMap::new());
        }

        #[derive(sqlx::FromRow)]
        struct Row {
            plan_id: Uuid,
            permission_string: String,
        }

        let rows: Vec<Row> = sqlx::query_as(
            "SELECT pgm.plan_id, p.permission_string \
             FROM plan_permissions pgm \
             JOIN permissions p ON pgm.permission_id = p.id \
             WHERE pgm.plan_id = ANY($1)",
        )
        .bind(plan_ids)
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to batch-fetch permissions: {}", e);
            AppError::database_error(e.to_string())
        })?;

        let mut map: HashMap<Uuid, Vec<String>> = HashMap::new();
        for row in rows {
            map.entry(row.plan_id)
                .or_default()
                .push(row.permission_string);
        }
        Ok(map)
    }

    /// Fetch permissions for a single plan
    async fn fetch_permissions(&self, plan_id: Uuid) -> AppResult<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT p.permission_string \
             FROM plan_permissions pgm \
             JOIN permissions p ON pgm.permission_id = p.id \
             WHERE pgm.plan_id = $1",
        )
        .bind(plan_id)
        .fetch_all(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to fetch permissions for plan {}: {}", plan_id, e);
            AppError::database_error(e.to_string())
        })?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Map DB row to Plan aggregate with pre-fetched permissions
    fn map_row_to_plan(row: PlanDb, permissions: Vec<String>) -> AppResult<Plan> {
        use crate::domain::permission_management::PlanId;
        use crate::domain::subscription_management::aggregates::plan::LoadPlanParams;
        use crate::domain::subscription_management::value_objects::BillingCycle;

        let id_val = PlanId::from_uuid(row.id);

        let billing_cycle = match row
            .billing_cycle
            .unwrap_or_else(|| "monthly".to_string())
            .as_str()
        {
            "monthly" => BillingCycle::Monthly,
            "yearly" => BillingCycle::Yearly,
            "one_time" | "lifetime" => BillingCycle::Lifetime,
            _ => BillingCycle::Monthly,
        };

        let price_val = Price::new(
            row.price
                .and_then(|p| Decimal::from_str(&p.to_string()).ok())
                .unwrap_or(Decimal::ZERO),
            row.currency.unwrap_or("USD".to_string()),
        )?;

        Ok(Plan::reconstruct(LoadPlanParams {
            id: id_val.clone(),
            name: row.name,
            description: row.description,
            plan_id: id_val,
            permissions,
            quotas: HashMap::new(),
            price: price_val,
            billing_cycle,
            features: Default::default(),
            target_audience: "all".to_string(),
            is_active: row.is_active,
            is_promoted: row.is_promoted,
            tier_level: row.tier_level,
            metadata: row.plan_metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
            version: 1,
        }))
    }
}

#[async_trait]
impl PlanRepositoryPort for PostgresPlanRepositoryAdapter {
    async fn find_by_id(&self, id: &PlanId) -> AppResult<Option<Plan>> {
        let row: Option<PlanDb> = sqlx::query_as(
            "SELECT id, name, slug, description, plan_type, plan_metadata, \
                    price, currency, billing_cycle, is_active, is_promoted, \
                    tier_level, max_members, auto_assign_enabled, assignment_rules, \
                    created_at, updated_at, created_by, last_modified_by, \
                    grace_period_hours, rate_limit_per_minute, rate_limit_per_hour, \
                    rate_limit_per_day, burst_capacity, is_public, plan_category, \
                    plan_group, is_system, version, contract_address, token_address, \
                    block_number, confirmations, expires_at, display_order \
             FROM plans WHERE id = $1 AND plan_type = 'subscription'",
        )
        .bind(id.value())
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to find plan by id {}: {}", id, e);
            AppError::database_error(e.to_string())
        })?;

        let Some(row) = row else { return Ok(None) };
        let perms = self.fetch_permissions(row.id).await?;
        let plan = Self::map_row_to_plan(row, perms)?;
        Ok(Some(plan))
    }

    async fn find_all(&self, criteria: PlanSearchCriteria) -> AppResult<Vec<Plan>> {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT id, name, slug, description, plan_type, plan_metadata, \
                    price, currency, billing_cycle, is_active, is_promoted, \
                    tier_level, max_members, auto_assign_enabled, assignment_rules, \
                    created_at, updated_at, created_by, last_modified_by, \
                    grace_period_hours, rate_limit_per_minute, rate_limit_per_hour, \
                    rate_limit_per_day, burst_capacity, is_public, plan_category, \
                    plan_group, is_system, version, contract_address, token_address, \
                    block_number, confirmations, expires_at, display_order \
             FROM plans WHERE plan_type = 'subscription'",
        );
        if let Some(is_active) = criteria.is_active {
            qb.push(" AND is_active = ").push_bind(is_active);
        }
        if let Some(is_promoted) = criteria.is_promoted {
            qb.push(" AND is_promoted = ").push_bind(is_promoted);
        }
        if let Some(search_term) = &criteria.search_term {
            let pattern = format!("%{}%", search_term);
            qb.push(" AND (name ILIKE ").push_bind(pattern.clone())
                .push(" OR description ILIKE ").push_bind(pattern)
                .push(")");
        }
        qb.push(" ORDER BY tier_level ASC, price ASC");
        if let Some(limit_val) = criteria.limit {
            qb.push(" LIMIT ").push_bind(limit_val as i64);
        }
        if let Some(offset_val) = criteria.offset {
            qb.push(" OFFSET ").push_bind(offset_val as i64);
        }

        let plan_rows: Vec<PlanDb> = qb
            .build_query_as()
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to find plans: {}", e);
                AppError::database_error(e.to_string())
            })?;

        let plan_ids: Vec<Uuid> = plan_rows.iter().map(|r| r.id).collect();
        let mut perms_map = self.fetch_permissions_batch(&plan_ids).await?;

        let mut result = Vec::with_capacity(plan_rows.len());
        for row in plan_rows {
            let perms = perms_map.remove(&row.id).unwrap_or_default();
            result.push(Self::map_row_to_plan(row, perms)?);
        }
        Ok(result)
    }

    async fn save(&self, plan: &Plan) -> AppResult<()> {
        let price_bd = Some(
            Decimal::from_str(&plan.price().amount().to_string())
                .unwrap_or_default(),
        );
        let currency_str = Some(plan.price().currency().to_string());
        let billing_cycle_str = Some(plan.billing_cycle().to_string());

        let new_plan = NewPlanDb {
            id: *plan.id().value(),
            name: plan.name().to_string(),
            slug: plan.name().to_lowercase().replace(" ", "-"),
            description: plan.description().to_string(),
            plan_type: "subscription".to_string(),
            plan_metadata: serde_json::json!({
                "permissions": plan.permissions
            }),
            price: price_bd,
            currency: currency_str,
            billing_cycle: billing_cycle_str,
            is_active: plan.is_active(),
            is_promoted: plan.is_promoted(),
            tier_level: plan.tier_level(),
            max_members: None,
            auto_assign_enabled: Some(false),
            assignment_rules: None,
            created_at: plan.created_at(),
            updated_at: plan.updated_at(),
            created_by: None,
            last_modified_by: None,
            grace_period_hours: 0,
            rate_limit_per_minute: 0,
            rate_limit_per_hour: 0,
            rate_limit_per_day: 0,
            burst_capacity: 0,
            is_public: true,
            plan_category: "base".to_string(),
            plan_group: "personal".to_string(),
            is_system: false,
        };

        // 1. Upsert Plan via sqlx ON CONFLICT
        sqlx::query(
            r#"
            INSERT INTO plans (
                id, name, slug, description, plan_type, plan_metadata,
                price, currency, billing_cycle, is_active, is_promoted,
                tier_level, auto_assign_enabled, created_at, updated_at, is_public,
                plan_category, plan_group, is_system
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11,
                $12, $13, $14, $15, $16,
                $17, $18, $19
            )
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                price = EXCLUDED.price,
                currency = EXCLUDED.currency,
                billing_cycle = EXCLUDED.billing_cycle,
                is_active = EXCLUDED.is_active,
                is_promoted = EXCLUDED.is_promoted,
                tier_level = EXCLUDED.tier_level,
                plan_metadata = EXCLUDED.plan_metadata,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(new_plan.id)
        .bind(&new_plan.name)
        .bind(&new_plan.slug)
        .bind(&new_plan.description)
        .bind(&new_plan.plan_type)
        .bind(&new_plan.plan_metadata)
        .bind(new_plan.price)
        .bind(&new_plan.currency)
        .bind(&new_plan.billing_cycle)
        .bind(new_plan.is_active)
        .bind(new_plan.is_promoted)
        .bind(new_plan.tier_level)
        .bind(new_plan.auto_assign_enabled.unwrap_or(false))
        .bind(new_plan.created_at)
        .bind(new_plan.updated_at)
        .bind(new_plan.is_public)
        .bind(&new_plan.plan_category)
        .bind(&new_plan.plan_group)
        .bind(new_plan.is_system)
        .execute(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to save plan/plan {}: {}", plan.id(), e);
            AppError::database_error(e.to_string())
        })?;

        // 2. Delete existing plan_permissions for this plan
        sqlx::query("DELETE FROM plan_permissions WHERE plan_id = $1")
            .bind(plan.id().value())
            .execute(self.db_pool.as_ref())
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // 3. Insert permissions (upsert and link to plan)
        #[derive(sqlx::FromRow)]
        struct IdResult {
            id: Uuid,
        }

        for perm_str in &plan.permissions {
            let parts: Vec<&str> = perm_str.split(':').collect();
            if parts.len() >= 3 {
                let perm_id: IdResult = sqlx::query_as(
                    "INSERT INTO permissions (permission_string, platform, resource, action, permission_type) \
                     VALUES ($1, $2, $3, $4, 'manual') \
                     ON CONFLICT (permission_string) DO UPDATE \
                     SET platform = EXCLUDED.platform \
                     RETURNING id",
                )
                .bind(perm_str)
                .bind(parts[0])
                .bind(parts[1])
                .bind(parts[2])
                .fetch_one(self.db_pool.as_ref())
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?;

                sqlx::query("INSERT INTO plan_permissions (plan_id, permission_id) VALUES ($1, $2)")
                    .bind(plan.id().value())
                    .bind(perm_id.id)
                    .execute(self.db_pool.as_ref())
                    .await
                    .map_err(|e| AppError::database_error(e.to_string()))?;
            }
        }
        Ok(())
    }

    async fn delete(&self, id: &PlanId) -> AppResult<()> {
        sqlx::query("DELETE FROM plans WHERE id = $1 AND plan_type = 'subscription'")
            .bind(id.value())
            .execute(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to delete plan {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;
        Ok(())
    }

    async fn count(&self, criteria: PlanSearchCriteria) -> AppResult<i64> {
        let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
            "SELECT COUNT(*) AS c FROM plans WHERE plan_type = 'subscription'",
        );
        if let Some(is_active) = criteria.is_active {
            qb.push(" AND is_active = ").push_bind(is_active);
        }
        if let Some(search_term) = &criteria.search_term {
            let pattern = format!("%{}%", search_term);
            qb.push(" AND (name ILIKE ").push_bind(pattern.clone())
                .push(" OR description ILIKE ").push_bind(pattern)
                .push(")");
        }

        let row: (i64,) = qb
            .build_query_as()
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to count plans: {}", e);
                AppError::database_error(e.to_string())
            })?;
        Ok(row.0)
    }

    async fn find_active(&self) -> AppResult<Vec<Plan>> {
        self.find_all(PlanSearchCriteria {
            is_active: Some(true),
            ..Default::default()
        })
        .await
    }

    async fn find_promoted(&self) -> AppResult<Vec<Plan>> {
        self.find_all(PlanSearchCriteria {
            is_promoted: Some(true),
            ..Default::default()
        })
        .await
    }
}