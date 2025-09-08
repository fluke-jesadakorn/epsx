// V1 User Permission Status API
// Clean endpoints for users to check their current granular permissions

use axum::{
    extract::{Query, State},
    response::Json,
    http::StatusCode,
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::info;

use crate::auth::granular_permissions::{GranularPermissionClaim, PermissionSource};
use crate::web::middleware::clean_auth::AuthenticatedUser;
use crate::web::auth::AppState;

/// Query parameters for permission check
#[derive(Debug, Deserialize)]
pub struct PermissionCheckQuery {
    pub permission: Option<String>,
    pub include_expired: Option<bool>,
    pub format: Option<String>, // "detailed" or "simple"
}

/// User's current permission info
#[derive(Debug, Serialize)]
pub struct UserPermissionInfo {
    pub permission: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub source: String,
    pub granted_by: Option<String>,
    pub granted_at: DateTime<Utc>,
    pub is_active: bool,
    pub expires_soon: Option<bool>, // true if expires within 24 hours
    pub time_until_expiry: Option<i64>, // seconds until expiry
    pub metadata: Option<HashMap<String, String>>,
}

/// User's complete permission status
#[derive(Debug, Serialize)]
pub struct UserPermissionStatus {
    pub user_id: String,
    pub permissions: Vec<UserPermissionInfo>,
    pub permission_version: u32,
    pub last_updated: DateTime<Utc>,
    pub total_permissions: usize,
    pub active_permissions: usize,
    pub expired_permissions: usize,
    pub expiring_soon: usize,
    pub has_admin_access: bool,
    pub platform_permissions: HashMap<String, Vec<String>>, // platform -> permissions
}

/// Simple permission check result
#[derive(Debug, Serialize)]
pub struct SimplePermissionCheck {
    pub user_id: String,
    pub permission: String,
    pub has_permission: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub expires_soon: bool,
    pub checked_at: DateTime<Utc>,
}

/// Standard API response
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.to_string(),
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: &str) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message: message.to_string(),
            timestamp: Utc::now(),
        }
    }
}

/// Get user's complete permission status
pub async fn get_user_permissions(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<PermissionCheckQuery>,
) -> Result<Json<ApiResponse<UserPermissionStatus>>, (StatusCode, Json<ErrorResponse>)> {
    
    info!("Getting permission status for user {}", user.user_id);
    
    // Skip cache for now - complex Arc to Box conversion needed  
    // TODO: Implement proper cache integration with DDD approach
    let cached_permissions: Option<Vec<String>> = None;
    
    // Since cache is disabled, use JWT permissions directly
    let (permissions_map, permission_version) = {
            // Fall back to JWT permissions (convert to expected format)
            let mut permissions_map = HashMap::new();
            for perm in &user.valid_permissions {
                // Create a basic permission claim for JWT permissions
                permissions_map.insert(
                    perm.clone(),
                    GranularPermissionClaim::permanent(
                        PermissionSource::SystemGrant,
                        None,
                    )
                );
            }
            (permissions_map, user.permission_version)
    };
    
    let now = Utc::now();
    let include_expired = query.include_expired.unwrap_or(false);
    
    // Convert to user permission info
    let mut permission_infos = Vec::new();
    let mut active_count = 0;
    let mut expired_count = 0;
    let mut expiring_soon_count = 0;
    
    for (permission, claim) in &permissions_map {
        let is_expired = claim.expires_at.map_or(false, |exp| {
            if let Some(exp_dt) = DateTime::from_timestamp(exp, 0) {
                exp_dt < now
            } else {
                false
            }
        });
        let is_active = !is_expired;
        let expires_soon = claim.expires_at.map_or(false, |exp| {
            if let Some(exp_dt) = DateTime::from_timestamp(exp, 0) {
                (exp_dt - now).num_hours() <= 24
            } else {
                false
            }
        });
        
        if is_expired {
            expired_count += 1;
            if !include_expired {
                continue;
            }
        } else {
            active_count += 1;
            if expires_soon {
                expiring_soon_count += 1;
            }
        }
        
        permission_infos.push(UserPermissionInfo {
            permission: permission.clone(),
            expires_at: claim.expires_at.and_then(|ts| DateTime::from_timestamp(ts, 0)),
            source: format!("{:?}", claim.source),
            granted_by: claim.granted_by.clone(),
            granted_at: DateTime::from_timestamp(claim.granted_at, 0)
                .unwrap_or_else(|| Utc::now())
                .into(),
            is_active,
            expires_soon: Some(expires_soon),
            time_until_expiry: claim.expires_at.and_then(|exp| {
                if let Some(exp_dt) = DateTime::from_timestamp(exp, 0) {
                    Some((exp_dt - now).num_seconds().max(0))
                } else {
                    None
                }
            }),
            metadata: claim.metadata.clone(),
        });
    }
    
    // Sort by expiry date (expiring soonest first)
    permission_infos.sort_by(|a, b| {
        match (a.expires_at, b.expires_at) {
            (Some(a_exp), Some(b_exp)) => a_exp.cmp(&b_exp),
            (Some(_), None) => std::cmp::Ordering::Less, // Expiring permissions first
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.permission.cmp(&b.permission), // Alphabetical for permanent
        }
    });
    
    // Group permissions by platform
    let mut platform_permissions: HashMap<String, Vec<String>> = HashMap::new();
    for info in &permission_infos {
        if info.is_active {
            let platform = if info.permission.starts_with("admin:") {
                "admin"
            } else if info.permission.starts_with("epsx-pay:") {
                "epsx-pay"
            } else if info.permission.starts_with("epsx-token:") {
                "epsx-token"
            } else {
                "epsx"
            };
            
            platform_permissions
                .entry(platform.to_string())
                .or_insert_with(Vec::new)
                .push(info.permission.clone());
        }
    }
    
    // Check if user has admin access
    let has_admin_access = user.valid_permissions.iter()
        .any(|p| p.starts_with("admin:") || p == "admin:*:*");
    
    let status = UserPermissionStatus {
        user_id: user.user_id.clone(),
        permissions: permission_infos,
        permission_version,
        last_updated: now,
        total_permissions: permissions_map.len(),
        active_permissions: active_count,
        expired_permissions: expired_count,
        expiring_soon: expiring_soon_count,
        has_admin_access,
        platform_permissions,
    };
    
    Ok(Json(ApiResponse::success(
        status,
        "User permissions retrieved successfully"
    )))
}

