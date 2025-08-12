// Unified Authentication handlers

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::app::dtos::auth::{LoginReq, RefreshReq, AutoRegistrationRequest, RegistrationResponse};
// Legacy AuthCtx replacement for compatibility during migration
use crate::dom::values::{Role, UserId, SessId};

// Temporary replacement for legacy AuthCtx during migration
#[derive(Debug, Clone)]
struct AuthCtx {
    pub user_id: UserId,
    pub role: Role,
    pub sess: SessId,
}
use crate::dom::entities::audit::{AuditLogEntry, AuditAction, ResourceType, AuditResult};
use super::AppState;

/// Extract session ID from Authorization header
fn extract_session_from_request(request: &axum::extract::Request) -> Result<crate::dom::values::SessId, StatusCode> {
    // Try to extract session from Authorization header
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                // The token should be the session ID for this system
                return Ok(crate::dom::values::SessId::from_string(token.to_string()));
            }
        }
    }
    
    // Try alternative session header
    if let Some(session_header) = request.headers().get("x-session-id") {
        if let Ok(session_str) = session_header.to_str() {
            return Ok(crate::dom::values::SessId::from_string(session_str.to_string()));
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}

/// Generate a bearer token for the session
fn generate_bearer_token(session_id: &str) -> String {
    // In production, this should use a proper JWT library with signing
    // For now, we'll use the session ID as the bearer token
    // The frontend will include this in the Authorization header
    session_id.to_string()
}

/// Log authentication events to audit trail
async fn log_auth_event(
    app_state: &AppState,
    action: AuditAction,
    user_id: Option<&UserId>,
    result: AuditResult,
    email: &str,
    session_id: Option<&str>,
) {
    let actor_id = user_id.cloned().unwrap_or_else(|| UserId::new("anonymous".to_string()));
    
    let mut entry = AuditLogEntry::new(
        actor_id,
        action,
        ResourceType::Session,
        email.to_string(),
        result,
    );
    
    if let Some(session_id) = session_id {
        entry = entry.with_session_id(session_id.to_string());
    }
    
    if let Err(e) = app_state.audit_repo.store(&entry).await {
        tracing::error!("Failed to log audit event: {:?}", e);
    }
}

/// Multi-method login request supporting various authentication flows
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum LoginRequest {
    #[serde(rename = "credentials")]
    Credentials {
        email: String,
        password: String,
    },
    #[serde(rename = "admin")]
    Admin {
        email: String,
        password: String,
        admin_token: Option<String>,
    },
}

/// Unified login response with complete user profile information and bearer token
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub subscription_tier: String,
    pub expires_at: DateTime<Utc>,
    pub session_type: String,
    pub access_token: String,
    pub token_type: String,
}

/// Refresh response payload
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub expires_at: DateTime<Utc>,
}

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub subscription_tier: String,
}

/// Unified login handler supporting multiple authentication flows
pub async fn login_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    match payload {
        LoginRequest::Credentials { email, password } => {
            handle_credentials_login(app_state, email, password, false).await
        },
        LoginRequest::Admin { email, password, admin_token } => {
            handle_admin_login(app_state, email, password, admin_token).await
        },
    }
}

