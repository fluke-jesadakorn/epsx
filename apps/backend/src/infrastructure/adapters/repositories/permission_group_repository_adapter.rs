// Permission Group Repository Adapter (Infrastructure Layer)
// Implements PermissionGroupRepositoryPort using SQLx and PostgreSQL

use crate::prelude::*;
use sqlx::{PgPool, Row};
use tracing::{error, info};

use crate::domain::permission_management::{
    PermissionGroup, GroupId, GroupSlug, PermissionString,
    repository_ports::{PermissionGroupRepositoryPort, GroupSearchCriteria, GroupStatistics},
    aggregates::permission_group::LoadPermissionGroupParams,
};
use std::collections::HashSet;

/// PostgreSQL implementation of PermissionGroupRepositoryPort
#[derive(Clone)]
pub struct PermissionGroupRepositoryAdapter {
    db_pool: Arc<PgPool>,
}

impl PermissionGroupRepositoryAdapter {
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        Self { db_pool }
    }
}

#[async_trait]
impl PermissionGroupRepositoryPort for PermissionGroupRepositoryAdapter {
    async fn find_by_id(&self, id: &GroupId) -> AppResult<Option<PermissionGroup>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, name, slug, description, group_type, price, currency, billing_cycle,
                is_active, is_promoted, display_order, max_members, auto_assign_enabled,
                group_metadata, created_at, updated_at
            FROM permission_groups
            WHERE id = $1
            "#
        )
        .bind(id.value())
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to find permission group by id {}: {}", id, e);
            AppError::database_error(e.to_string())
        })?;

        if let Some(row) = row {
            // Get permissions for this group
            let permission_rows = sqlx::query(
                r#"
                SELECT p.platform, p.resource, p.action
                FROM permission_group_memberships pgm
                JOIN permissions p ON pgm.permission_id = p.id
                WHERE pgm.group_id = $1
                "#
            )
            .bind(id.value())
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to fetch permissions for group {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;

            let permissions: HashSet<PermissionString> = permission_rows
                .iter()
                .filter_map(|r| {
                    let platform: String = r.get("platform");
                    let resource: String = r.get("resource");
                    let action: String = r.get("action");
                    let perm_str = format!("{}:{}:{}", platform, resource, action);
                    PermissionString::new(perm_str).ok()
                })
                .collect();

            let group_id = GroupId::from_uuid(row.get("id"));
            let slug = GroupSlug::new(row.get::<String, _>("slug"))
                .map_err(|e| AppError::validation_error(e.to_string()))?;

            let group = PermissionGroup::load(LoadPermissionGroupParams {
                id: group_id,
                name: row.get("name"),
                slug,
                description: row.get("description"),
                group_type: row.get("group_type"),
                permissions,
                price: row.get::<f64, _>("price"),
                currency: row.get("currency"),
                billing_cycle: row.get("billing_cycle"),
                is_active: row.get("is_active"),
                is_promoted: row.get("is_promoted"),
                display_order: row.get("display_order"),
                max_members: row.get("max_members"),
                auto_assign_enabled: row.get("auto_assign_enabled"),
                metadata: row.get("group_metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                version: 1,
            });

            Ok(Some(group))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug(&self, slug: &GroupSlug) -> AppResult<Option<PermissionGroup>> {
        let row = sqlx::query(
            r#"
            SELECT id FROM permission_groups WHERE slug = $1
            "#
        )
        .bind(slug.as_str())
        .fetch_optional(self.db_pool.as_ref())
        .await
        .map_err(|e| {
            error!("Failed to find permission group by slug {}: {}", slug, e);
            AppError::database_error(e.to_string())
        })?;

        if let Some(row) = row {
            let group_id = GroupId::from_uuid(row.get("id"));
            self.find_by_id(&group_id).await
        } else {
            Ok(None)
        }
    }

    async fn find_all(&self, criteria: GroupSearchCriteria) -> AppResult<Vec<PermissionGroup>> {
        let mut query = String::from(
            r#"
            SELECT id FROM permission_groups WHERE 1=1
            "#
        );

        if let Some(group_type) = &criteria.group_type {
            query.push_str(&format!(" AND group_type = '{}'", group_type));
        }

        if let Some(is_active) = criteria.is_active {
            query.push_str(&format!(" AND is_active = {}", is_active));
        }

        if let Some(is_promoted) = criteria.is_promoted {
            query.push_str(&format!(" AND is_promoted = {}", is_promoted));
        }

        if let Some(search_term) = &criteria.search_term {
            query.push_str(&format!(" AND (name ILIKE '%{}%' OR description ILIKE '%{}%')", search_term, search_term));
        }

        query.push_str(" ORDER BY display_order ASC, created_at DESC");

        if let Some(limit) = criteria.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = criteria.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let rows = sqlx::query(&query)
            .fetch_all(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to find permission groups: {}", e);
                AppError::database_error(e.to_string())
            })?;

        let mut groups = Vec::new();
        for row in rows {
            let group_id = GroupId::from_uuid(row.get("id"));
            if let Some(group) = self.find_by_id(&group_id).await? {
                groups.push(group);
            }
        }

        Ok(groups)
    }

    async fn save(&self, group: &PermissionGroup) -> AppResult<()> {
        let mut tx = self.db_pool.begin().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Upsert permission group
        sqlx::query(
            r#"
            INSERT INTO permission_groups (
                id, name, slug, description, group_type, price, currency, billing_cycle,
                is_active, is_promoted, display_order, max_members, auto_assign_enabled,
                group_metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                price = EXCLUDED.price,
                currency = EXCLUDED.currency,
                billing_cycle = EXCLUDED.billing_cycle,
                is_active = EXCLUDED.is_active,
                is_promoted = EXCLUDED.is_promoted,
                display_order = EXCLUDED.display_order,
                max_members = EXCLUDED.max_members,
                auto_assign_enabled = EXCLUDED.auto_assign_enabled,
                group_metadata = EXCLUDED.group_metadata,
                updated_at = EXCLUDED.updated_at
            "#
        )
        .bind(group.id().value())
        .bind(group.name())
        .bind(group.slug().as_str())
        .bind(group.description())
        .bind(group.group_type())
        .bind(group.price())
        .bind(group.currency())
        .bind(group.billing_cycle())
        .bind(group.is_active())
        .bind(group.is_promoted())
        .bind(group.display_order())
        .bind(group.max_members())
        .bind(group.auto_assign_enabled())
        .bind(group.metadata())
        .bind(group.created_at())
        .bind(group.updated_at())
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("Failed to save permission group: {}", e);
            AppError::database_error(e.to_string())
        })?;

        // Delete existing permission associations
        sqlx::query(
            r#"
            DELETE FROM permission_group_memberships WHERE group_id = $1
            "#
        )
        .bind(group.id().value())
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        // Insert permission associations
        for permission in group.permissions() {
            // Get or create permission
            let parts: Vec<&str> = permission.as_str().split(':').collect();
            if parts.len() >= 3 {
                let perm_id: uuid::Uuid = sqlx::query_scalar(
                    r#"
                    INSERT INTO permissions (platform, resource, action)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (platform, resource, action) DO UPDATE
                    SET platform = EXCLUDED.platform
                    RETURNING id
                    "#
                )
                .bind(parts[0])
                .bind(parts[1])
                .bind(parts[2])
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?;

                // Link permission to group
                sqlx::query(
                    r#"
                    INSERT INTO permission_group_memberships (group_id, permission_id)
                    VALUES ($1, $2)
                    "#
                )
                .bind(group.id().value())
                .bind(perm_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?;
            }
        }

        tx.commit().await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        info!("Permission group {} saved successfully", group.id());
        Ok(())
    }

    async fn delete(&self, id: &GroupId) -> AppResult<()> {
        sqlx::query("DELETE FROM permission_groups WHERE id = $1")
            .bind(id.value())
            .execute(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to delete permission group {}: {}", id, e);
                AppError::database_error(e.to_string())
            })?;

        info!("Permission group {} deleted successfully", id);
        Ok(())
    }

    async fn count(&self, criteria: GroupSearchCriteria) -> AppResult<i64> {
        let mut query = String::from("SELECT COUNT(*) FROM permission_groups WHERE 1=1");

        if let Some(group_type) = &criteria.group_type {
            query.push_str(&format!(" AND group_type = '{}'", group_type));
        }

        if let Some(is_active) = criteria.is_active {
            query.push_str(&format!(" AND is_active = {}", is_active));
        }

        if let Some(is_promoted) = criteria.is_promoted {
            query.push_str(&format!(" AND is_promoted = {}", is_promoted));
        }

        if let Some(search_term) = &criteria.search_term {
            query.push_str(&format!(" AND (name ILIKE '%{}%' OR description ILIKE '%{}%')", search_term, search_term));
        }

        let count: i64 = sqlx::query_scalar(&query)
            .fetch_one(self.db_pool.as_ref())
            .await
            .map_err(|e| {
                error!("Failed to count permission groups: {}", e);
                AppError::database_error(e.to_string())
            })?;

        Ok(count)
    }

    async fn get_statistics(&self) -> AppResult<GroupStatistics> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_groups,
                COUNT(*) FILTER (WHERE is_active = true) as active_groups,
                COUNT(*) FILTER (WHERE is_promoted = true) as promoted_groups
            FROM permission_groups
            "#
        )
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        let total_members: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT wallet_address) FROM wallet_group_memberships"
        )
        .fetch_one(self.db_pool.as_ref())
        .await
        .unwrap_or(0);

        Ok(GroupStatistics {
            total_groups: row.get("total_groups"),
            active_groups: row.get("active_groups"),
            promoted_groups: row.get("promoted_groups"),
            total_members,
        })
    }

    async fn slug_exists(&self, slug: &GroupSlug) -> AppResult<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM permission_groups WHERE slug = $1)"
        )
        .bind(slug.as_str())
        .fetch_one(self.db_pool.as_ref())
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(exists)
    }
}
