// ============================================================================
// SIMPLE ROLE SYSTEM - REPLACING ALL COMPLEX PERMISSION SYSTEMS
// ============================================================================
// This file replaces 2000+ lines of complex permission code with simple role-based access
// Roles: admin, user, guest
// Features: view_eps, export_data, realtime, profile, notifications, billing, advanced_filters

use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::dom::entities::user::UserRole;
use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// ============================================================================
// SIMPLE ROLE TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
    Guest,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::User => write!(f, "user"),
            Role::Guest => write!(f, "guest"),
        }
    }
}

impl std::str::FromStr for Role {
    type Err = &'static str;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Role::Admin),
            "user" => Ok(Role::User),
            "guest" => Ok(Role::Guest),
            _ => Err("Invalid role"),
        }
    }
}

// ============================================================================
// SIMPLE USER CLAIMS (REPLACES COMPLEX JWT GENERATION)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleUserClaims {
    pub firebase_uid: String,
    pub email: String,
    pub role: Role,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================================
// SIMPLE FEATURE ACCESS LOGIC (MATCHES DATABASE FUNCTIONS)
// ============================================================================

pub fn check_feature_access(user_role: &Role, feature: &str) -> bool {
    match (user_role, feature) {
        // Admin can access everything
        (Role::Admin, _) => true,
        
        // User can access all premium features
        (Role::User, f) if matches!(f, 
            "view_eps" | "export_data" | "realtime" | "profile" | 
            "notifications" | "billing" | "advanced_filters") => true,
        
        // Guest can only view basic EPS data
        (Role::Guest, "view_eps") => true,
        
        // All other combinations are denied
        _ => false,
    }
}

pub fn check_role_access(user_role: &Role, required_role: &Role) -> bool {
    match (user_role, required_role) {
        // Admin can access everything
        (Role::Admin, _) => true,
        
        // User can access user and guest level
        (Role::User, Role::User | Role::Guest) => true,
        
        // Guest can only access guest level
        (Role::Guest, Role::Guest) => true,
        
        // All other combinations are denied
        _ => false,
    }
}

pub fn get_role_features(role: &Role) -> Vec<String> {
    let all_features = [
        "view_eps", 
        "export_data", 
        "realtime", 
        "profile", 
        "notifications", 
        "billing", 
        "advanced_filters"
    ];
    
    all_features
        .iter()
        .filter(|feature| check_feature_access(role, feature))
        .map(|s| s.to_string())
        .collect()
}

// ============================================================================
// DATABASE INTEGRATION WITH DIESEL
// ============================================================================

// Diesel enum for UserRole
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, diesel::deserialize::FromSqlRow, diesel::expression::AsExpression)]
#[diesel(sql_type = crate::infra::db::diesel::schema::sql_types::UserRole)]
pub enum UserRoleEnum {
    Admin,
    User,
    Guest,
}

// Conversion from old UserRole to new UserRoleEnum
impl From<UserRole> for UserRoleEnum {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::Admin => UserRoleEnum::Admin,
            UserRole::Moderator | UserRole::User => UserRoleEnum::User,
        }
    }
}

impl diesel::serialize::ToSql<crate::infra::db::diesel::schema::sql_types::UserRole, diesel::pg::Pg> for UserRoleEnum {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>) -> diesel::serialize::Result {
        use diesel::serialize::ToSql;
        let value = match *self {
            UserRoleEnum::Admin => "admin",
            UserRoleEnum::User => "user",
            UserRoleEnum::Guest => "guest",
        };
        <str as ToSql<diesel::sql_types::Text, diesel::pg::Pg>>::to_sql(value, out)
    }
}