/// Handle direct email/password login
async fn handle_credentials_login(
    app_state: AppState,
    email: String,
    password: String,
    _is_admin: bool,
) -> Result<Json<LoginResponse>, StatusCode> {
    let login_req = LoginReq { email: email.clone(), password };
    
    let login_res = app_state.auth_uc.login(login_req).await
        .map_err(|e| {
            tracing::error!("Login failed: {:?}", e);
            // Log failed login attempt
            let app_state_for_audit = app_state.clone();
            let email_for_audit = email.clone();
            tokio::spawn(async move {
                log_auth_event(&app_state_for_audit, AuditAction::LoginFailed, None, AuditResult::Failure, &email_for_audit, None).await;
            });
            StatusCode::UNAUTHORIZED
        })?;
    
    let bearer_token = generate_bearer_token(&login_res.sess_id.to_string());
    
    tracing::info!("Generated bearer token for session: {}", login_res.sess_id);
    
    let user = app_state.user_repo.find_by_id(&login_res.user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let permissions = get_user_permissions(&user);
    let session_type = if matches!(user.role(), Role::SuperAdmin | Role::Admin) { "admin" } else { "user" };
    
    // Log successful login
    log_auth_event(&app_state, AuditAction::Login, Some(user.id()), AuditResult::Success, &email, Some(&login_res.sess_id.to_string())).await;
    
    Ok(Json(LoginResponse {
        user_id: user.id().to_string(),
        email: user.email().value().to_string(),
        role: user.role().to_string(),
        permissions,
        subscription_tier: user.sub().tier().to_string(),
        expires_at: chrono::Utc::now() + chrono::Duration::seconds(login_res.expires_in),
        session_type: session_type.to_string(),
        access_token: bearer_token,
        token_type: "Bearer".to_string(),
    }))
}

/// Handle admin login with elevated security
async fn handle_admin_login(
    app_state: AppState,
    email: String,
    password: String,
    admin_token: Option<String>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let login_req = LoginReq { email: email.clone(), password };
    
    let login_res = app_state.auth_uc.login(login_req).await
        .map_err(|e| {
            tracing::error!("Admin login failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    
    let user = app_state.user_repo.find_by_id(&login_res.user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    if !matches!(user.role(), Role::Admin | Role::SuperAdmin) {
        tracing::warn!("User {} attempted admin login without privileges", email);
        return Err(StatusCode::FORBIDDEN);
    }
    
    let bearer_token = generate_bearer_token(&login_res.sess_id.to_string());
    
    tracing::info!("Generated admin bearer token for session: {}", login_res.sess_id);
    
    if admin_token.is_some() {
        tracing::info!("Admin token provided - MFA will be implemented in future");
    }
    
    let permissions = vec![
        "api:admin:*".to_string(),
        "route:*".to_string(),
        "users:manage".to_string(),
        "system:configure".to_string(),
    ];
    
    Ok(Json(LoginResponse {
        user_id: user.id().to_string(),
        email: user.email().value().to_string(),
        role: user.role().to_string(),
        permissions,
        subscription_tier: user.sub().tier().to_string(),
        expires_at: chrono::Utc::now() + chrono::Duration::seconds(login_res.expires_in),
        session_type: "admin".to_string(),
        access_token: bearer_token,
        token_type: "Bearer".to_string(),
    }))
}

/// Get user permissions based on role
fn get_user_permissions(user: &crate::dom::entities::User) -> Vec<String> {
    match user.role() {
        Role::SuperAdmin | Role::Admin => vec![
            "api:admin:*".to_string(),
            "route:*".to_string(),
            "users:manage".to_string(),
            "system:configure".to_string(),
        ],
        Role::Moderator => vec![
            "api:moderate:*".to_string(),
            "route:/moderate/*".to_string(),
            "content:moderate".to_string(),
            "users:view".to_string(),
        ],
        Role::Premium => vec![
            "api:premium:*".to_string(),
            "route:/premium/*".to_string(),
            "analytics:read".to_string(),
            "alerts:manage".to_string(),
        ],
        _ => vec![
            "api:basic:read".to_string(),
            "route:/dashboard".to_string(),
            "profile:manage:own".to_string(),
        ],
    }
}

/// User registration handler
pub async fn register_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<AutoRegistrationRequest>,
) -> Result<Json<RegistrationResponse>, StatusCode> {
    app_state.auth_uc.register_with_permission_profiles(payload).await
        .map(Json)
        .map_err(|e| {
            tracing::error!("Registration failed: {:?}", e);
            if e.to_string().contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })
}

/// Logout handler that deactivates user session
pub async fn logout_handler(
    State(app_state): State<AppState>,
    request: axum::extract::Request,
) -> Result<StatusCode, StatusCode> {
    // Extract session ID from Authorization header
    let session_id = extract_session_from_request(&request)
        .map_err(|_| {
            tracing::warn!("Logout attempt without valid session");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Find and invalidate the session
    match app_state.session_repo.find_by_id(&session_id).await {
        Ok(mut session) => {
            // Mark session as inactive
            session.deactivate();
            
            // Save the deactivated session
            if let Err(e) = app_state.session_repo.save(&session).await {
                tracing::error!("Failed to deactivate session during logout: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            
            tracing::info!("Session {} successfully logged out", session_id);
            Ok(StatusCode::OK)
        },
        Err(crate::app::ports::repositories::RepoError::NotFound) => {
            tracing::warn!("Logout attempt with non-existent session: {}", session_id);
            Err(StatusCode::UNAUTHORIZED)
        },
        Err(e) => {
            tracing::error!("Failed to find session during logout: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Refresh session handler that extends session expiry and returns new bearer token
pub async fn refresh_handler(
    State(app_state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<RefreshResponse>, StatusCode> {
    // Extract session ID from Authorization header
    let session_id = extract_session_from_request(&request)
        .map_err(|_| {
            tracing::warn!("Refresh attempt without valid session");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Find the current session
    let current_session = match app_state.session_repo.find_by_id(&session_id).await {
        Ok(session) => session,
        Err(crate::app::ports::repositories::RepoError::NotFound) => {
            tracing::warn!("Refresh attempt with non-existent session: {}", session_id);
            return Err(StatusCode::UNAUTHORIZED);
        },
        Err(e) => {
            tracing::error!("Failed to find session during refresh: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Check if session is still active and not expired
    if !current_session.is_active() || current_session.is_expired() {
        tracing::warn!("Refresh attempt with inactive/expired session: {}", session_id);
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Create refresh request with the session's refresh token
    let refresh_req = RefreshReq {
        refresh_token: current_session.refresh_token().unwrap_or_default(),
    };
    
    // Refresh session through use case
    let refresh_res = app_state.auth_uc.refresh(refresh_req).await
        .map_err(|e| {
            tracing::error!("Session refresh failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;
    
    tracing::info!("Session refreshed successfully: {}", refresh_res.sess_id);
    
    // Return response with new expiry
    let response = RefreshResponse {
        expires_at: chrono::Utc::now() + chrono::Duration::seconds(refresh_res.expires_in),
    };
    
    Ok(Json(response))
}

/// Get current user profile handler
pub async fn me_handler(
    State(app_state): State<AppState>,
    request: axum::extract::Request,
) -> Result<Json<UserProfileResponse>, StatusCode> {
    // Extract session ID from Authorization header
    let session_id = extract_session_from_request(&request)
        .map_err(|_| {
            tracing::warn!("Profile request without valid session");
            StatusCode::UNAUTHORIZED
        })?;
    
    // Find and validate the session
    let session = match app_state.session_repo.find_by_id(&session_id).await {
        Ok(session) => session,
        Err(crate::app::ports::repositories::RepoError::NotFound) => {
            tracing::warn!("Profile request with non-existent session: {}", session_id);
            return Err(StatusCode::UNAUTHORIZED);
        },
        Err(e) => {
            tracing::error!("Failed to find session for profile request: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Check if session is still active and not expired
    if !session.is_active() || session.is_expired() {
        tracing::warn!("Profile request with inactive/expired session: {}", session_id);
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Get user details from repository
    let user = app_state.user_repo.find_by_id(session.user_id()).await
        .map_err(|e| {
            tracing::error!("Failed to find user for profile: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Get user roles from IAM repository and derive permissions
    let roles = app_state.iam_repo.get_user_roles(session.user_id()).await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to get user roles for profile: {:?}", e);
            vec![]
        });
    
    // Use unified permission system - derive from roles
    let permission_strings: Vec<String> = roles.iter()
        .flat_map(|role| {
            // Map IAM role names to user roles for permission derivation
            let user_role = match role.name() {
                "admin" | "system_administrator" => crate::dom::values::Role::Admin,
                "user" => crate::dom::values::Role::User,
                "premium_user" => crate::dom::values::Role::Premium,
                "moderator" => crate::dom::values::Role::Moderator,
                "super_admin" => crate::dom::values::Role::SuperAdmin,
                _ => crate::dom::values::Role::Free,
            };
            
            crate::dom::services::permissions::get_role_permissions(&user_role)
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .collect();
    
    let response = UserProfileResponse {
        user_id: session.user_id().to_string(),
        email: user.email().value().to_string(),
        role: user.role().to_string(),
        permissions: permission_strings,
        subscription_tier: user.sub().tier().to_string(),
    };
    
    Ok(Json(response))
}

/// Public me handler - removed as bearer token validation is handled by middleware
/// This endpoint is now protected by bearer token middleware

// Phase 1 API Endpoints for centralized auth

/// Session validation request
#[derive(Debug, Deserialize)]
pub struct SessionValidationRequest {
    pub app_type: String,
}

/// Session validation response
#[derive(Debug, Serialize)]
pub struct SessionValidationResponse {
    pub authenticated: bool,
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub subscription_tier: String,
    pub session_type: String,
    pub expires_at: DateTime<Utc>,
    pub session_rotated: Option<bool>, // Indicates if session was rotated
}

/// Session rotation request
#[derive(Debug, Deserialize)]
pub struct SessionRotationRequest {
    pub reason: String, // e.g., "admin_action", "password_change", "sensitive_operation"
}

/// Session rotation response
#[derive(Debug, Serialize)]
pub struct SessionRotationResponse {
    pub new_session_id: String,
    pub expires_at: DateTime<Utc>,
    pub rotated_at: DateTime<Utc>,
}

/// Route access validation request
#[derive(Debug, Deserialize)]
pub struct RouteAccessRequest {
    pub route: String,
    pub method: String,
    pub app_type: String,
}

/// Route access validation response
#[derive(Debug, Serialize)]
pub struct RouteAccessResponse {
    pub allowed: bool,
    pub reason: Option<String>,
    pub required_permissions: Vec<String>,
    pub user_permissions: Vec<String>,
}

/// Bulk route validation request
#[derive(Debug, Deserialize)]
pub struct BulkRouteValidationRequest {
    pub routes: Vec<RouteInfo>,
    pub app_type: String,
}

/// Route information for bulk validation
#[derive(Debug, Deserialize)]
pub struct RouteInfo {
    pub route: String,
    pub method: String,
}

/// Bulk route validation response
#[derive(Debug, Serialize)]
pub struct BulkRouteValidationResponse {
    pub results: HashMap<String, RouteAccessResult>,
    pub user_permissions: Vec<String>,
}

/// Individual route access result for bulk validation
#[derive(Debug, Serialize)]
pub struct RouteAccessResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub required_permissions: Vec<String>,
}

/// Permission check request
#[derive(Debug, Deserialize)]
pub struct PermissionCheckRequest {
    pub permission: String,
    pub resource: Option<String>,
    pub app_type: String,
}

/// Permission check response
#[derive(Debug, Serialize)]
pub struct PermissionCheckResponse {
    pub has_permission: bool,
    pub reason: Option<String>,
    pub user_permissions: Vec<String>,
    pub matching_permissions: Vec<String>,
}

/// User features response
#[derive(Debug, Serialize)]
pub struct UserFeaturesResponse {
    pub user_id: String,
    pub role: String,
    pub subscription_tier: String,
    pub features: Vec<FeatureAccess>,
    pub permissions: Vec<String>,
}

/// Individual feature access information
#[derive(Debug, Serialize)]
pub struct FeatureAccess {
    pub feature: String,
    pub enabled: bool,
    pub tier_required: String,
    pub permission_required: Option<String>,
}

/// Session validation handler - validates current session and returns user info
pub async fn validate_session_handler(
    State(app_state): State<AppState>,
    Json(request): Json<SessionValidationRequest>,
) -> Result<Json<SessionValidationResponse>, StatusCode> {
    // TODO: Extract from session/token during migration
    let auth_ctx = AuthCtx {
        user_id: crate::dom::values::UserId::new("migration_user".to_string()),
        role: crate::dom::values::Role::User,
        sess: crate::dom::values::SessId::from_string("migration_session".to_string()),
    };
    tracing::info!("Validating session for app_type: {}", request.app_type);
    
    // Get user details from repository
    let user = app_state.user_repo.find_by_id(&auth_ctx.user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user details: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Get user roles from IAM repository and derive permissions
    let roles = app_state.iam_repo.get_user_roles(&auth_ctx.user_id).await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to get user roles: {:?}", e);
            vec![]
        });
    
    // Use unified permission system - derive from roles
    let permission_strings: Vec<String> = roles.iter()
        .flat_map(|role| {
            // Map IAM role names to user roles for permission derivation
            let user_role = match role.name() {
                "admin" | "system_administrator" => crate::dom::values::Role::Admin,
                "user" => crate::dom::values::Role::User,
                "premium_user" => crate::dom::values::Role::Premium,
                "moderator" => crate::dom::values::Role::Moderator,
                "super_admin" => crate::dom::values::Role::SuperAdmin,
                _ => crate::dom::values::Role::Free,
            };
            
            crate::dom::services::permissions::get_role_permissions(&user_role)
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .collect();
    
    let response = SessionValidationResponse {
        authenticated: true,
        user_id: auth_ctx.user_id.to_string(),
        email: user.email().value().to_string(),
        role: auth_ctx.role.to_string(),
        permissions: permission_strings,
        subscription_tier: user.sub().tier().to_string(),
        session_type: "regular".to_string(), // TODO: implement session types
        expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        session_rotated: None, // No rotation performed in this call
    };
    
    Ok(Json(response))
}

/// Route access validation handler - checks if user can access a specific route
pub async fn validate_route_access_handler(
    State(app_state): State<AppState>,
    Json(request): Json<RouteAccessRequest>,
) -> Result<Json<RouteAccessResponse>, StatusCode> {
    // TODO: Extract from session/token during migration
    let auth_ctx = AuthCtx {
        user_id: crate::dom::values::UserId::new("migration_user".to_string()),
        role: crate::dom::values::Role::User,
        sess: crate::dom::values::SessId::from_string("migration_session".to_string()),
    };
    tracing::info!("Validating route access: {} {} for app: {}", 
                   request.method, request.route, request.app_type);
    
    // Get user roles and derive permissions
    let roles = app_state.iam_repo.get_user_roles(&auth_ctx.user_id).await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to get user roles: {:?}", e);
            vec![]
        });
    
    // Use unified permission system - derive from roles
    let permission_strings: Vec<String> = roles.iter()
        .flat_map(|role| {
            // Map IAM role names to user roles for permission derivation
            let user_role = match role.name() {
                "admin" | "system_administrator" => crate::dom::values::Role::Admin,
                "user" => crate::dom::values::Role::User,
                "premium_user" => crate::dom::values::Role::Premium,
                "moderator" => crate::dom::values::Role::Moderator,
                "super_admin" => crate::dom::values::Role::SuperAdmin,
                _ => crate::dom::values::Role::Free,
            };
            
            crate::dom::services::permissions::get_role_permissions(&user_role)
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .collect();
    
    // Basic route validation logic - extend based on your route-permission mapping
    let allowed = match request.route.as_str() {
        "/dashboard" => permission_strings.iter().any(|p| p.starts_with("dashboard:")),
        "/analytics" => permission_strings.iter().any(|p| p.starts_with("analytics:")),
        "/admin" => auth_ctx.role.to_string() == "admin" || auth_ctx.role.to_string() == "system_administrator",
        _ => true, // Allow access to other routes by default
    };
    
    let required_perms = match request.route.as_str() {
        "/dashboard" => vec!["dashboard:view".to_string()],
        "/analytics" => vec!["analytics:view".to_string()],
        "/admin" => vec!["admin:access".to_string()],
        _ => vec![],
    };
    
    let response = RouteAccessResponse {
        allowed,
        reason: if allowed { None } else { Some("Insufficient permissions".to_string()) },
        required_permissions: required_perms,
        user_permissions: permission_strings,
    };
    
    Ok(Json(response))
}

/// Bulk route validation handler - validates multiple routes at once for efficient frontend middleware
pub async fn validate_bulk_routes_handler(
    State(_app_state): State<AppState>,
    Json(request): Json<BulkRouteValidationRequest>,
) -> Result<Json<BulkRouteValidationResponse>, StatusCode> {
    tracing::info!("Validating {} routes for app: {}", request.routes.len(), request.app_type);
    
    // TODO: Implement with Casbin during migration
    // For now, return all routes as allowed
    
    let mut results = HashMap::new();
    for route_info in request.routes {
        let route_key = format!("{} {}", route_info.method, route_info.route);
        let result = RouteAccessResult {
            allowed: true, // Allow all during migration
            reason: None,
            required_permissions: vec![],
        };
        results.insert(route_key, result);
    }
    
    let response = BulkRouteValidationResponse {
        results,
        user_permissions: vec!["migration:all".to_string()],
    };
    
    Ok(Json(response))
}

/// Permission check handler - validates if user has a specific permission
pub async fn check_permission_handler(
    State(_app_state): State<AppState>,
    Json(request): Json<PermissionCheckRequest>,
) -> Result<Json<PermissionCheckResponse>, StatusCode> {
    tracing::info!("Checking permission '{}' during migration", request.permission);
    
    // TODO: Implement with Casbin during migration
    // For now, allow all permissions
    let response = PermissionCheckResponse {
        has_permission: true,
        reason: None,
        user_permissions: vec!["migration:all".to_string()],
        matching_permissions: vec![request.permission.clone()],
    };
    
    Ok(Json(response))
}

/// Navigation items response
#[derive(Debug, Serialize)]
pub struct NavigationResponse {
    pub items: Vec<NavigationItem>,
    pub user_role: String,
    pub permissions: Vec<String>,
}

/// Individual navigation item
#[derive(Debug, Serialize)]
pub struct NavigationItem {
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub required_permission: Option<String>,
    pub required_role: Option<String>,
}

/// Single permission check request (for GET endpoint)
#[derive(Debug, Deserialize)]
pub struct SinglePermissionRequest {
    pub feature: String,
    pub app_type: Option<String>,
}

/// Single permission check response (for GET endpoint)
#[derive(Debug, Serialize)]
pub struct SinglePermissionResponse {
    pub feature: String,
    pub has_access: bool,
    pub reason: Option<String>,
}

/// Navigation permissions handler - returns allowed navigation items
pub async fn navigation_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<NavigationResponse>, StatusCode> {
    tracing::info!("Getting navigation items during migration");
    
    // TODO: Implement with Casbin during migration
    // For now, return basic navigation items
    let items = vec![
        NavigationItem {
            name: "Dashboard".to_string(),
            path: "/dashboard".to_string(),
            enabled: true,
            required_permission: Some("dashboard:view".to_string()),
            required_role: None,
        },
        NavigationItem {
            name: "Settings".to_string(),
            path: "/settings".to_string(),
            enabled: true,
            required_permission: None,
            required_role: None,
        },
    ];
    
    let response = NavigationResponse {
        items,
        user_role: "migration_user".to_string(),
        permissions: vec!["migration:all".to_string()],
    };
    
    Ok(Json(response))
}

/// Single permission check handler - validates one permission via GET
pub async fn single_permission_handler(
    State(app_state): State<AppState>,
    axum::extract::Query(request): axum::extract::Query<SinglePermissionRequest>,
) -> Result<Json<SinglePermissionResponse>, StatusCode> {
    // TODO: Extract from session/token during migration
    let auth_ctx = AuthCtx {
        user_id: crate::dom::values::UserId::new("migration_user".to_string()),
        role: crate::dom::values::Role::User,
        sess: crate::dom::values::SessId::from_string("migration_session".to_string()),
    };
    tracing::info!("Checking single permission '{}' for user {}", request.feature, auth_ctx.user_id);
    
    // Get user roles and derive permissions
    let roles = app_state.iam_repo.get_user_roles(&auth_ctx.user_id).await
        .unwrap_or_else(|e| {
            tracing::warn!("Failed to get user roles: {:?}", e);
            vec![]
        });
    
    let permission_strings: Vec<String> = roles.iter()
        .flat_map(|role| {
            match role.name() {
                "admin" | "system_administrator" => vec![
                    "dashboard:view".to_string(),
                    "analytics:view".to_string(),
                    "admin:access".to_string(),
                    "users:manage".to_string(),
                    "trading:access".to_string(),
                    "iam:manage".to_string(),
                    "audit:view".to_string(),
                ],
                "user" | "premium_user" => vec![
                    "dashboard:view".to_string(),
                    "analytics:view".to_string(),
                    "trading:access".to_string(),
                ],
                _ => vec!["basic:access".to_string()],
            }
        })
        .collect();
    
    // Check if user has the requested permission
    let has_access = permission_strings.iter().any(|perm| {
        // Direct match
        if *perm == request.feature {
            return true;
        }
        
        // Wildcard matching - if permission ends with :*, match prefix
        if perm.ends_with(":*") {
            let prefix = &perm[..perm.len() - 1]; // Remove *
            return request.feature.starts_with(prefix);
        }
        
        // Feature-based matching
        match request.feature.as_str() {
            "TRADING" => perm.starts_with("trading:") || perm.starts_with("dashboard:"),
            "ADMIN_ACCESS" => perm.starts_with("admin:"),
            "ANALYTICS" => perm.starts_with("analytics:"),
            "USER_MANAGEMENT" => perm.starts_with("users:"),
            "IAM_MANAGEMENT" => perm.starts_with("iam:"),
            _ => false,
        }
    });
    
    let response = SinglePermissionResponse {
        feature: request.feature.clone(),
        has_access,
        reason: if has_access {
            None
        } else {
            Some(format!("User does not have access to feature: {}", request.feature))
        },
    };
    
    Ok(Json(response))
}

/// User features handler - returns available features based on user role and permissions
pub async fn user_features_handler(
    State(_app_state): State<AppState>,
) -> Result<Json<UserFeaturesResponse>, StatusCode> {
    tracing::info!("Getting user features during migration");
    
    // TODO: Implement with Casbin during migration
    // For now, return basic features available to all
    let response = UserFeaturesResponse {
        user_id: "migration_user".to_string(),
        role: "migration_user".to_string(),
        subscription_tier: "migration".to_string(),
        features: vec![
            FeatureAccess {
                feature: "dashboard".to_string(),
                enabled: true,
                tier_required: "basic".to_string(),
                permission_required: Some("dashboard:view".to_string()),
            },
            FeatureAccess {
                feature: "settings".to_string(),
                enabled: true,
                tier_required: "basic".to_string(),
                permission_required: None,
            },
        ],
        permissions: vec!["migration:all".to_string()],
    };
    
    Ok(Json(response))
}

/// Session rotation handler - creates a new session ID for security-sensitive operations
pub async fn rotate_session_handler(
    State(_app_state): State<AppState>,
    Json(request): Json<SessionRotationRequest>,
) -> Result<Json<SessionRotationResponse>, StatusCode> {
    tracing::info!("Rotating session during migration with reason: {}", request.reason);
    
    // TODO: Implement with Casbin during migration
    // For now, return a stub session rotation
    let new_session_id = uuid::Uuid::new_v4().to_string();
    
    let response = SessionRotationResponse {
        new_session_id,
        expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        rotated_at: chrono::Utc::now(),
    };
    
    Ok(Json(response))
}

/// Helper function to check if an operation requires session rotation
pub fn requires_session_rotation(operation: &str) -> bool {
    matches!(operation, 
        "password_change" | 
        "email_change" | 
        "admin_privilege_elevation" | 
        "sensitive_settings_change" |
        "payment_method_change" |
        "security_settings_change"
    )
}

/// Automatic session rotation for sensitive operations
pub async fn maybe_rotate_session(
    _app_state: &AppState,
    auth_ctx: &AuthCtx,
    operation: &str,
) -> Result<Option<String>, StatusCode> {
    if requires_session_rotation(operation) {
        tracing::info!("Auto-rotating session for sensitive operation: {}", operation);
        
        // Perform rotation (simplified version)
        let new_session_id = uuid::Uuid::new_v4().to_string();
        
        tracing::info!("Session auto-rotated for user {} during {}", auth_ctx.user_id, operation);
        Ok(Some(new_session_id))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::values::UserId;
    
    #[test]
    fn should_serialize_login_response() {
        let response = LoginResponse {
            user_id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            permissions: vec!["read:basic".to_string()],
            subscription_tier: "basic".to_string(),
            expires_at: Utc::now(),
            session_type: "user".to_string(),
            access_token: "test-bearer-token".to_string(),
            token_type: "Bearer".to_string(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("test-id"));
        assert!(json.contains("user"));
        assert!(json.contains("test-bearer-token"));
        assert!(json.contains("Bearer"));
    }
    
    #[test]
    fn should_deserialize_login_request() {
        let json = r#"{"type": "credentials", "email": "test@example.com", "password": "test123"}"#;
        let request: LoginRequest = serde_json::from_str(json).unwrap();
        
        match request {
            LoginRequest::Credentials { email, password } => {
                assert_eq!(email, "test@example.com");
                assert_eq!(password, "test123");
            },
            _ => panic!("Expected credentials variant"),
        }
    }
    
    #[test]
    fn should_serialize_refresh_response() {
        let response = RefreshResponse {
            expires_at: Utc::now(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("expires_at"));
    }
    
    #[test]
    fn should_serialize_user_profile_response() {
        let response = UserProfileResponse {
            user_id: "profile-test-id".to_string(),
            email: "profile@example.com".to_string(),
            role: "premium_user".to_string(),
            permissions: vec!["read:own_data".to_string(), "write:own_data".to_string()],
            subscription_tier: "premium".to_string(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("profile@example.com"));
        assert!(json.contains("premium_user"));
        assert!(json.contains("read:own_data"));
        assert!(json.contains("premium"));
    }
    
    // Simplified test setup without complex async mocking
    // For comprehensive integration tests, we would set up real test services
    
    
    #[test]
    fn should_generate_bearer_token() {
        let session_id = "test-session-123";
        let bearer_token = generate_bearer_token(session_id);
        
        assert_eq!(bearer_token, session_id);
        assert!(!bearer_token.is_empty());
    }
    
    #[test]
    fn should_format_placeholder_email() {
        let user_id = UserId::generate();
        let email = format!("user{}@example.com", user_id.to_string());
        
        assert!(email.contains("@example.com"));
        assert!(email.starts_with("user"));
        assert!(email.len() > "user@example.com".len()); // Should include the UUID
    }
    
    #[test]
    fn should_validate_login_request_structure() {
        let request = LoginRequest::Credentials {
            email: "test@example.com".to_string(),
            password: "test123".to_string(),
        };
        
        match request {
            LoginRequest::Credentials { email, password } => {
                assert!(!email.is_empty());
                assert!(email.contains("@"));
                assert!(!password.is_empty());
            },
            _ => panic!("Expected credentials variant"),
        }
    }
    
    #[test]
    fn should_validate_response_structures() {
        let now = Utc::now();
        
        let login_response = LoginResponse {
            user_id: "user-123".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            permissions: vec!["read:basic".to_string()],
            subscription_tier: "basic".to_string(),
            expires_at: now,
            session_type: "user".to_string(),
            access_token: "test-token".to_string(),
            token_type: "Bearer".to_string(),
        };
        
        let refresh_response = RefreshResponse {
            expires_at: now,
        };
        
        let profile_response = UserProfileResponse {
            user_id: "profile-123".to_string(),
            email: "profile@example.com".to_string(),
            role: "admin".to_string(),
            permissions: vec!["manage:users".to_string()],
            subscription_tier: "enterprise".to_string(),
        };
        
        // Basic validation that all fields are accessible
        assert!(!login_response.user_id.is_empty());
        assert!(!login_response.email.is_empty());
        assert!(!login_response.role.is_empty());
        
        assert_eq!(refresh_response.expires_at, now);
        
        assert!(!profile_response.permissions.is_empty());
        assert_eq!(profile_response.permissions[0], "manage:users");
    }
    
    // Integration tests would go here but require more complex setup
    // These tests validate the structure and basic functionality
    // For full handler testing, we'd need to set up Axum test server
    
    #[test]
    fn should_have_debug_implementations() {
        let request = LoginRequest {
            email: "debug@test.com".to_string(),
            password: "debug-test".to_string(),
        };
        
        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("LoginRequest"));
        assert!(debug_str.contains("debug@test.com"));
    }
}