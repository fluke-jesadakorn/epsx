// Group Repository Adapter (Infrastructure Layer)
// PostgreSQL implementation of GroupRepositoryPort using Diesel
// (Previously PermissionGroupRepositoryAdapter - renamed for clarity)
use crate::prelude::*;
use tracing::{error, info};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl, pooled_connection::deadpool::Pool};

use crate::domain::permission_management::{
    Group, GroupId, GroupSlug, PermissionString,
    repository_ports::{GroupRepositoryPort, GroupSearchCriteria, GroupStatistics},
    aggregates::group::LoadGroupParams,
};
use crate::infrastructure::models::group::{GroupDb, NewGroupDb};
use crate::infrastructure::adapters::repositories::database_types::PermissionRow;
use crate::schemas::primary::{groups, group_permissions};
use std::collections::HashSet;

#[derive(diesel::QueryableByName)]
struct IdResult {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
}

#[derive(diesel::QueryableByName)]
struct GroupStatsRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub total_groups: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub active_groups: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub promoted_groups: i64,
}

#[derive(diesel::QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

/// PostgreSQL implementation of GroupRepositoryPort using Diesel
#[derive(Clone)]
pub struct GroupRepositoryAdapter {
    db_pool: &'static Pool<AsyncPgConnection>,
}

impl GroupRepositoryAdapter {
    pub fn new(db_pool: &'static Pool<AsyncPgConnection>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl GroupRepositoryPort for GroupRepositoryAdapter {
    async fn find_by_id(&self, id: &GroupId) -> AppResult<Option<Group>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let group_result = groups::table
            .filter(groups::id.eq(id.value()))
            .first::<GroupDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find permission group by id {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;

        if let Some(row) = group_result {
            // Get permissions for this group using raw SQL (JOIN query)
            let query = r#"
                SELECT p.platform, p.resource, p.action
                FROM group_permissions pgm
                JOIN permissions p ON pgm.permission_id = p.id
                WHERE pgm.group_id = $1
            "#;

            let permission_rows = diesel::sql_query(query)
                .bind::<diesel::sql_types::Uuid, _>(id.value())
                .load::<PermissionRow>(&mut conn)
                .await
                .map_err(|e| {
                    error!("Failed to fetch permissions for group {}: {}", id, e);
                    AppError::database_error(e.to_string())
                })?;

            let permissions: HashSet<PermissionString> = permission_rows
                .iter()
                .filter_map(|r| {
                    let perm_str = format!("{}:{}:{}", r.platform, r.resource, r.action);
                    PermissionString::new(perm_str).ok()
                })
                .collect();

            let group_id = GroupId::from_uuid(row.id);
            let slug = GroupSlug::new(row.slug)
                .map_err(|e| AppError::validation_error(e.to_string()))?;

            // Convert BigDecimal to f64 for domain model
            let price_f64 = row.price
                .and_then(|bd| bd.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);

            let group = Group::load(LoadGroupParams {
                id: group_id,
                name: row.name,
                slug,
                description: row.description,
                group_type: row.group_type,
                permissions,
                price: price_f64,
                currency: row.currency.unwrap_or_else(|| "USD".to_string()),
                billing_cycle: row.billing_cycle.unwrap_or_else(|| "monthly".to_string()),
                is_active: row.is_active,
                is_promoted: row.is_promoted,
                display_order: row.display_order.unwrap_or(0),
                max_members: row.max_members,
                auto_assign_enabled: row.auto_assign_enabled.unwrap_or(false),
                metadata: row.group_metadata,
                created_at: row.created_at,
                updated_at: row.updated_at,
                version: 1,
            });

            Ok(Some(group))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug(&self, slug: &GroupSlug) -> AppResult<Option<Group>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let id_result = groups::table
            .filter(groups::slug.eq(slug.as_str()))
            .select(groups::id)
            .first::<uuid::Uuid>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!("Failed to find permission group by slug {}: {}", slug, e);
                AppError::database_error(e.to_string())
            })?;

        if let Some(id_uuid) = id_result {
            let group_id = GroupId::from_uuid(id_uuid);
            self.find_by_id(&group_id).await
        } else {
            Ok(None)
        }
    }

    async fn find_all(&self, criteria: GroupSearchCriteria) -> AppResult<Vec<Group>> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Build dynamic query using Diesel DSL
        let mut query = groups::table.into_boxed();

        if let Some(group_type) = &criteria.group_type {
            query = query.filter(groups::group_type.eq(group_type));
        }

        if let Some(is_active) = criteria.is_active {
            query = query.filter(groups::is_active.eq(is_active));
        }

        if let Some(is_promoted) = criteria.is_promoted {
            query = query.filter(groups::is_promoted.eq(is_promoted));
        }

