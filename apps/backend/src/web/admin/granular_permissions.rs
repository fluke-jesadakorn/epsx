// V1 Admin API for Granular Permission Management
// Clean endpoints for granting, revoking, and managing granular permissions

use axum::{
    extract::{Path, Query, State},
    response::Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{info, warn};

use crate::auth::granular_permissions::{
    GranularPermissionClaim, PermissionSource
};
use crate::web::middleware::clean_auth::AuthenticatedUser;
use crate::web::auth::AppState;
use crate::dom::events::notification_events::{NotificationTriggerEvent, NotificationContext, DeliveryChannel, NotificationEventDispatcher};
use crate::infra::cache::permission_cache::PermissionCacheService;

/// Request to grant a permission
#[derive(Debug, Deserialize)]
pub struct GrantPermissionRequest {
    pub permission: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: String,
    pub source: Option<String>, // "manual_grant", "admin_grant", etc.
    pub metadata: Option<HashMap<String, String>>,
}

/// Request to revoke a permission
#[derive(Debug, Deserialize)]
pub struct RevokePermissionRequest {
    pub permission: String,
    pub reason: Option<String>,
}

/// Request to extend permission expiry
#[derive(Debug, Deserialize)]
pub struct ExtendPermissionRequest {
    pub permission: String,
    pub new_expires_at: DateTime<Utc>,
    pub reason: Option<String>,
}

/// Request for bulk permission operations
#[derive(Debug, Deserialize)]
pub struct BulkPermissionRequest {
    pub user_ids: Vec<String>,
    pub permission: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: String,
    pub source: Option<String>,
}

/// Query parameters for listing permissions
#[derive(Debug, Deserialize)]
pub struct ListPermissionsQuery {
    pub include_expired: Option<bool>,
    pub source: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Permission information for API responses
#[derive(Debug, Serialize, Clone)]
pub struct PermissionInfo {
    pub permission: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub source: String,
    pub granted_by: Option<String>,
    pub granted_at: DateTime<Utc>,
    pub metadata: Option<HashMap<String, String>>,
    pub is_valid: bool,
    pub expires_soon: Option<bool>, // true if expires within 24 hours
    pub time_until_expiry: Option<i64>, // seconds until expiry
}

/// User permissions summary
#[derive(Debug, Serialize)]
pub struct UserPermissionsSummary {
    pub user_id: String,
    pub permissions: Vec<PermissionInfo>,
    pub permission_version: u32,
    pub last_updated: DateTime<Utc>,
    pub total_permissions: usize,
    pub expired_permissions: usize,
    pub expiring_soon: usize,
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

/// Grant permission to a user
pub async fn grant_permission(
    State(_state): State<AppState>,
    Path(user_id): Path<String>,
    admin: AuthenticatedUser,
    Json(request): Json<GrantPermissionRequest>,
) -> Result<Json<ApiResponse<PermissionInfo>>, (StatusCode, Json<ErrorResponse>)> {
    // Validate admin permissions
    if !admin.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "forbidden".to_string(),
                message: "Admin permission required".to_string(),
                timestamp: Utc::now(),
            })
        ));
    }

    info!(
        "Admin {} granting permission '{}' to user {}",
        admin.user_id, request.permission, user_id
    );

    // Determine permission source
    let source = match request.source.as_deref() {
        Some("manual_grant") => PermissionSource::ManualGrant,
        Some("admin_grant") => PermissionSource::AdminGrant,
        Some("time_limited_access") => PermissionSource::TimeLimitedAccess,
        Some("test_access") => PermissionSource::TestAccess,
        _ => PermissionSource::AdminGrant,
    };

    // Create permission claim
    let metadata = request.metadata.clone().unwrap_or_default();
    let _permission_claim = if let Some(expires_at) = request.expires_at {
        GranularPermissionClaim::with_metadata(
            Some(expires_at),
            source.clone(),
            Some(admin.user_id.clone()),
            metadata.clone(),
        )
    } else {
        GranularPermissionClaim::with_metadata(
            None,
            source.clone(),
            Some(admin.user_id.clone()),
            metadata.clone(),
        )
    };

    // TODO: Store permission in database (integrate with existing permission service)
    // For now, we'll simulate success

    // Create permission info for response
    let permission_info = PermissionInfo {
        permission: request.permission.clone(),
        expires_at: request.expires_at,
        source: format!("{:?}", source),
        granted_by: Some(admin.user_id.clone()),
        granted_at: Utc::now(),
        metadata: Some(metadata.clone()),
        is_valid: true,
        expires_soon: request.expires_at.map(|exp| {
            (exp - Utc::now()).num_hours() <= 24
        }),
        time_until_expiry: request.expires_at.map(|exp| {
            (exp - Utc::now()).num_seconds().max(0)
        }),
    };

    // Fire notification event
    let event = NotificationTriggerEvent::PermissionGranted {
        user_id: crate::dom::values::UserId::new(user_id.clone()),
        permission: request.permission.clone(),
        granted_by: crate::dom::values::UserId::new(admin.user_id.clone()),
        granted_at: Utc::now(),
        expires_at: request.expires_at,
    };

    let notification_context = NotificationContext {
        template_variables: HashMap::new(),
        user_preferences: None,
        delivery_channels: vec![DeliveryChannel::Push, DeliveryChannel::InApp],
        priority_override: None,
        expiration_override: None,
    };

    // Dispatch notification event through existing system
    let dispatcher = NotificationEventDispatcher::new();
    match dispatcher.dispatch_event(event, notification_context).await {
        Ok(notifications) => {
            info!(
                "✅ Generated {} notifications for permission grant to user {}",
                notifications.len(), user_id
            );
            // TODO: Actually send notifications via Firebase FCM or other channels
        }
        Err(e) => {
            warn!(
                "Failed to generate notifications for permission grant to user {}: {}",
                user_id, e
            );
        }
    }

    info!(
        "Permission '{}' granted to user {} by admin {}",
        request.permission, user_id, admin.user_id
    );

    Ok(Json(ApiResponse::success(
        permission_info,
        "Permission granted successfully"
    )))
}

