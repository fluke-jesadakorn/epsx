// Plan Repository Adapter
// Implements PlanRepositoryPort using Diesel and PostgreSQL
// Maps 'Plan' aggregate to 'plans' table (where plan_type = 'subscription')

use crate::prelude::*;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};
use crate::domain::subscription_management::Price;
use tracing::error;
use std::str::FromStr;
use rust_decimal::Decimal;

use crate::domain::subscription_management::{
    aggregates::Plan,
    value_objects::PlanId,
    repository_ports::{PlanRepositoryPort, PlanSearchCriteria},
};
use crate::schemas::primary::plans;
use crate::infrastructure::models::plan::{PlanDb, NewPlanDb};
use crate::infrastructure::adapters::repositories::database_types::PermissionRow;

#[derive(Clone)]
pub struct PostgresPlanRepositoryAdapter {
    db_pool: &'static Pool<AsyncPgConnection>,
}

impl PostgresPlanRepositoryAdapter {
    pub fn new(db_pool: &'static Pool<AsyncPgConnection>) -> Self {
        Self { db_pool }
    }

    // Helper to map DB row to Aggregate
    async fn map_row_to_plan(&self, conn: &mut AsyncPgConnection, row: PlanDb) -> AppResult<Plan> {
         // Fetch permissions
         let query = r#"
            SELECT p.permission_string, p.platform, p.resource, p.action
            FROM plan_permissions pgm
            JOIN permissions p ON pgm.permission_id = p.id
            WHERE pgm.plan_id = $1
        "#;

        let permission_rows = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(row.id)
            .load::<PermissionRow>(conn)
            .await
            .map_err(|e| {
                error!("Failed to fetch permissions for plan {}: {}", row.id, e);
                AppError::database_error(e.to_string())
            })?;

        let permissions: Vec<String> = permission_rows.into_iter()
            .map(|r| r.action) // Simplified mapping to one of the fields if string is gone
            .collect();

        // Reconstruct Plan Aggregate
        use crate::domain::subscription_management::aggregates::plan::LoadPlanParams;
        use crate::domain::permission_management::PlanId;

        let id_val = PlanId::from_uuid(row.id);
        let plan_id = PlanId::from_uuid(row.id);
        
        // Handle billing cycle string conversion
        use crate::domain::subscription_management::value_objects::BillingCycle;
        let billing_cycle = match row.billing_cycle.unwrap_or_else(|| "monthly".to_string()).as_str() {
            "monthly" => BillingCycle::Monthly,
            "yearly" => BillingCycle::Yearly,
            "one_time" | "lifetime" => BillingCycle::Lifetime,
            _ => BillingCycle::Monthly, // Default
        };

        // Handle Price
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
            quotas: std::collections::HashMap::new(), // Will be recalculated or fetched if needed
            price: price_val,
            billing_cycle, 
            features: Default::default(), // Future: Load from metadata
            target_audience: "all".to_string(), // Future: Load from metadata
            is_active: row.is_active,
            is_promoted: row.is_promoted,
            display_order: row.display_order.unwrap_or(0),
            metadata: row.plan_metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
            version: 1, // Placeholder
        }))
    }
}

#[async_trait]
impl PlanRepositoryPort for PostgresPlanRepositoryAdapter {
    async fn find_by_id(&self, id: &PlanId) -> AppResult<Option<Plan>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

         let plan_result = plans::table
            .filter(plans::id.eq(id.value()))
            .filter(plans::plan_type.eq("subscription"))
            .first::<PlanDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find plan by id {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;
        
        if let Some(row) = plan_result {
            let plan = self.map_row_to_plan(&mut conn, row).await?;
            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }

    async fn find_all(&self, criteria: PlanSearchCriteria) -> AppResult<Vec<Plan>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

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
            plans::display_order.asc(), // Assuming non-nullable, or we handle nulls
            plans::price.asc(),
        ));

        if let Some(limit_val) = criteria.limit {
            query = query.limit(limit_val);
        }

        if let Some(offset_val) = criteria.offset {
            query = query.offset(offset_val);
        }

        let plan_rows = query
            .load::<PlanDb>(&mut conn)
            .await
            .map_err(|e| {
                 error!("Failed to find plans: {}", e);
                 AppError::database_error(e.to_string())
            })?;

        let mut plans = Vec::new();
        for row in plan_rows {
            // Need to fetch permissions for EACH plan. N+1 problem unless joined.
            // For now, simple loop is fine as plans are few (< 20).
            let plan = self.map_row_to_plan(&mut conn, row).await?;
            plans.push(plan);
        }
        
        Ok(plans)
    }

    async fn save(&self, plan: &Plan) -> AppResult<()> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

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
             display_order: Some(plan.display_order()),
             max_members: None, 
             auto_assign_enabled: Some(false),
             assignment_rules: None,
             created_at: plan.created_at(),
             updated_at: plan.updated_at(),
             created_by: None,
             last_modified_by: None,
             rate_limit_per_minute: 0,
             rate_limit_per_hour: 0,
             rate_limit_per_day: 0,
             burst_capacity: 0,
             tier_level: 0, 
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
                plans::display_order.eq(&new_plan.display_order),
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
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

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
        let mut conn = self.db_pool.get().await
             .map_err(|e| AppError::database_error(e.to_string()))?;

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
