use diesel::prelude::*;
use chrono::{DateTime, Utc};
use uuid::Uuid;


use serde::{Serialize, Deserialize};


use crate::infra::db::diesel::schema::users;


// ============================================================================
// USER MODELS - PERMISSION-ONLY SYSTEM
// ============================================================================
// These models now use only permissions (no role field) for access control
// Permissions use format: "platform:resource:action" (e.g., "epsx:analytics:view")

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DieselUser {
    pub id: Uuid,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: Option<bool>,
    pub is_active: Option<bool>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub primary_platform_id: Option<Uuid>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = users)]
pub struct NewDieselUser {
    pub id: Uuid,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: Option<bool>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub primary_platform_id: Option<Uuid>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = users)]
pub struct UpdateDieselUser {
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: Option<bool>,
    pub is_active: Option<bool>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}