impl diesel::deserialize::FromSql<crate::infra::db::diesel::schema::sql_types::UserRole, diesel::pg::Pg> for UserRoleEnum {
    fn from_sql(bytes: <diesel::pg::Pg as diesel::backend::Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match std::str::from_utf8(bytes.as_bytes())? {
            "admin" => Ok(UserRoleEnum::Admin),
            "user" => Ok(UserRoleEnum::User),
            "guest" => Ok(UserRoleEnum::Guest),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

// Convert between our Role enum and UserRoleEnum
impl From<UserRoleEnum> for Role {
    fn from(user_role: UserRoleEnum) -> Self {
        match user_role {
            UserRoleEnum::Admin => Role::Admin,
            UserRoleEnum::User => Role::User,
            UserRoleEnum::Guest => Role::Guest,
        }
    }
}

impl From<Role> for UserRoleEnum {
    fn from(role: Role) -> Self {
        match role {
            Role::Admin => UserRoleEnum::Admin,
            Role::User => UserRoleEnum::User,
            Role::Guest => UserRoleEnum::Guest,
        }
    }
}

// Simple user data from database
#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::infra::db::diesel::schema::users)]
pub struct SimpleUser {
    pub id: Uuid,
    pub firebase_uid: String,
    pub email: String,
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub role: UserRoleEnum,
}

pub fn get_user_by_firebase_uid(conn: &mut PgConnection, firebase_uid_param: &str) -> Result<Option<SimpleUser>, DieselError> {
    use crate::infra::db::diesel::schema::users::dsl::*;
    
    users
        .filter(firebase_uid.eq(firebase_uid_param))
        .filter(is_active.eq(Some(true)))
        .select(SimpleUser::as_select())
        .first(conn)
        .optional()
}

pub fn get_user_claims_from_db(conn: &mut PgConnection, firebase_uid_param: &str) -> Result<Option<SimpleUserClaims>, DieselError> {
    let user = get_user_by_firebase_uid(conn, firebase_uid_param)?;
    
    if let Some(user) = user {
        let role: Role = user.role.into();
        
        Ok(Some(SimpleUserClaims {
            firebase_uid: user.firebase_uid,
            email: user.email,
            role,
            display_name: user.display_name,
            name: user.name,
            avatar_url: user.avatar_url,
            is_active: user.is_active.unwrap_or(false),
            last_login_at: user.last_login_at,
        }))
    } else {
        Ok(None)
    }
}

pub fn check_user_feature_access_db(
    conn: &mut PgConnection,
    firebase_uid: &str, 
    feature: &str
) -> Result<bool, DieselError> {
    let user = get_user_by_firebase_uid(conn, firebase_uid)?;
    
    if let Some(user) = user {
        let role: Role = user.role.into();
        Ok(check_feature_access(&role, feature))
    } else {
        Ok(false)
    }
}

pub fn check_user_role_access_db(
    conn: &mut PgConnection,
    firebase_uid: &str,
    required_role: &str
) -> Result<bool, DieselError> {
    let user = get_user_by_firebase_uid(conn, firebase_uid)?;
    
    if let Some(user) = user {
        let user_role: Role = user.role.into();
        let required = required_role.parse::<Role>().map_err(|_| DieselError::DeserializationError(
            "Invalid required role format".into()  
        ))?;
        Ok(check_role_access(&user_role, &required))
    } else {
        Ok(false)
    }
}

// ============================================================================
// ROLE VALIDATION HELPERS
// ============================================================================

pub fn is_admin(role: &Role) -> bool {
    matches!(role, Role::Admin)
}

pub fn is_user_or_admin(role: &Role) -> bool {
    matches!(role, Role::Admin | Role::User)
}

pub fn can_view_eps(role: &Role) -> bool {
    matches!(role, Role::Admin | Role::User | Role::Guest)
}

pub fn can_export_data(role: &Role) -> bool {
    matches!(role, Role::Admin | Role::User)
}

pub fn can_access_realtime(role: &Role) -> bool {
    matches!(role, Role::Admin | Role::User)
}

pub fn can_manage_profile(role: &Role) -> bool {
    matches!(role, Role::Admin | Role::User)
}

pub fn can_receive_notifications(role: &Role) -> bool {
    matches!(role, Role::Admin | Role::User)
}

pub fn can_manage_billing(role: &Role) -> bool {
    matches!(role, Role::Admin | Role::User)
}

pub fn can_use_advanced_filters(role: &Role) -> bool {
    matches!(role, Role::Admin | Role::User)
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum RoleError {
    #[error("Access denied: insufficient role")]
    InsufficientRole,
    
    #[error("Access denied: feature not available for role")]
    FeatureNotAvailable,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Database error: {0}")]
    Database(#[from] DieselError),
    
    #[error("Invalid role format")]
    InvalidRole,
}

// ============================================================================
// MIDDLEWARE HELPERS (FOR AXUM INTEGRATION WITH DIESEL)
// ============================================================================

pub fn require_role_sync(
    conn: &mut PgConnection,
    firebase_uid: &str,
    required_role: Role
) -> Result<SimpleUserClaims, RoleError> {
    // Get user claims
    let claims = get_user_claims_from_db(conn, firebase_uid)?
        .ok_or(RoleError::UserNotFound)?;
    
    // Check role access
    if check_role_access(&claims.role, &required_role) {
        Ok(claims)
    } else {
        Err(RoleError::InsufficientRole)
    }
}

pub fn require_feature_sync(
    conn: &mut PgConnection,
    firebase_uid: &str,
    feature: &str
) -> Result<SimpleUserClaims, RoleError> {
    // Get user claims
    let claims = get_user_claims_from_db(conn, firebase_uid)?
        .ok_or(RoleError::UserNotFound)?;
    
    // Check feature access
    if check_user_feature_access_db(conn, firebase_uid, feature)? {
        Ok(claims)
    } else {
        Err(RoleError::FeatureNotAvailable)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_hierarchy() {
        // Admin can access everything
        assert!(check_role_access(&Role::Admin, &Role::Admin));
        assert!(check_role_access(&Role::Admin, &Role::User));
        assert!(check_role_access(&Role::Admin, &Role::Guest));
        
        // User can access user and guest
        assert!(!check_role_access(&Role::User, &Role::Admin));
        assert!(check_role_access(&Role::User, &Role::User));
        assert!(check_role_access(&Role::User, &Role::Guest));
        
        // Guest can only access guest
        assert!(!check_role_access(&Role::Guest, &Role::Admin));
        assert!(!check_role_access(&Role::Guest, &Role::User));
        assert!(check_role_access(&Role::Guest, &Role::Guest));
    }

    #[test]
    fn test_feature_access() {
        // Admin can access everything
        assert!(check_feature_access(&Role::Admin, "view_eps"));
        assert!(check_feature_access(&Role::Admin, "export_data"));
        assert!(check_feature_access(&Role::Admin, "realtime"));
        assert!(check_feature_access(&Role::Admin, "profile"));
        assert!(check_feature_access(&Role::Admin, "notifications"));
        assert!(check_feature_access(&Role::Admin, "billing"));
        assert!(check_feature_access(&Role::Admin, "advanced_filters"));
        
        // User can access premium features
        assert!(check_feature_access(&Role::User, "view_eps"));
        assert!(check_feature_access(&Role::User, "export_data"));
        assert!(check_feature_access(&Role::User, "realtime"));
        assert!(check_feature_access(&Role::User, "profile"));
        assert!(check_feature_access(&Role::User, "notifications"));
        assert!(check_feature_access(&Role::User, "billing"));
        assert!(check_feature_access(&Role::User, "advanced_filters"));
        
        // Guest can only view EPS
        assert!(check_feature_access(&Role::Guest, "view_eps"));
        assert!(!check_feature_access(&Role::Guest, "export_data"));
        assert!(!check_feature_access(&Role::Guest, "realtime"));
        assert!(!check_feature_access(&Role::Guest, "profile"));
        assert!(!check_feature_access(&Role::Guest, "notifications"));
        assert!(!check_feature_access(&Role::Guest, "billing"));
        assert!(!check_feature_access(&Role::Guest, "advanced_filters"));
    }

    #[test]
    fn test_role_conversion() {
        assert_eq!(Role::Admin.to_string(), "admin");
        assert_eq!(Role::User.to_string(), "user");
        assert_eq!(Role::Guest.to_string(), "guest");
        
        assert_eq!("admin".parse::<Role>().unwrap(), Role::Admin);
        assert_eq!("user".parse::<Role>().unwrap(), Role::User);
        assert_eq!("guest".parse::<Role>().unwrap(), Role::Guest);
    }
}

// ============================================================================
// SECURITY EVENT STUB (FOR REPLACED SECURITY SYSTEM)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: Option<Uuid>,
    pub event_type: String,
    pub severity: String, 
    pub client_ip: Option<String>,
    pub user_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

impl Default for SecurityEvent {
    fn default() -> Self {
        Self {
            id: Some(Uuid::new_v4()),
            event_type: "UNKNOWN".to_string(),
            severity: "LOW".to_string(),
            client_ip: None,
            user_id: None,
            timestamp: Utc::now(),
            details: HashMap::new(),
        }
    }
}