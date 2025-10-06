// Database Types and Models
// Unified type definitions for database operations

use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;
use sqlx::PgPool;

// Database Pool Types
pub type DbPool = PgPool;

// Session Types
#[derive(Debug, Clone)]
pub struct SessionRepository {
    _pool: Arc<PgPool>,
}

impl SessionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
    
    pub async fn save_session(&self, _session_data: &str) -> Result<(), String> {
        Ok(())
    }
    
    pub async fn save(&self, _session: &crate::domain::wallet_management::aggregates::session::Session) -> Result<(), String> {
        // TODO: Implement session storage
        Ok(())
    }
}

// User Repository Types
#[derive(Debug, Clone)]
pub struct UserRepository {
    _pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

// Token Types
#[derive(Debug, Clone)]
pub struct RefreshTokenRepository {
    _pool: Arc<PgPool>,
}

impl RefreshTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

#[derive(Debug, Clone)]
pub struct RevokedTokenRepository {
    _pool: Arc<PgPool>,
}

impl RevokedTokenRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

#[derive(Debug, Clone)]
pub struct RefreshToken {
    pub id: Uuid,
    pub token: String,
    pub wallet_address: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewRefreshToken {
    pub token: String,
    pub wallet_address: Uuid,
    pub expires_at: DateTime<Utc>,
}

// User limits types have been moved to domain/shared_kernel/value_objects/user_limits.rs
// for proper clean architecture separation

// Notification Types
#[derive(Debug, Clone)]
pub struct NotificationRepositoryAdapter {
    _pool: Arc<PgPool>,
}

impl NotificationRepositoryAdapter {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
    
    pub async fn deliver_notification_to_topic(
        &self,
        _topic: &str,
        _title: &str,
        _body: &str,
        _data: Option<serde_json::Value>,
    ) -> Result<crate::domain::notification::aggregates::notification::DeliveryResult, crate::application::ApplicationError> {
        // TODO: Implement topic notification delivery
        Ok(crate::domain::notification::aggregates::notification::DeliveryResult::Success {
            message_id: Some("placeholder_message_id".to_string()),
            delivered_at: chrono::Utc::now(),
        })
    }