        if let Some(search_term) = &criteria.search_term {
            let pattern = format!("%{}%", search_term);
            let p = pattern.clone();
            query = query.filter(
                groups::name.ilike(pattern)
                    .or(groups::description.ilike(p))
            );
        }

        query = query.order((
            groups::display_order.asc(),
            groups::created_at.desc(),
        ));

        if let Some(limit_val) = criteria.limit {
            query = query.limit(limit_val);
        }

        if let Some(offset_val) = criteria.offset {
            query = query.offset(offset_val);
        }

        let group_ids = query
            .select(groups::id)
            .load::<uuid::Uuid>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to find permission groups: {}", e);
                AppError::database_error(e.to_string())
            })?;

        let mut groups = Vec::new();
        for id_uuid in group_ids {
            let group_id = GroupId::from_uuid(id_uuid);
            if let Some(group) = self.find_by_id(&group_id).await? {
                groups.push(group);
            }
        }

        Ok(groups)
    }

    async fn save(&self, group: &Group) -> AppResult<()> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let new_group = NewGroupDb {
            id: *group.id().value(),
            name: group.name().to_string(),
            slug: group.slug().as_str().to_string(),
            description: group.description().to_string(),
            group_type: group.group_type().to_string(),
            group_metadata: group.metadata().clone(),
            price: group.price().to_string().parse::<bigdecimal::BigDecimal>().ok(),
            currency: Some(group.currency().to_string()),
            billing_cycle: Some(group.billing_cycle().to_string()),
            is_active: group.is_active(),
            is_promoted: group.is_promoted(),
            display_order: Some(group.display_order()),
            max_members: group.max_members(),
            auto_assign_enabled: Some(group.auto_assign_enabled()),
            assignment_rules: None,
            created_at: group.created_at(),
            updated_at: group.updated_at(),
            created_by: None,
            last_modified_by: None,
            rate_limit_per_minute: 0,
            rate_limit_per_hour: 0,
            rate_limit_per_day: 0,
            burst_capacity: 0,
            tier_level: 0, // Default to free tier
        };

        // Upsert permission group
        diesel::insert_into(groups::table)
            .values(&new_group)
            .on_conflict(groups::id)
            .do_update()
            .set((
                groups::name.eq(&new_group.name),
                groups::description.eq(&new_group.description),
                groups::price.eq(&new_group.price),
                groups::currency.eq(&new_group.currency),
                groups::billing_cycle.eq(&new_group.billing_cycle),
                groups::is_active.eq(new_group.is_active),
                groups::is_promoted.eq(new_group.is_promoted),
                groups::display_order.eq(&new_group.display_order),
                groups::max_members.eq(&new_group.max_members),
                groups::auto_assign_enabled.eq(&new_group.auto_assign_enabled),
                groups::group_metadata.eq(&new_group.group_metadata),
                groups::updated_at.eq(new_group.updated_at),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to save permission group: {}", e);
                AppError::database_error(e.to_string())
            })?;

        // Delete existing permission associations
        diesel::delete(group_permissions::table)
            .filter(group_permissions::group_id.eq(group.id().value()))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Insert permission associations
        // Note: Sequential execution instead of transaction (following wallet_user_repository pattern)
        for permission in group.permissions() {
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

                // Link permission to group
                diesel::sql_query(
                    r#"
                    INSERT INTO group_permissions (group_id, permission_id)
                    VALUES ($1, $2)
                    "#
                )
                .bind::<diesel::sql_types::Uuid, _>(group.id().value())
                .bind::<diesel::sql_types::Uuid, _>(perm_id)
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?;
            }
        }

        info!("Permission group {} saved successfully", group.id());
        Ok(())
    }

    async fn delete(&self, id: &GroupId) -> AppResult<()> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        diesel::delete(groups::table)
            .filter(groups::id.eq(id.value()))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to delete permission group {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;

        info!("Permission group {} deleted successfully", id);
        Ok(())
    }

    async fn count(&self, criteria: GroupSearchCriteria) -> AppResult<i64> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let mut query = groups::table.into_boxed();

        if let Some(group_type) = &criteria.group_type {
            query = query.filter(groups::group_type.eq(group_type));
        }

        if let Some(is_active) = criteria.is_active {
            query = query.filter(groups::is_active.eq(is_active));
        }

        if let Some(is_promoted) = criteria.is_promoted {
            query = query.filter(groups::is_promoted.eq(is_promoted));
        }

        if let Some(search_term) = &criteria.search_term {
            let pattern = format!("%{}%", search_term);
            let p = pattern.clone();
            query = query.filter(
                groups::name.ilike(pattern)
                    .or(groups::description.ilike(p))
            );
        }

        let count = query
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to count permission groups: {}", e);
                AppError::database_error(e.to_string())
            })?;

        Ok(count)
    }

    async fn get_statistics(&self) -> AppResult<GroupStatistics> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Use diesel::sql_query for FILTER clause compatibility
        let query = r#"
            SELECT
                COUNT(*) as total_groups,
                COUNT(*) FILTER (WHERE is_active = true) as active_groups,
                COUNT(*) FILTER (WHERE is_promoted = true) as promoted_groups
            FROM groups
        "#;

        let row = diesel::sql_query(query)
            .get_result::<GroupStatsRow>(&mut conn)
            .await
            .map_err(|e| {
                AppError::database_error(format!("Failed to get group statistics: {}", e))
            })?;

        let total_members: i64 = diesel::sql_query(
            "SELECT COUNT(DISTINCT wallet_address) as count FROM wallet_group_assignments"
        )
        .get_result::<CountResult>(&mut conn)
        .await
        .map(|result| result.count)
        .unwrap_or(0);

        Ok(GroupStatistics {
            total_groups: row.total_groups,
            active_groups: row.active_groups,
            promoted_groups: row.promoted_groups,
            total_members,
        })
    }

    async fn slug_exists(&self, slug: &GroupSlug) -> AppResult<bool> {
        let mut conn = self.db_pool.get().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let exists = diesel::select(diesel::dsl::exists(
            groups::table.filter(groups::slug.eq(slug.as_str()))
        ))
        .get_result::<bool>(&mut conn)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(exists)
    }
}