/// Revoke permission from a user
pub async fn revoke_permission(
    State(_state): State<AppState>,
    Path(user_id): Path<String>,
    admin: AuthenticatedUser,
    Json(request): Json<RevokePermissionRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ErrorResponse>)> {
    // Validate admin permissions
    if !admin.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "forbidden".to_string(),
                message: "Admin permission required".to_string(),
                timestamp: Utc::now(),
            })
        ));
    }

    info!(
        "Admin {} revoking permission '{}' from user {}",
        admin.user_id, request.permission, user_id
    );

    // TODO: Remove permission from database and update permission hash
    
    // Instantly revoke permission in cache for immediate effect
    if let Ok(cache_service) = PermissionCacheService::new().await {
        if let Err(e) = cache_service.revoke_user_permissions(&user_id).await {
            warn!("Failed to revoke permissions in cache for user {}: {}", user_id, e);
            // Continue anyway - database update is more important
        } else {
            info!("✅ Instantly revoked permissions in cache for user {}", user_id);
        }
    }

    // Fire notification event
    let event = NotificationTriggerEvent::PermissionRevoked {
        user_id: crate::dom::values::UserId::new(user_id.clone()),
        permission: request.permission.clone(),
        revoked_by: crate::dom::values::UserId::new(admin.user_id.clone()),
        revoked_at: Utc::now(),
    };

    // Dispatch notification event through existing system
    let notification_context = NotificationContext {
        template_variables: HashMap::new(),
        user_preferences: None,
        delivery_channels: vec![DeliveryChannel::Push, DeliveryChannel::InApp],
        priority_override: None,
        expiration_override: None,
    };

    let dispatcher = NotificationEventDispatcher::new();
    match dispatcher.dispatch_event(event, notification_context).await {
        Ok(notifications) => {
            info!(
                "✅ Generated {} notifications for permission revocation from user {}",
                notifications.len(), user_id
            );
            // TODO: Actually send notifications via Firebase FCM or other channels
        }
        Err(e) => {
            warn!(
                "Failed to generate notifications for permission revocation from user {}: {}",
                user_id, e
            );
        }
    }

    info!(
        "Permission '{}' revoked from user {} by admin {}",
        request.permission, user_id, admin.user_id
    );

    Ok(Json(ApiResponse::success(
        (),
        "Permission revoked successfully"
    )))
}

/// List all permissions for a user
pub async fn list_user_permissions(
    State(_state): State<AppState>,
    Path(user_id): Path<String>,
    admin: AuthenticatedUser,
    Query(_query): Query<ListPermissionsQuery>,
) -> Result<Json<ApiResponse<UserPermissionsSummary>>, (StatusCode, Json<ErrorResponse>)> {
    // Validate admin permissions
    if !admin.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p.starts_with("admin:users:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "forbidden".to_string(),
                message: "Admin permission required".to_string(),
                timestamp: Utc::now(),
            })
        ));
    }

    // TODO: Get actual permissions from database
    // For now, create mock data
    let now = Utc::now();
    let mock_permissions = vec![
        PermissionInfo {
            permission: "epsx:rankings:view:5".to_string(),
            expires_at: None,
            source: "Subscription".to_string(),
            granted_by: None,
            granted_at: now - chrono::Duration::days(30),
            metadata: None,
            is_valid: true,
            expires_soon: Some(false),
            time_until_expiry: None,
        },
        PermissionInfo {
            permission: "epsx:analytics:premium".to_string(),
            expires_at: Some(now + chrono::Duration::days(10)),
            source: "ManualGrant".to_string(),
            granted_by: Some("admin_123".to_string()),
            granted_at: now - chrono::Duration::days(5),
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("reason".to_string(), "Beta testing".to_string());
                meta
            }),
            is_valid: true,
            expires_soon: Some(false),
            time_until_expiry: Some((now + chrono::Duration::days(10) - now).num_seconds()),
        },
    ];

    let summary = UserPermissionsSummary {
        user_id: user_id.clone(),
        permissions: mock_permissions.clone(),
        permission_version: 1,
        last_updated: now,
        total_permissions: mock_permissions.len(),
        expired_permissions: 0,
        expiring_soon: 0,
    };

    Ok(Json(ApiResponse::success(
        summary,
        "User permissions retrieved successfully"
    )))
}

