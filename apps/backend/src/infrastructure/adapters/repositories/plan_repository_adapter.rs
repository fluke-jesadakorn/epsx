// Plan Repository Adapter
// Implements PlanRepositoryPort using Diesel and PostgreSQL
// Maps 'Plan' aggregate to 'plans' table (where plan_type = 'subscription')

use crate::prelude::*;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl};
use crate::domain::subscription_management::Price;
use tracing::error;
use std::str::FromStr;
use rust_decimal::Decimal;

use std::collections::HashMap;

use crate::domain::subscription_management::{
    aggregates::Plan,
    value_objects::PlanId,
    repository_ports::{PlanRepositoryPort, PlanSearchCriteria},
};
use crate::schemas::primary::plans;
use crate::infrastructure::models::plan::{PlanDb, NewPlanDb};
use crate::infrastructure::adapters::repositories::database_types::{PermissionRow, PlanPermissionRow};

#[derive(Clone)]
pub struct PostgresPlanRepositoryAdapter {
    db_pool: &'static TlsPool,
}

impl PostgresPlanRepositoryAdapter {
    pub fn new(db_pool: &'static TlsPool) -> Self {
        Self { db_pool }
    }

    /// Batch-fetch permissions for multiple plans in a single query
    async fn fetch_permissions_batch(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        plan_ids: &[uuid::Uuid],
    ) -> AppResult<HashMap<uuid::Uuid, Vec<String>>> {
        if plan_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let query = r#"
            SELECT pgm.plan_id, p.permission_string
            FROM plan_permissions pgm
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE pgm.plan_id = ANY($1)
        "#;

        let rows = diesel::sql_query(query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(plan_ids)
            .load::<PlanPermissionRow>(conn)
            .await
            .map_err(|e| {
                error!("Failed to batch-fetch permissions: {}", e);
                AppError::database_error(e.to_string())
            })?;

        let mut map: HashMap<uuid::Uuid, Vec<String>> = HashMap::new();
        for row in rows {
            map.entry(row.plan_id).or_default().push(row.permission_string);
        }
        Ok(map)
    }

    /// Fetch permissions for a single plan
    async fn fetch_permissions(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        plan_id: uuid::Uuid,
    ) -> AppResult<Vec<String>> {
        let query = r#"
            SELECT p.permission_string, p.platform, p.resource, p.action
            FROM plan_permissions pgm
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE pgm.plan_id = $1
        "#;

        let rows = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(plan_id)
            .load::<PermissionRow>(conn)
            .await
            .map_err(|e| {
                error!("Failed to fetch permissions for plan {}: {}", plan_id, e);
                AppError::database_error(e.to_string())
            })?;

        Ok(rows.into_iter().map(|r| r.permission_string).collect())
    }

    /// Map DB row to Plan aggregate with pre-fetched permissions
    fn map_row_to_plan(row: PlanDb, permissions: Vec<String>) -> AppResult<Plan> {
        use crate::domain::subscription_management::aggregates::plan::LoadPlanParams;
        use crate::domain::permission_management::PlanId;
        use crate::domain::subscription_management::value_objects::BillingCycle;

        let id_val = PlanId::from_uuid(row.id);
        let plan_id = PlanId::from_uuid(row.id);

        let billing_cycle = match row.billing_cycle.unwrap_or_else(|| "monthly".to_string()).as_str() {
            "monthly" => BillingCycle::Monthly,
            "yearly" => BillingCycle::Yearly,
            "one_time" | "lifetime" => BillingCycle::Lifetime,
            _ => BillingCycle::Monthly,
        };

        let price_val = Price::new(
            row.price.and_then(|p| Decimal::from_str(&p.to_string()).ok()).unwrap_or(Decimal::ZERO),
            row.currency.unwrap_or("USD".to_string())
        )?;

        Ok(Plan::reconstruct(LoadPlanParams {
            id: id_val,
            name: row.name,
            description: row.description,
            plan_id,
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
        let mut conn = self.db_pool.conn().await?;

        let plan_result = plans::table
            .filter(plans::id.eq(id.value()))
            .filter(plans::plan_type.eq("subscription"))
            .select(PlanDb::as_select())
            .first::<PlanDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find plan by id {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;

        if let Some(row) = plan_result {
            let perms = self.fetch_permissions(&mut conn, row.id).await?;
            let plan = Self::map_row_to_plan(row, perms)?;
            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }

    async fn find_all(&self, criteria: PlanSearchCriteria) -> AppResult<Vec<Plan>> {
        let mut conn = self.db_pool.conn().await?;

        let mut query = plans::table
            .filter(plans::plan_type.eq("subscription"))
            .into_boxed();

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
            plans::price.asc(),
        ));

        if let Some(limit_val) = criteria.limit {
            query = query.limit(limit_val);
        }

        if let Some(offset_val) = criteria.offset {
            query = query.offset(offset_val);
        }

        let plan_rows = query
            .select(PlanDb::as_select())
            .load::<PlanDb>(&mut conn)
            .await
            .map_err(|e| {
                 error!("Failed to find plans: {}", e);
                 AppError::database_error(e.to_string())
            })?;

        // Batch-fetch all permissions in a single query (avoids N+1)
        let plan_ids: Vec<uuid::Uuid> = plan_rows.iter().map(|r| r.id).collect();
        let mut perms_map = self.fetch_permissions_batch(&mut conn, &plan_ids).await?;

        let mut result = Vec::with_capacity(plan_rows.len());
        for row in plan_rows {
            let perms = perms_map.remove(&row.id).unwrap_or_default();
            result.push(Self::map_row_to_plan(row, perms)?);
        }

        Ok(result)
    }

    async fn save(&self, plan: &Plan) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        let price_bd = Some(bigdecimal::BigDecimal::from_str(&plan.price().amount().to_string()).unwrap_or_default());
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
             is_public: true, // Default to public for subscription plans
             plan_category: "base".to_string(),
             plan_group: "personal".to_string(),
             is_system: false,
        };

        // 1. Upsert Plan
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
                plans::plan_metadata.eq(&new_plan.plan_metadata),
                plans::updated_at.eq(new_plan.updated_at),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to save plan/plan {}: {}", plan.id(), e);
                AppError::database_error(e.to_string())
            })?;

        // 2. Handle Permissions
         use crate::schemas::primary::plan_permissions;
         
         diesel::delete(plan_permissions::table)
            .filter(plan_permissions::plan_id.eq(plan.id().value()))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

         for perm_str in &plan.permissions {
             let parts: Vec<&str> = perm_str.split(':').collect();
             if parts.len() >= 3 {
                use diesel::QueryableByName;
                 #[derive(QueryableByName)]
                 struct IdResult {
                     #[diesel(sql_type = diesel::sql_types::Uuid)]
                     id: uuid::Uuid,
                 }
                 let query = r#"
                    INSERT INTO permissions (permission_string, platform, resource, action, permission_type)
                    VALUES ($1, $2, $3, $4, 'manual')
                    ON CONFLICT (permission_string) DO UPDATE
                    SET platform = EXCLUDED.platform
                    RETURNING id
                "#;
                 let perm_id = diesel::sql_query(query)
                    .bind::<diesel::sql_types::Text, _>(perm_str)
                    .bind::<diesel::sql_types::Text, _>(parts[0])
                    .bind::<diesel::sql_types::Text, _>(parts[1])
                    .bind::<diesel::sql_types::Text, _>(parts[2])
                    .get_result::<IdResult>(&mut conn)
                    .await
                    .map(|result| result.id)
                    .map_err(|e| AppError::database_error(e.to_string()))?;

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
        Ok(())
    }

    async fn delete(&self, id: &PlanId) -> AppResult<()> {
        let mut conn = self.db_pool.conn().await?;

        diesel::delete(plans::table)
            .filter(plans::id.eq(id.value()))
            .filter(plans::plan_type.eq("subscription"))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to delete plan {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;
        Ok(())
    }

    async fn count(&self, criteria: PlanSearchCriteria) -> AppResult<i64> {
        let mut conn = self.db_pool.conn().await?;

        let mut query = plans::table
            .filter(plans::plan_type.eq("subscription"))
            .into_boxed();

        if let Some(is_active) = criteria.is_active {
            query = query.filter(plans::is_active.eq(is_active));
        }
        
         // ... same filters ...
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
                error!("Failed to count plans: {}", e);
                AppError::database_error(e.to_string())
            })?;
        Ok(count)
    }

    async fn find_active(&self) -> AppResult<Vec<Plan>> {
        self.find_all(PlanSearchCriteria {
            is_active: Some(true),
            ..Default::default()
        }).await
    }

    async fn find_promoted(&self) -> AppResult<Vec<Plan>> {
         self.find_all(PlanSearchCriteria {
            is_promoted: Some(true),
            ..Default::default()
        }).await
    }
}