// Additional helper methods for subscription plan management
impl GroupRepositoryAdapter {
    /// Get all subscription plans (database_types.rs compatibility layer)
    pub async fn get_subscription_plans(&self) -> Result<Vec<crate::infrastructure::adapters::repositories::database_types::PermissionGroup>, diesel::result::Error> {
        use crate::infrastructure::adapters::repositories::database_types::PermissionGroup as DbPermissionGroup;
        use crate::schemas::primary::groups;

        let mut conn = self.db_pool.get().await
            .map_err(|e| diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string())
            ))?;

        groups::table
            .filter(groups::group_type.eq("subscription"))
            .order_by((
                groups::display_order.assume_not_null().asc(),
                groups::price.assume_not_null().asc()
            ))
            .load::<DbPermissionGroup>(&mut conn)
            .await
    }

    /// Get plan by ID (database_types.rs compatibility layer)
    pub async fn get_plan_by_id(&self, plan_id: uuid::Uuid) -> Result<Option<crate::infrastructure::adapters::repositories::database_types::PermissionGroup>, diesel::result::Error> {
        use crate::infrastructure::adapters::repositories::database_types::PermissionGroup as DbPermissionGroup;
        use crate::schemas::primary::groups;

        let mut conn = self.db_pool.get().await
            .map_err(|e| diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string())
            ))?;

        groups::table
            .filter(groups::id.eq(plan_id))
            .filter(groups::group_type.eq("subscription"))
            .first::<DbPermissionGroup>(&mut conn)
            .await
            .optional()
    }

    /// Update plan (database_types.rs compatibility layer)
    pub async fn update_plan(&self, plan: crate::infrastructure::adapters::repositories::database_types::PermissionGroup) -> Result<crate::infrastructure::adapters::repositories::database_types::PermissionGroup, diesel::result::Error> {
        use crate::infrastructure::adapters::repositories::database_types::PermissionGroup as DbPermissionGroup;
        use crate::schemas::primary::groups;

        let mut conn = self.db_pool.get().await
            .map_err(|e| diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string())
            ))?;

        diesel::update(groups::table.filter(groups::id.eq(plan.id)))
            .set((
                groups::name.eq(plan.name),
                groups::slug.eq(plan.slug),
                groups::description.eq(plan.description),
                groups::group_metadata.eq(plan.group_metadata),
                groups::price.eq(plan.price),
                groups::currency.eq(plan.currency),
                groups::billing_cycle.eq(plan.billing_cycle),
                groups::is_active.eq(plan.is_active.unwrap_or(true)),
                groups::is_promoted.eq(plan.is_promoted.unwrap_or(false)),
                groups::display_order.eq(plan.display_order),
                groups::updated_at.eq(diesel::dsl::now),
            ))
            .get_result::<DbPermissionGroup>(&mut conn)
            .await
    }

    /// Create a new permission group (database_types.rs compatibility layer)
    pub async fn create_group(&self, new_group: crate::infrastructure::adapters::repositories::database_types::NewPermissionGroup) -> Result<crate::infrastructure::adapters::repositories::database_types::PermissionGroup, diesel::result::Error> {
        use crate::schemas::primary::groups;
        use crate::infrastructure::adapters::repositories::database_types::PermissionGroup as DbPermissionGroup;

        let mut conn = self.db_pool.get().await
            .map_err(|e| diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string())
            ))?;

        diesel::insert_into(groups::table)
            .values(&new_group)
            .get_result::<DbPermissionGroup>(&mut conn)
            .await
    }
}

// Type alias for backward compatibility
pub type PermissionGroupRepositoryAdapter = GroupRepositoryAdapter;
