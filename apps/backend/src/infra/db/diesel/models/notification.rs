// ============================================================================
// SIMPLE NOTIFICATION MODELS - REPLACING COMPLEX NOTIFICATION SYSTEM
// ============================================================================
// This file replaces complex notification models with simple ones using actual schema fields
// Works with the simple role system from auth/roles.rs

use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;

use crate::infra::db::diesel::schema::notifications;

// ============================================================================
// SIMPLE NOTIFICATION MODEL (USING ONLY EXISTING FIELDS)
// ============================================================================

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = notifications)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub priority: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<JsonValue>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = notifications)]
pub struct NewDieselNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub priority: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<JsonValue>,
}

// ============================================================================
// SIMPLE NOTIFICATION TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
    FeatureAccess,
    RoleChange,
    SystemUpdate,
}

impl std::fmt::Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Info => write!(f, "info"),
            NotificationType::Warning => write!(f, "warning"),
            NotificationType::Error => write!(f, "error"),
            NotificationType::Success => write!(f, "success"),
            NotificationType::FeatureAccess => write!(f, "feature_access"),
            NotificationType::RoleChange => write!(f, "role_change"),
            NotificationType::SystemUpdate => write!(f, "system_update"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for NotificationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationPriority::Low => write!(f, "low"),
            NotificationPriority::Medium => write!(f, "medium"),
            NotificationPriority::High => write!(f, "high"),
            NotificationPriority::Critical => write!(f, "critical"),
        }
    }
}

// ============================================================================
// SIMPLE NOTIFICATION HELPER FUNCTIONS
// ============================================================================

impl NewDieselNotification {
    pub fn new(
        user_id: Uuid,
        title: &str,
        message: &str,
        notification_type: NotificationType,
        priority: NotificationPriority,
        expires_at: Option<DateTime<Utc>>,
        metadata: Option<JsonValue>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            title: title.to_string(),
            message: message.to_string(),
            notification_type: notification_type.to_string(),
            priority: priority.to_string(),
            is_read: false,
            created_at: Utc::now(),
            expires_at,
            metadata,
        }
    }
    
    pub fn info_notification(user_id: Uuid, title: &str, message: &str) -> Self {
        Self::new(
            user_id,
            title,
            message,
            NotificationType::Info,
            NotificationPriority::Low,
            None,
            None,
        )
    }
    
    pub fn warning_notification(user_id: Uuid, title: &str, message: &str) -> Self {
        Self::new(
            user_id,
            title,
            message,
            NotificationType::Warning,
            NotificationPriority::Medium,
            None,
            None,
        )
    }
    
    pub fn error_notification(user_id: Uuid, title: &str, message: &str) -> Self {
        Self::new(
            user_id,
            title,
            message,
            NotificationType::Error,
            NotificationPriority::High,
            None,
            None,
        )
    }
    
    pub fn success_notification(user_id: Uuid, title: &str, message: &str) -> Self {
        Self::new(
            user_id,
            title,
            message,
            NotificationType::Success,
            NotificationPriority::Low,
            None,
            None,
        )
    }
    
    pub fn role_change_notification(user_id: Uuid, new_role: &str) -> Self {
        Self::new(
            user_id,
            "Role Changed",
            &format!("Your role has been changed to: {}", new_role),
            NotificationType::RoleChange,
            NotificationPriority::High,
            None,
            Some(serde_json::json!({ "new_role": new_role })),
        )
    }
}

// ============================================================================
// DATABASE OPERATIONS (SIMPLE)
// ============================================================================

pub fn create_notification(
    conn: &mut diesel::pg::PgConnection,
    notification: NewDieselNotification,
) -> Result<DieselNotification, diesel::result::Error> {
    diesel::insert_into(notifications::table)
        .values(&notification)
        .get_result(conn)
}

pub fn get_user_notifications(
    conn: &mut diesel::pg::PgConnection,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<DieselNotification>, diesel::result::Error> {
    notifications::table
        .filter(notifications::user_id.eq(user_id))
        .order(notifications::created_at.desc())
        .limit(limit)
        .load(conn)
}

pub fn mark_notification_read(
    conn: &mut diesel::pg::PgConnection,
    notification_id: Uuid,
) -> Result<DieselNotification, diesel::result::Error> {
    diesel::update(notifications::table.filter(notifications::id.eq(notification_id)))
        .set(notifications::is_read.eq(true))
        .get_result(conn)
}

pub fn get_unread_count(
    conn: &mut diesel::pg::PgConnection,
    user_id: Uuid,
) -> Result<i64, diesel::result::Error> {
    notifications::table
        .filter(notifications::user_id.eq(user_id))
        .filter(notifications::is_read.eq(false))
        .count()
        .get_result(conn)
}