    pub async fn deliver_notification_to_user(
        &self,
        _notification: &crate::domain::notification::aggregates::notification::Notification,
        _wallet_address: uuid::Uuid,
        _fcm_token: Option<String>,
        _email: Option<String>,
    ) -> Result<Vec<crate::domain::notification::aggregates::notification::DeliveryResult>, crate::application::ApplicationError> {
        // TODO: Implement user notification delivery
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
pub struct UserNotificationRepository {
    _pool: Arc<PgPool>,
}

impl UserNotificationRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { _pool: pool }
    }
}

#[derive(Debug, Clone)]
pub struct NotificationMapper;

impl Default for NotificationMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationMapper {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_ddd_notification_from_legacy(
        _recipient_wallet_address: Option<Uuid>,
        _fcm_topic_id: Option<String>,
        _title: String,
        _body: String,
        _notification_type: crate::domain::notification::value_objects::user_preferences::NotificationType,
        _priority: crate::domain::notification::aggregates::notification::NotificationPriority,
        _channels: Vec<String>,
        _scheduled_for: Option<chrono::DateTime<chrono::Utc>>,
        _expires_at: Option<chrono::DateTime<chrono::Utc>>,
        _action_url: Option<String>,
        _image_url: Option<String>,
        _data_payload: Option<serde_json::Value>,
    ) -> Result<crate::domain::notification::aggregates::notification::Notification, String> {
        // TODO: Implement actual DDD notification creation 
        // For now, return an error since we don't have the full implementation
        Err("NotificationMapper::create_ddd_notification_from_legacy not yet implemented".to_string())
    }
}

// User response types for API compatibility (Web3-first: wallet-based)
#[derive(Debug, Clone)]
pub struct UserUpdateResponse {
    pub wallet_address: String,
    pub is_active: bool,
    pub permissions: Vec<String>,
}

impl UserUpdateResponse {
    pub fn placeholder(wallet_address: String) -> Self {
        Self {
            wallet_address,
            is_active: true,
            permissions: vec!["epsx:basic:access".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserCreateResponse {
    pub wallet_address: String,
}

impl UserCreateResponse {
    pub fn new(wallet_address: String) -> Self {
        Self { wallet_address }
    }
}

// Pool creation function
pub async fn create_pool() -> Result<Arc<PgPool>, Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable is required")?;
    let pool = PgPool::connect(&database_url).await?;
    Ok(Arc::new(pool))
}

// Database model types for mappers compatibility
// Legacy User/NewUser/UpdateUser structs removed - Web3-first uses WalletUser only

#[derive(Debug, Clone)]
pub struct Session {
    pub id: uuid::Uuid,
    pub wallet_address: uuid::Uuid,
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct NewSession {
    pub id: uuid::Uuid,
    pub wallet_address: uuid::Uuid,
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateSession {
    pub access_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct IpAddr(pub String);

// Permission Group Types - Updated to match database schema exactly
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PermissionGroup {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub permissions: serde_json::Value,
    pub group_metadata: serde_json::Value,
    pub price: Option<sqlx::types::BigDecimal>, // Handle nullable decimal
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub assignment_rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewPermissionGroup {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub permissions: serde_json::Value,
    pub group_metadata: serde_json::Value,
    pub price: Option<sqlx::types::BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub display_order: Option<i32>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdatePermissionGroup {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<serde_json::Value>,
    pub price: Option<sqlx::types::BigDecimal>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub last_modified_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PermissionGroupRepository {
    pool: Arc<PgPool>,
}

impl PermissionGroupRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Get all active subscription plans
    pub async fn get_subscription_plans(&self) -> Result<Vec<PermissionGroup>, sqlx::Error> {
        let plans = sqlx::query_as::<_, PermissionGroup>(
            r#"
            SELECT 
                id, name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order,
                max_members, auto_assign_enabled, assignment_rules, created_at, updated_at,
                created_by, last_modified_by
            FROM permission_groups 
            WHERE group_type = 'subscription' AND COALESCE(is_active, true) = true 
            ORDER BY COALESCE(display_order, 0), COALESCE(price, 0)
            "#
        )
        .fetch_all(&*self.pool)
        .await?;
        
        Ok(plans)
    }

    /// Get plan by ID
    pub async fn get_plan_by_id(&self, plan_id: Uuid) -> Result<Option<PermissionGroup>, sqlx::Error> {
        let plan = sqlx::query_as::<_, PermissionGroup>(
            r#"
            SELECT 
                id, name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order,
                max_members, auto_assign_enabled, assignment_rules, created_at, updated_at,
                created_by, last_modified_by
            FROM permission_groups 
            WHERE id = $1 AND group_type = 'subscription'
            "#
        )
        .bind(plan_id)
        .fetch_optional(&*self.pool)
        .await?;
        
        Ok(plan)
    }

    /// Get all permission groups
    pub async fn get_all_groups(&self) -> Result<Vec<PermissionGroup>, sqlx::Error> {
        let groups = sqlx::query_as::<_, PermissionGroup>(
            r#"
            SELECT 
                id, name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order,
                max_members, auto_assign_enabled, assignment_rules, created_at, updated_at,
                created_by, last_modified_by
            FROM permission_groups 
            WHERE COALESCE(is_active, true) = true 
            ORDER BY COALESCE(display_order, 0), name
            "#
        )
        .fetch_all(&*self.pool)
        .await?;
        
        Ok(groups)
    }

    /// Get permission group by ID
    pub async fn get_group_by_id(&self, group_id: Uuid) -> Result<Option<PermissionGroup>, sqlx::Error> {
        let group = sqlx::query_as::<_, PermissionGroup>(
            r#"
            SELECT 
                id, name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order,
                max_members, auto_assign_enabled, assignment_rules, created_at, updated_at,
                created_by, last_modified_by
            FROM permission_groups 
            WHERE id = $1
            "#
        )
        .bind(group_id)
        .fetch_optional(&*self.pool)
        .await?;
        
        Ok(group)
    }

    /// Create a new permission group
    pub async fn create_group(&self, new_group: NewPermissionGroup) -> Result<PermissionGroup, sqlx::Error> {
        let group = sqlx::query_as::<_, PermissionGroup>(
            r#"
            INSERT INTO permission_groups (
                name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, display_order, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING 
                id, name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order,
                max_members, auto_assign_enabled, assignment_rules, created_at, updated_at,
                created_by, last_modified_by
            "#
        )
        .bind(&new_group.name)
        .bind(&new_group.slug)
        .bind(&new_group.description)
        .bind(&new_group.group_type)
        .bind(&new_group.permissions)
        .bind(&new_group.group_metadata)
        .bind(new_group.price)
        .bind(new_group.currency)
        .bind(new_group.billing_cycle)
        .bind(new_group.is_active.unwrap_or(true))
        .bind(new_group.display_order.unwrap_or(0))
        .bind(new_group.created_by)
        .fetch_one(&*self.pool)
        .await?;
        
        Ok(group)
    }

    /// Update an existing permission group
    pub async fn update_group(&self, group_id: Uuid, update_group: UpdatePermissionGroup) -> Result<Option<PermissionGroup>, sqlx::Error> {
        let group = sqlx::query_as::<_, PermissionGroup>(
            r#"
            UPDATE permission_groups SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                permissions = COALESCE($4, permissions),
                price = COALESCE($5, price),
                currency = COALESCE($6, currency),
                billing_cycle = COALESCE($7, billing_cycle),
                is_active = COALESCE($8, is_active),
                last_modified_by = COALESCE($9, last_modified_by),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1
            RETURNING
                id, name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order,
                max_members, auto_assign_enabled, assignment_rules, created_at, updated_at,
                created_by, last_modified_by
            "#
        )
        .bind(group_id)
        .bind(update_group.name)
        .bind(update_group.description)
        .bind(update_group.permissions)
        .bind(update_group.price)
        .bind(update_group.currency)
        .bind(update_group.billing_cycle)
        .bind(update_group.is_active)
        .bind(update_group.last_modified_by)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(group)
    }

    /// Update plan (alias for update_group, used specifically for subscription plans)
    pub async fn update_plan(&self, plan: PermissionGroup) -> Result<PermissionGroup, sqlx::Error> {
        let updated_plan = sqlx::query_as::<_, PermissionGroup>(
            r#"
            UPDATE permission_groups SET
                name = $2,
                description = $3,
                permissions = $4,
                group_metadata = $5,
                price = $6,
                currency = $7,
                billing_cycle = $8,
                is_active = $9,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND group_type = 'subscription'
            RETURNING
                id, name, slug, description, group_type, permissions, group_metadata,
                price, currency, billing_cycle, is_active, is_promoted, display_order,
                max_members, auto_assign_enabled, assignment_rules, created_at, updated_at,
                created_by, last_modified_by
            "#
        )
        .bind(plan.id)
        .bind(plan.name)
        .bind(plan.description)
        .bind(plan.permissions)
        .bind(plan.group_metadata)
        .bind(plan.price)
        .bind(plan.currency)
        .bind(plan.billing_cycle)
        .bind(plan.is_active)
        .fetch_one(&*self.pool)
        .await?;

        Ok(updated_plan)
    }

    /// Delete a permission group (soft delete by setting is_active = false)
    pub async fn delete_group(&self, group_id: Uuid) -> Result<bool, sqlx::Error> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE permission_groups 
            SET is_active = false, updated_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND is_active = true
            "#
        )
        .bind(group_id)
        .execute(&*self.pool)
        .await?
        .rows_affected();
        
        Ok(rows_affected > 0)
    }

    /// Assign wallet to group using database function
    pub async fn assign_wallet_to_group(
        &self,
        wallet_address: &str,
        group_id: Uuid,
        assigned_by: Option<&str>,
        assignment_reason: Option<&str>,
        expires_at: Option<chrono::DateTime<Utc>>
    ) -> Result<Uuid, sqlx::Error> {
        let membership_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT assign_wallet_to_group($1, $2, $3, 'manual', $4, $5)
            "#
        )
        .bind(wallet_address)
        .bind(group_id)
        .bind(assigned_by)
        .bind(assignment_reason)
        .bind(expires_at)
        .fetch_one(&*self.pool)
        .await?;
        
        Ok(membership_id)
    }

    /// Get wallet assignments using database query
    pub async fn get_wallet_assignments(&self, wallet_address: &str) -> Result<Vec<WalletAssignment>, sqlx::Error> {
        let assignments = sqlx::query_as::<_, WalletAssignment>(
            r#"
            SELECT 
                wgm.id, wgm.wallet_address, wgm.group_id, 
                pg.name as group_name, pg.group_type,
                wgm.assignment_source, wgm.assignment_reason,
                wgm.assigned_by, wgm.assigned_at, wgm.expires_at,
                wgm.is_active
            FROM wallet_group_memberships wgm
            JOIN permission_groups pg ON wgm.group_id = pg.id
            WHERE wgm.wallet_address = $1 
                AND wgm.is_active = true
                AND (wgm.expires_at IS NULL OR wgm.expires_at > CURRENT_TIMESTAMP)
            ORDER BY wgm.assigned_at DESC
            "#
        )
        .bind(wallet_address)
        .fetch_all(&*self.pool)
        .await?;
        
        Ok(assignments)
    }

    /// Check if wallet has permission using database function
    pub async fn wallet_has_permission(&self, wallet_address: &str, permission: &str) -> Result<bool, sqlx::Error> {
        let has_permission = sqlx::query_scalar::<_, bool>(
            "SELECT wallet_has_permission($1, $2)"
        )
        .bind(wallet_address)
        .bind(permission)
        .fetch_one(&*self.pool)
        .await?;
        
        Ok(has_permission)
    }

    /// Get wallet effective permissions using database function
    pub async fn get_wallet_effective_permissions(&self, wallet_address: &str) -> Result<Vec<String>, sqlx::Error> {
        let permissions_json = sqlx::query_scalar::<_, serde_json::Value>(
            "SELECT get_wallet_effective_permissions($1)"
        )
        .bind(wallet_address)
        .fetch_one(&*self.pool)
        .await?;
        
        // Convert JSONB array to Vec<String>
        let permissions: Vec<String> = serde_json::from_value(permissions_json)
            .unwrap_or_default();
        
        Ok(permissions)
    }
}

/// Wallet Assignment structure for database queries
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct WalletAssignment {
    pub id: Uuid,
    pub wallet_address: String,
    pub group_id: Uuid,
    pub group_name: String,
    pub group_type: String,
    pub assignment_source: String,
    pub assignment_reason: Option<String>,
    pub assigned_by: Option<String>,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}