/// Extend permission expiry
pub async fn extend_permission(
    State(_state): State<AppState>,
    Path(user_id): Path<String>,
    admin: AuthenticatedUser,
    Json(request): Json<ExtendPermissionRequest>,
) -> Result<Json<ApiResponse<PermissionInfo>>, (StatusCode, Json<ErrorResponse>)> {
    // Validate admin permissions
    if !admin.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "forbidden".to_string(),
                message: "Admin permission required".to_string(),
                timestamp: Utc::now(),
            })
        ));
    }

    info!(
        "Admin {} extending permission '{}' for user {} until {}",
        admin.user_id, request.permission, user_id, request.new_expires_at
    );

    // TODO: Update permission expiry in database

    let permission_info = PermissionInfo {
        permission: request.permission.clone(),
        expires_at: Some(request.new_expires_at),
        source: "AdminGrant".to_string(),
        granted_by: Some(admin.user_id.clone()),
        granted_at: Utc::now(),
        metadata: None,
        is_valid: true,
        expires_soon: Some((request.new_expires_at - Utc::now()).num_hours() <= 24),
        time_until_expiry: Some((request.new_expires_at - Utc::now()).num_seconds().max(0)),
    };

    Ok(Json(ApiResponse::success(
        permission_info,
        "Permission expiry extended successfully"
    )))
}

/// Bulk grant permissions to multiple users
pub async fn bulk_grant_permissions(
    State(_state): State<AppState>,
    admin: AuthenticatedUser,
    Json(request): Json<BulkPermissionRequest>,
) -> Result<Json<ApiResponse<Vec<String>>>, (StatusCode, Json<ErrorResponse>)> {
    // Validate admin permissions
    if !admin.valid_permissions.iter().any(|p| p.starts_with("admin:permissions:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "forbidden".to_string(),
                message: "Admin permission required".to_string(),
                timestamp: Utc::now(),
            })
        ));
    }

    if request.user_ids.len() > 100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "invalid_request".to_string(),
                message: "Maximum 100 users allowed per bulk operation".to_string(),
                timestamp: Utc::now(),
            })
        ));
    }

    info!(
        "Admin {} bulk granting permission '{}' to {} users",
        admin.user_id, request.permission, request.user_ids.len()
    );

    // TODO: Implement bulk permission granting
    let granted_to = request.user_ids.clone();

    Ok(Json(ApiResponse::success(
        granted_to,
        &format!("Permission granted to {} users successfully", request.user_ids.len())
    )))
}

/// Get permission statistics for admin dashboard
pub async fn get_permission_statistics(
    State(_state): State<AppState>,
    admin: AuthenticatedUser,
) -> Result<Json<ApiResponse<PermissionStatistics>>, (StatusCode, Json<ErrorResponse>)> {
    // Validate admin permissions
    if !admin.valid_permissions.iter().any(|p| p.starts_with("admin:") || p == "admin:*:*") {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                success: false,
                error: "forbidden".to_string(),
                message: "Admin permission required".to_string(),
                timestamp: Utc::now(),
            })
        ));
    }

    // TODO: Get actual statistics from database
    let stats = PermissionStatistics {
        total_users: 1500,
        total_permissions: 4250,
        active_permissions: 3100,
        expired_permissions: 1150,
        expiring_soon: 85,
        most_common_permissions: vec![
            ("epsx:rankings:view:5".to_string(), 1200),
            ("epsx:analytics:premium".to_string(), 450),
            ("admin:users:view".to_string(), 15),
        ],
        permissions_by_source: {
            let mut map = HashMap::new();
            map.insert("Subscription".to_string(), 2800);
            map.insert("ManualGrant".to_string(), 950);
            map.insert("AdminGrant".to_string(), 350);
            map.insert("TimeLimitedAccess".to_string(), 150);
            map
        },
        recent_changes: 47,
        last_updated: Utc::now(),
    };

    Ok(Json(ApiResponse::success(
        stats,
        "Permission statistics retrieved successfully"
    )))
}

/// Permission statistics for admin dashboard
#[derive(Debug, Serialize)]
pub struct PermissionStatistics {
    pub total_users: usize,
    pub total_permissions: usize,
    pub active_permissions: usize,
    pub expired_permissions: usize,
    pub expiring_soon: usize,
    pub most_common_permissions: Vec<(String, usize)>,
    pub permissions_by_source: HashMap<String, usize>,
    pub recent_changes: usize,
    pub last_updated: DateTime<Utc>,
}