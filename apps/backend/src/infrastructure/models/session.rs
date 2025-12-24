//! Diesel Models for Sessions
//!
//! Database models for sessions table using Diesel ORM

use chrono::{DateTime, Utc};
use diesel::QueryableByName;
use uuid::Uuid;

/// Diesel Queryable model for sessions table
/// Note: We use raw SQL with ip_address::TEXT casting to handle INET->String conversion
#[derive(Debug, Clone, QueryableByName)]
pub struct SessionDb {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub user_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub access_token: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub expires_at: DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub provider: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub session_token: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub user_agent: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub ip_address: Option<String>, // Cast from INET to TEXT in SQL queries
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: DateTime<Utc>,
}

/// Model for creating new sessions (used with raw SQL, not Diesel DSL)
#[derive(Debug, Clone)]
pub struct NewSessionDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub provider: Option<String>,
    pub session_token: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>, // Using String for IP, convert to/from IpAddr in adapter
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}