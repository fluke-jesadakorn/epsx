use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::infra::db::diesel::schema::users;
use crate::auth::roles::UserRoleEnum as DieselUserRole;

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
    pub is_active: Option<bool>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: DieselUserRole, // Simple role instead of complex permissions
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
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: DieselUserRole, // Simple role instead of complex permissions
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = users)]
pub struct UpdateDieselUser {
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub role: Option<DieselUserRole>, // Simple role instead of complex permissions
}