/// Check if user has a specific permission
pub async fn check_user_permission(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Query(query): Query<PermissionCheckQuery>,
) -> Result<Json<ApiResponse<SimplePermissionCheck>>, (StatusCode, Json<ErrorResponse>)> {
    
    let permission = query.permission.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "missing_parameter".to_string(),
                message: "Permission parameter is required".to_string(),
                timestamp: Utc::now(),
            })
        )
    })?;
    
    info!("Checking permission '{}' for user {}", permission, user.user_id);
    
    // Check if user has the permission (supports wildcard matching)
    let has_permission = user.valid_permissions.iter().any(|p| {
        permission_matches(p, &permission)
    });
    
    let expires_at = None;
    let expires_soon = false;
    
    // Skip cache for now - complex Arc to Box conversion needed
    // TODO: Implement proper cache integration for permission expiry info
    // For now, expiry info will remain None
    
    let check_result = SimplePermissionCheck {
        user_id: user.user_id.clone(),
        permission,
        has_permission,
        expires_at,
        expires_soon,
        checked_at: Utc::now(),
    };
    
    Ok(Json(ApiResponse::success(
        check_result,
        if has_permission { 
            "Permission granted" 
        } else { 
            "Permission denied" 
        }
    )))
}

/// Check if user permission matches required permission (supports wildcards)
fn permission_matches(user_permission: &str, required_permission: &str) -> bool {
    // Exact match
    if user_permission == required_permission {
        return true;
    }
    
    // Wildcard matching
    if user_permission.ends_with(":*:*") {
        let prefix = &user_permission[..user_permission.len() - 4]; // Remove ":*:*"
        if required_permission.starts_with(prefix) {
            return true;
        }
    }
    
    if user_permission.ends_with(":*") {
        let prefix = &user_permission[..user_permission.len() - 2]; // Remove ":*"
        if required_permission.starts_with(prefix) {
            return true;
        }
    }
    
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_matching() {
        // Exact match
        assert!(permission_matches("epsx:rankings:view", "epsx:rankings:view"));
        
        // Wildcard matching
        assert!(permission_matches("admin:*:*", "admin:users:manage"));
        assert!(permission_matches("admin:*:*", "admin:permissions:grant"));
        assert!(permission_matches("epsx:*", "epsx:rankings:view"));
        
        // No match
        assert!(!permission_matches("epsx:analytics:view", "epsx:rankings:view"));
        assert!(!permission_matches("user:*", "admin:users:manage"));
    }
}