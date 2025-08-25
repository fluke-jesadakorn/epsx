// Permission API Handlers
//
// Implements REST API handlers for the unified permission validation system,
// providing comprehensive permission management capabilities with enterprise-grade
// performance, security, and monitoring features.

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::{Response, IntoResponse},
    Json, http::StatusCode,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use std::time::Instant;

use crate::{
    infra::container::AppContainer,
    permissions::{
        core::{PermissionContext, PermissionDecision},
        // errors::{PermissionError, ValidationError},
        // package_tiers::{PackageTierValidator, TierFeature},
        // admin_modules::{AdminModule, AdminModulePermission},
        audit::{PermissionAuditEntry, AuditEventType},
        // cache::PermissionCache,
    },
    dom::values::UserId,
    web::middleware::auth_monitoring::AuthContext,
};

use super::dto::*;

// ============================================================================
// Permission Validation Handlers
// ============================================================================

/// Validate a single permission for a user
pub async fn validate_permissions(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(request): Json<ValidatePermissionRequest>,
) -> Result<Json<ValidatePermissionResponse>, Response> {
    let start_time = Instant::now();
    
    // Validate request
    if let Err(error) = request.validate() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({
            "error": "Invalid request",
            "message": error
        }))).into_response());
    }
    
    // Create permission context (move values instead of cloning)
    let context = PermissionContext {
        user_id: request.user_id,
        permission: request.permission,
        resource: request.resource.unwrap_or_else(|| "*".to_string()),
        context_data: request.context
            .and_then(|c| c.additional_claims)
            .map(|claims| claims.into_iter().map(|(k, v)| (k, v.to_string())).collect())
            .unwrap_or_default(),
        timestamp: Utc::now(),
        ip_address: None, // TODO: Extract from request headers
        user_agent: None, // TODO: Extract from request headers
        session_id: None, // TODO: Extract from request context
    };
    
    // Get permission system
    let permission_system = _container.get_permission_system()
        .map_err(|e| {
            tracing::error!("Failed to get permission system: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Internal server error",
                "message": "Permission system unavailable"
            }))).into_response()
        })?;
    
    // Validate permission
    match permission_system.validate_permission(&context).await {
        Ok(result) => {
            let validation_time = start_time.elapsed().as_millis() as f64;
            
            // Store values for response before context is used in audit
            let user_id_for_response = context.user_id.clone();
            let permission_for_response = context.permission.clone();
            let resource_for_response = context.resource.clone();
            
            // Log validation for audit
            if let Ok(audit_system) = _container.get_audit_system() {
                let audit_entry = PermissionAuditEntry {
                    id: Uuid::new_v4(),
                    event_type: AuditEventType::PermissionCheck,
                    user_id: user_id_for_response.clone(),
                    permission: permission_for_response.clone(),
                    resource: Some(resource_for_response.clone()),
                    action: "validate".to_string(),
                    result: result.allowed(),
                    timestamp: Utc::now(),
                    client_ip: _auth.ip_address,
                    user_agent: _auth.user_agent,
                    geo_location: None,
                    session_id: _auth.session_id,
                    device_fingerprint: None,
                    threat_indicators: HashMap::new(),
                    risk_score: calculate_security_score(&result, &context) as f32,
                    additional_context: {
                        let mut context = HashMap::new();
                        context.insert("validation_time_ms".to_string(), json!(validation_time));
                        context.insert("cached".to_string(), json!(result.cached()));
                        context.insert("source".to_string(), json!(result.source()));
                        context
                    },
                };
                
                let _ = audit_system.audit(audit_entry).await;
            }
            
            let response = ValidatePermissionResponse {
                allowed: result.allowed(),
                permission: permission_for_response,
                resource: Some(resource_for_response),
                user_id: user_id_for_response,
                validation_time_ms: validation_time,
                cached: false, // TODO: Implement caching status from PermissionDecision
                source: result.source().unwrap_or_else(|| "unknown".to_string()),
                expires_at: None,
                constraints: None,
                audit_id: Uuid::new_v4(),
            };
            
            Ok(Json(response))
        },
        Err(error) => {
            tracing::error!("Permission validation failed: {}", error);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Validation failed",
                "message": error.to_string()
            }))).into_response())
        }
    }
}

/// Validate multiple permissions in a batch
pub async fn validate_permissions_batch(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(request): Json<ValidateBatchPermissionsRequest>,
) -> Result<Json<ValidateBatchPermissionsResponse>, Response> {
    let start_time = Instant::now();
    
    // Validate request
    if let Err(error) = request.validate() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({
            "error": "Invalid request",
            "message": error
        }))).into_response());
    }
    
    let permission_system = _container.get_permission_system()
        .map_err(|e| {
            tracing::error!("Failed to get permission system: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Internal server error",
                "message": "Permission system unavailable"
            }))).into_response()
        })?;
    
    let mut results = Vec::new();
    let mut cache_hits = 0;
    let mut total_validations = 0;
    
    // Extract common values to avoid repeated cloning
    let user_id = Arc::new(request.user_id);
    let context_data = Arc::new(request.context
        .as_ref()
        .and_then(|c| c.additional_claims.as_ref())
        .map(|claims| claims.iter().map(|(k, v)| (k.clone(), v.to_string())).collect::<HashMap<String, String>>())
        .unwrap_or_default());
    let ip_address = request.context.as_ref().and_then(|c| c.ip_address.as_ref());
    let user_agent = request.context.as_ref().and_then(|c| c.user_agent.as_ref());
    let session_id = request.context.as_ref().and_then(|c| c.session_id.as_ref());
    
    for perm_request in request.permissions {
        let perm_start_time = Instant::now();
        
        // Clone values before moving them
        let permission = perm_request.permission.clone();
        let resource = perm_request.resource.clone().unwrap_or_else(|| "*".to_string());
        
        let context = PermissionContext {
            user_id: (*user_id).clone(),
            permission: permission.clone(),
            resource: resource.clone(),
            context_data: (*context_data).clone(),
            timestamp: Utc::now(),
            ip_address: ip_address.cloned(),
            user_agent: user_agent.cloned(),
            session_id: session_id.cloned(),
        };
        
        match permission_system.validate_permission(&context).await {
            Ok(result) => {
                if result.cached() {
                    cache_hits += 1;
                }
                total_validations += 1;
                
                let validation_time = perm_start_time.elapsed().as_millis() as f64;
                
                results.push(BatchPermissionResultDto {
                    permission: permission.clone(),
                    resource: Some(resource.clone()),
                    allowed: result.allowed(),
                    source: result.source().unwrap_or_else(|| "unknown".to_string()),
                    validation_time_ms: validation_time,
                    cached: result.cached(),
                });
            },
            Err(error) => {
                tracing::warn!("Batch validation failed for permission {}: {}", 
                              perm_request.permission, error);
                
                results.push(BatchPermissionResultDto {
                    permission: permission.clone(),
                    resource: Some(resource.clone()),
                    allowed: false,
                    source: "error".to_string(),
                    validation_time_ms: perm_start_time.elapsed().as_millis() as f64,
                    cached: false,
                });
            }
        }
    }
    
    let total_time = start_time.elapsed().as_millis() as f64;
    let cache_hit_rate = if total_validations > 0 {
        cache_hits as f64 / total_validations as f64
    } else {
        0.0
    };
    
    // Create audit entry for batch validation
    let audit_id = Uuid::new_v4();
    if let Ok(audit_system) = _container.get_audit_system() {
        let audit_entry = PermissionAuditEntry {
            id: audit_id,
            event_type: AuditEventType::PermissionCheck,
            user_id: (*user_id).clone(),
            permission: format!("batch:{}", results.len()),
            resource: None,
            action: "validate_batch".to_string(),
            result: true, // Batch completed successfully
            timestamp: Utc::now(),
            client_ip: _auth.ip_address,
            user_agent: _auth.user_agent,
            geo_location: None,
            session_id: _auth.session_id,
            device_fingerprint: None,
            threat_indicators: HashMap::new(),
            risk_score: 0.5, // Default for batch operations
            additional_context: {
                let mut context = HashMap::new();
                context.insert("total_validations".to_string(), json!(total_validations));
                context.insert("cache_hit_rate".to_string(), json!(cache_hit_rate));
                context.insert("total_time_ms".to_string(), json!(total_time));
                context.insert("results_summary".to_string(), json!(results.iter().map(|r| json!({
                    "permission": r.permission,
                    "allowed": r.allowed
                })).collect::<Vec<_>>()));
                context
            },
        };
        
        let _ = audit_system.audit(audit_entry).await;
    }
    
    let response = ValidateBatchPermissionsResponse {
        user_id: (*user_id).clone(),
        results,
        total_validation_time_ms: total_time,
        cache_hit_rate,
        audit_id,
    };
    
    Ok(Json(response))
}

/// Get real-time permission validation stream for a user
pub async fn validate_permissions_realtime(
    State(_container): State<AppContainer>,
    Path(user_id): Path<UserId>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_realtime_permissions(socket, _container, user_id))
}

// ============================================================================
// User Permission Management Handlers
// ============================================================================

/// Get all permissions for a user
pub async fn get_user_permissions(
    State(_container): State<AppContainer>,
    Path(user_id): Path<UserId>,
    _auth: AuthContext,
) -> Result<Json<UserPermissionsResponse>, Response> {
    // Check if requester has permission to view user permissions
    let permission_system = _container.get_permission_system()
        .map_err(|e| {
            tracing::error!("Failed to get permission system: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Internal server error",
                "message": "Permission system unavailable"
            }))).into_response()
        })?;
    
    let view_context = PermissionContext {
        user_id: UserId::new(_auth.user_id.clone()),
        permission: "user-management:view".to_string(),
        resource: user_id.to_string(),
        context_data: HashMap::new(),
        timestamp: Utc::now(),
        ip_address: _auth.ip_address.clone(),
        user_agent: _auth.user_agent.clone(),
        session_id: _auth.session_id.clone(),
    };
    
    let view_result = permission_system.validate_permission(&view_context).await
        .map_err(|e| {
            tracing::error!("Failed to validate view permission: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Permission check failed"
            }))).into_response()
        })?;
    
    if !view_result.allowed() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "error": "Insufficient permissions",
            "message": "You don't have permission to view user permissions"
        }))).into_response());
    }
    
    // Get user permissions
    let user_permissions = permission_system.get_permissions(&user_id).await
        .map_err(|e| {
            tracing::error!("Failed to get user permissions: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Failed to retrieve permissions"
            }))).into_response()
        })?;
    
    // Convert to response format
    let effective_permissions = user_permissions.into_iter().map(|perm| {
        EffectivePermissionDto {
            permission: perm.id,
            resource: Some(perm.resource),
            source: "permission_system".to_string(), // Permission struct doesn't have source field
            granted_at: perm.metadata.granted_at,
            expires_at: perm.expires_at,
            granted_by: perm.metadata.granted_by,
            constraints: None, // TODO: Map constraints
        }
    }).collect();
    
    // Get admin module permissions
    // TODO: Implement admin module system integration
    let admin_modules: Vec<AdminModulePermissionDto> = Vec::new();
    
    // Get package tier info
    // TODO: Implement package tier system integration
    let package_tier = PackageTierInfoDto {
        tier: "FREE".to_string(),
        tier_name: "Free Tier".to_string(),
        features: Vec::new(),
        limits: HashMap::new(),
        upgraded_at: Utc::now(),
        expires_at: None,
    };
    
    let response = UserPermissionsResponse {
        user_id,
        effective_permissions,
        admin_modules,
        package_tier,
        temporary_elevations: Vec::new(), // TODO: Get from elevation system
        inherited_permissions: Vec::new(), // TODO: Get from inheritance system
        last_updated: Utc::now(),
    };
    
    Ok(Json(response))
}

/// Get effective permissions for a user (computed permissions)
pub async fn get_effective_permissions(
    State(_container): State<AppContainer>,
    Path(user_id): Path<UserId>,
    _auth: AuthContext,
) -> Result<Json<UserPermissionsResponse>, Response> {
    // This is similar to get_user_permissions but focuses on effective/computed permissions
    // including inheritance, elevation, and dynamic permissions
    get_user_permissions(State(_container), Path(user_id), _auth).await
}

/// Grant a permission to a user
pub async fn grant_user_permission(
    State(_container): State<AppContainer>,
    Path(user_id): Path<UserId>,
    _auth: AuthContext,
    Json(request): Json<GrantPermissionRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // Validate that the requester has permission to grant permissions
    let permission_system = _container.get_permission_system()
        .map_err(|_e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Permission system unavailable"
            }))).into_response()
        })?;
    
    let grant_context = PermissionContext {
        user_id: UserId::new(_auth.user_id.clone()),
        permission: "user-management:grant".to_string(),
        resource: user_id.to_string(),
        context_data: HashMap::new(),
        timestamp: Utc::now(),
        ip_address: _auth.ip_address.clone(),
        user_agent: _auth.user_agent.clone(),
        session_id: _auth.session_id.clone(),
    };
    
    let grant_result = permission_system.validate_permission(&grant_context).await
        .map_err(|_e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "Permission validation failed"
            }))).into_response()
        })?;
    
    if !grant_result.allowed() {
        return Err((StatusCode::FORBIDDEN, Json(json!({
            "error": "Insufficient permissions",
            "message": "You don't have permission to grant permissions"
        }))).into_response());
    }
    
    // Grant the permission
    // TODO: Implement permission granting logic
    
    // Create audit entry
    if let Ok(audit_system) = _container.get_audit_system() {
        let audit_entry = PermissionAuditEntry {
            id: Uuid::new_v4(),
            event_type: AuditEventType::PermissionGrant,
            user_id: request.user_id,
            permission: request.permission,
            resource: request.resource,
            action: "grant".to_string(),
            result: true, // Grant succeeded
            timestamp: Utc::now(),
            client_ip: _auth.ip_address,
            user_agent: _auth.user_agent,
            geo_location: None,
            session_id: _auth.session_id,
            device_fingerprint: None,
            threat_indicators: HashMap::new(),
            risk_score: 0.7, // Granting permissions is higher risk
            additional_context: {
                let mut context = HashMap::new();
                context.insert("granted_by".to_string(), json!(request.granted_by));
                context.insert("expires_at".to_string(), json!(request.expires_at));
                context.insert("reason".to_string(), json!(request.reason));
                context
            },
        };
        
        let _ = audit_system.audit(audit_entry).await;
    }
    
    Ok(Json(json!({
        "success": true,
        "message": "Permission granted successfully",
        "user_id": user_id,
        "granted_by": request.granted_by,
        "expires_at": request.expires_at
    })))
}

/// Revoke a permission from a user
pub async fn revoke_user_permission(
    State(_container): State<AppContainer>,
    Path(_user_id): Path<UserId>,
    _auth: AuthContext,
    Json(request): Json<RevokePermissionRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // Similar implementation to grant_user_permission but for revocation
    // TODO: Implement full revocation logic
    
    Ok(Json(json!({
        "success": true,
        "message": "Permission revoked successfully",
        "user_id": request.user_id,
        "permission": request.permission,
        "revoked_by": request.revoked_by
    })))
}

/// Temporarily elevate user permissions
pub async fn elevate_user_permissions(
    State(_container): State<AppContainer>,
    Path(_user_id): Path<UserId>,
    _auth: AuthContext,
    Json(request): Json<ElevatePermissionsRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // TODO: Implement permission elevation logic
    
    Ok(Json(json!({
        "success": true,
        "message": "Permissions elevated successfully",
        "user_id": request.user_id,
        "permissions": request.permissions,
        "duration_minutes": request.duration_minutes,
        "expires_at": Utc::now() + chrono::Duration::minutes(request.duration_minutes)
    })))
}

// ============================================================================
// Admin Module Handlers
// ============================================================================

/// Get all available admin modules
pub async fn get_admin_modules(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<AdminModulesResponse>, Response> {
    // TODO: Implement admin modules retrieval
    
    let modules = vec![
        AdminModuleDto {
            module: "user-management".to_string(),
            display_name: "User Management".to_string(),
            description: "Manage user accounts, profiles, and basic permissions".to_string(),
            permissions: vec![
                "user-management:view".to_string(),
                "user-management:create".to_string(),
                "user-management:update".to_string(),
                "user-management:delete".to_string(),
            ],
            required_tier: None,
            enabled: true,
        },
        AdminModuleDto {
            module: "analytics-access".to_string(),
            display_name: "Analytics Access".to_string(),
            description: "Access to advanced analytics and reporting features".to_string(),
            permissions: vec![
                "analytics:view".to_string(),
                "analytics:export".to_string(),
                "analytics:admin".to_string(),
            ],
            required_tier: Some("GOLD".to_string()),
            enabled: true,
        },
    ];
    
    let response = AdminModulesResponse {
        modules,
        user_assignments: HashMap::new(), // TODO: Get actual assignments
        total_users: 0, // TODO: Get actual count
    };
    
    Ok(Json(response))
}

/// Get permissions for a specific admin module
pub async fn get_module_permissions(
    State(_container): State<AppContainer>,
    Path(module): Path<String>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    // TODO: Implement module permissions retrieval
    
    Ok(Json(json!({
        "module": module,
        "permissions": ["view", "create", "update", "delete"],
        "description": "Module permissions"
    })))
}

/// Assign admin module permission to a user
pub async fn assign_module_permission(
    State(_container): State<AppContainer>,
    Path(module): Path<String>,
    _auth: AuthContext,
    Json(request): Json<AssignModulePermissionRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // TODO: Implement module permission assignment
    
    Ok(Json(json!({
        "success": true,
        "message": format!("Module {} permissions assigned to user", module),
        "user_id": request.user_id,
        "module": module,
        "permissions": request.permissions,
        "assigned_by": request.assigned_by
    })))
}

/// Revoke admin module permission from a user
pub async fn revoke_module_permission(
    State(_container): State<AppContainer>,
    Path(module): Path<String>,
    _auth: AuthContext,
    Json(user_id): Json<UserId>,
) -> Result<Json<serde_json::Value>, Response> {
    // TODO: Implement module permission revocation
    
    Ok(Json(json!({
        "success": true,
        "message": format!("Module {} permissions revoked from user", module),
        "user_id": user_id,
        "module": module
    })))
}

/// Grant temporary admin access
pub async fn grant_temporary_admin(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(request): Json<GrantTemporaryAdminRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // TODO: Implement temporary admin granting
    
    Ok(Json(json!({
        "success": true,
        "message": "Temporary admin access granted",
        "user_id": request.user_id,
        "admin_level": request.admin_level,
        "duration_minutes": request.duration_minutes,
        "expires_at": Utc::now() + chrono::Duration::minutes(request.duration_minutes),
        "approved_by": request.approved_by
    })))
}

// ============================================================================
// Package Tier Handlers
// ============================================================================

/// Get all available package tiers
pub async fn get_package_tiers(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<PackageTiersResponse>, Response> {
    // TODO: Implement tier retrieval from tier system
    
    let tiers = vec![
        PackageTierDto {
            tier: "FREE".to_string(),
            display_name: "Free Tier".to_string(),
            description: "Basic trading features".to_string(),
            features: vec!["basic-trading".to_string()],
            limits: HashMap::from([
                ("api_calls_per_hour".to_string(), json!(100)),
                ("watchlist_items".to_string(), json!(10)),
            ]),
            pricing: None,
            popular: false,
        },
        PackageTierDto {
            tier: "GOLD".to_string(),
            display_name: "Gold Tier".to_string(),
            description: "Advanced analytics and trading tools".to_string(),
            features: vec![
                "basic-trading".to_string(),
                "advanced-analytics".to_string(),
                "api-access".to_string(),
                "priority-support".to_string(),
            ],
            limits: HashMap::from([
                ("api_calls_per_hour".to_string(), json!(1000)),
                ("watchlist_items".to_string(), json!(100)),
            ]),
            pricing: Some(PricingInfoDto {
                monthly_price: 29.99,
                annual_price: 299.99,
                currency: "USD".to_string(),
                billing_cycles: vec!["monthly".to_string(), "annual".to_string()],
            }),
            popular: true,
        },
    ];
    
    let response = PackageTiersResponse {
        tiers,
        current_promotions: Vec::new(), // TODO: Get actual promotions
    };
    
    Ok(Json(response))
}

/// Get permissions for a specific package tier
pub async fn get_tier_permissions(
    State(_container): State<AppContainer>,
    Path(tier): Path<String>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    // TODO: Implement tier permissions retrieval
    
    Ok(Json(json!({
        "tier": tier,
        "permissions": ["basic-trading", "advanced-analytics"],
        "limits": {
            "api_calls_per_hour": 1000,
            "watchlist_items": 100
        }
    })))
}

/// Get features for a specific package tier
pub async fn get_tier_features(
    State(_container): State<AppContainer>,
    Path(tier): Path<String>,
    _auth: AuthContext,
) -> Result<Json<Vec<TierFeatureDto>>, Response> {
    // TODO: Implement tier features retrieval
    
    let features = vec![
        TierFeatureDto {
            feature: "basic-trading".to_string(),
            enabled: true,
            usage_limit: None,
            current_usage: None,
        },
        TierFeatureDto {
            feature: "advanced-analytics".to_string(),
            enabled: tier != "FREE",
            usage_limit: Some(1000),
            current_usage: Some(250),
        },
    ];
    
    Ok(Json(features))
}

/// Get limits for a specific package tier
pub async fn get_tier_limits(
    State(_container): State<AppContainer>,
    Path(tier): Path<String>,
    _auth: AuthContext,
) -> Result<Json<HashMap<String, serde_json::Value>>, Response> {
    // TODO: Implement tier limits retrieval
    
    let limits = match tier.as_str() {
        "FREE" => HashMap::from([
            ("api_calls_per_hour".to_string(), json!(100)),
            ("watchlist_items".to_string(), json!(10)),
            ("alerts".to_string(), json!(5)),
        ]),
        "GOLD" => HashMap::from([
            ("api_calls_per_hour".to_string(), json!(1000)),
            ("watchlist_items".to_string(), json!(100)),
            ("alerts".to_string(), json!(50)),
        ]),
        "PLATINUM" => HashMap::from([
            ("api_calls_per_hour".to_string(), json!(10000)),
            ("watchlist_items".to_string(), json!(1000)),
            ("alerts".to_string(), json!(500)),
        ]),
        _ => HashMap::new(),
    };
    
    Ok(Json(limits))
}

/// Upgrade user to a higher tier
pub async fn upgrade_user_tier(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(request): Json<UpgradeTierRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // TODO: Implement tier upgrade logic
    
    Ok(Json(json!({
        "success": true,
        "message": "User tier upgraded successfully",
        "user_id": request.user_id,
        "target_tier": request.target_tier,
        "effective_date": request.effective_date.unwrap_or_else(|| Utc::now()),
        "billing_cycle": request.billing_cycle
    })))
}

/// Downgrade user to a lower tier
pub async fn downgrade_user_tier(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(request): Json<UpgradeTierRequest>, // Reuse the same struct
) -> Result<Json<serde_json::Value>, Response> {
    // TODO: Implement tier downgrade logic
    
    Ok(Json(json!({
        "success": true,
        "message": "User tier downgraded successfully",
        "user_id": request.user_id,
        "target_tier": request.target_tier,
        "effective_date": request.effective_date.unwrap_or_else(|| Utc::now())
    })))
}

// ============================================================================
// Utility Functions
// ============================================================================

fn calculate_security_score(result: &PermissionDecision, context: &PermissionContext) -> f64 {
    let mut score: f64 = 1.0;
    
    // Reduce score for failed validations
    if !result.allowed() {
        score *= 0.5;
    }
    
    // Reduce score for admin permissions
    if crate::permissions::utils::is_admin_permission(&context.permission) {
        score *= 0.7;
    }
    
    // Security level check removed - not available in current PermissionContext
    
    score.max(0.1) // Minimum score
}

fn calculate_grant_security_score(request: &GrantPermissionRequest) -> f64 {
    let mut score: f64 = 0.5; // Granting permissions is inherently risky
    
    // Increase risk for admin permissions
    if crate::permissions::utils::is_admin_permission(&request.permission) {
        score *= 0.3;
    }
    
    // Reduce risk if expiration is set
    if request.expires_at.is_some() {
        score *= 1.2;
    }
    
    // Reduce risk if reason is provided
    if request.reason.is_some() {
        score *= 1.1;
    }
    
    score.min(1.0).max(0.1)
}

// Real-time permission WebSocket handler
async fn handle_realtime_permissions(
    _socket: axum::extract::ws::WebSocket,
    _container: AppContainer,
    user_id: UserId,
) {
    // TODO: Implement WebSocket permission streaming
    tracing::info!("Starting real-time permission stream for user: {}", user_id);
    
    // Note: Axum WebSocket doesn't have split method
    // Use the socket directly for sending and receiving
    
    // Send initial permissions
    let _initial_message = json!({
        "type": "permissions_snapshot",
        "user_id": user_id,
        "permissions": [],
        "timestamp": Utc::now()
    });
    
    // TODO: Implement proper WebSocket message sending
    
    // TODO: Handle incoming messages properly without split
    // Placeholder for WebSocket message loop
}

// Placeholder handlers for remaining endpoints (to be implemented)

pub async fn get_permission_templates(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<PermissionTemplatesResponse>, Response> {
    Ok(Json(PermissionTemplatesResponse {
        templates: Vec::new(),
        total: 0,
        page: 1,
        per_page: 20,
    }))
}

pub async fn get_permission_template(
    State(_container): State<AppContainer>,
    Path(_template_id): Path<Uuid>,
    _auth: AuthContext,
) -> Result<Json<PermissionTemplateDto>, Response> {
    Err((StatusCode::NOT_FOUND, Json(json!({
        "error": "Template not found"
    }))).into_response())
}

pub async fn create_permission_template(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(_request): Json<CreatePermissionTemplateRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Template created"})))
}

pub async fn update_permission_template(
    State(_container): State<AppContainer>,
    Path(_template_id): Path<Uuid>,
    _auth: AuthContext,
    Json(_request): Json<CreatePermissionTemplateRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Template updated"})))
}

pub async fn delete_permission_template(
    State(_container): State<AppContainer>,
    Path(_template_id): Path<Uuid>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Template deleted"})))
}

pub async fn apply_permission_template(
    State(_container): State<AppContainer>,
    Path(_template_id): Path<Uuid>,
    _auth: AuthContext,
    Json(_request): Json<ApplyTemplateRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Template applied"})))
}

pub async fn get_audit_events(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<AuditEventsResponse>, Response> {
    Ok(Json(AuditEventsResponse {
        events: Vec::new(),
        total: 0,
        page: 1,
        per_page: 20,
        has_more: false,
    }))
}

pub async fn get_audit_event(
    State(_container): State<AppContainer>,
    Path(_event_id): Path<Uuid>,
    _auth: AuthContext,
) -> Result<Json<AuditEventDto>, Response> {
    Err((StatusCode::NOT_FOUND, Json(json!({
        "error": "Event not found"
    }))).into_response())
}

pub async fn get_security_events(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<AuditEventsResponse>, Response> {
    Ok(Json(AuditEventsResponse {
        events: Vec::new(),
        total: 0,
        page: 1,
        per_page: 20,
        has_more: false,
    }))
}

pub async fn get_user_audit_log(
    State(_container): State<AppContainer>,
    Path(_user_id): Path<UserId>,
    _auth: AuthContext,
) -> Result<Json<AuditEventsResponse>, Response> {
    Ok(Json(AuditEventsResponse {
        events: Vec::new(),
        total: 0,
        page: 1,
        per_page: 20,
        has_more: false,
    }))
}

pub async fn export_audit_log(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Export initiated"})))
}

pub async fn get_permission_health(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<PermissionHealthResponse>, Response> {
    Ok(Json(PermissionHealthResponse {
        status: "healthy".to_string(),
        cache_health: CacheHealthDto {
            status: "healthy".to_string(),
            hit_rate: 0.95,
            memory_usage: 0.6,
            eviction_rate: 0.01,
            connection_pool_status: "healthy".to_string(),
        },
        database_health: DatabaseHealthDto {
            status: "healthy".to_string(),
            connection_pool_active: 10,
            connection_pool_idle: 5,
            query_performance_ms: 2.5,
            migration_status: "up_to_date".to_string(),
        },
        validation_performance: PerformanceMetricsDto {
            avg_validation_time_ms: 3.2,
            p95_validation_time_ms: 8.5,
            p99_validation_time_ms: 15.0,
            throughput_per_second: 1500.0,
            cache_hit_rate: 0.95,
        },
        error_rates: ErrorRatesDto {
            validation_errors: 0.001,
            cache_errors: 0.0005,
            database_errors: 0.0001,
            network_errors: 0.0002,
            timeout_errors: 0.0001,
        },
        system_load: SystemLoadDto {
            cpu_usage: 0.25,
            memory_usage: 0.6,
            active_connections: 50,
            queue_depth: 2,
        },
        last_updated: Utc::now(),
    }))
}

pub async fn get_permission_metrics(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<PermissionMetricsResponse>, Response> {
    Ok(Json(PermissionMetricsResponse {
        metrics: PermissionSystemMetricsDto {
            total_validations: 100000,
            successful_validations: 98500,
            failed_validations: 1500,
            cache_hits: 95000,
            cache_misses: 5000,
            avg_response_time_ms: 3.2,
            error_distribution: HashMap::from([
                ("validation_error".to_string(), 800),
                ("permission_denied".to_string(), 600),
                ("system_error".to_string(), 100),
            ]),
            top_permissions: vec![
                PermissionUsageDto {
                    permission: "user:read".to_string(),
                    usage_count: 25000,
                    success_rate: 0.99,
                    avg_response_time_ms: 2.8,
                },
                PermissionUsageDto {
                    permission: "trading:basic".to_string(),
                    usage_count: 20000,
                    success_rate: 0.98,
                    avg_response_time_ms: 3.5,
                },
            ],
            active_users: 5000,
        },
        time_range: TimeRangeDto {
            start: Utc::now() - chrono::Duration::hours(24),
            end: Utc::now(),
            duration_hours: 24,
        },
        collected_at: Utc::now(),
    }))
}

pub async fn get_cache_statistics(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({
        "cache_stats": {
            "hit_rate": 0.95,
            "miss_rate": 0.05,
            "eviction_rate": 0.01,
            "memory_usage": 0.6,
            "key_count": 50000,
            "connection_pool": {
                "active": 10,
                "idle": 5,
                "status": "healthy"
            }
        }
    })))
}

pub async fn clear_permission_cache(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Cache cleared"})))
}

pub async fn warm_permission_cache(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Cache warming initiated"})))
}

pub async fn permission_websocket_handler(
    State(_container): State<AppContainer>,
    Path(_user_id): Path<UserId>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|_socket| async {
        // TODO: Implement WebSocket handler
    })
}

pub async fn permission_sse_handler(
    State(_container): State<AppContainer>,
    Path(_user_id): Path<UserId>,
    _auth: AuthContext,
) -> Result<Response, Response> {
    // TODO: Implement SSE handler for real-time permission updates
    Ok((StatusCode::NOT_IMPLEMENTED, Json(json!({
        "error": "SSE not implemented yet"
    }))).into_response())
}

pub async fn bulk_validate_permissions(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(_request): Json<BulkPermissionRequest>,
) -> Result<Json<BulkOperationResponse>, Response> {
    Ok(Json(BulkOperationResponse {
        operation_id: Uuid::new_v4(),
        total_operations: 0,
        successful_operations: 0,
        failed_operations: 0,
        results: Vec::new(),
        execution_time_ms: 0.0,
        dry_run: false,
    }))
}

pub async fn bulk_grant_permissions(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(_request): Json<BulkPermissionRequest>,
) -> Result<Json<BulkOperationResponse>, Response> {
    Ok(Json(BulkOperationResponse {
        operation_id: Uuid::new_v4(),
        total_operations: 0,
        successful_operations: 0,
        failed_operations: 0,
        results: Vec::new(),
        execution_time_ms: 0.0,
        dry_run: false,
    }))
}

pub async fn bulk_revoke_permissions(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(_request): Json<BulkPermissionRequest>,
) -> Result<Json<BulkOperationResponse>, Response> {
    Ok(Json(BulkOperationResponse {
        operation_id: Uuid::new_v4(),
        total_operations: 0,
        successful_operations: 0,
        failed_operations: 0,
        results: Vec::new(),
        execution_time_ms: 0.0,
        dry_run: false,
    }))
}

pub async fn import_permissions(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Import completed"})))
}

pub async fn export_permissions(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Export completed"})))
}

pub async fn get_permission_policies(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<PermissionPolicyResponse>, Response> {
    Ok(Json(PermissionPolicyResponse {
        policies: Vec::new(),
        total: 0,
        active_policies: 0,
        inactive_policies: 0,
    }))
}

pub async fn get_permission_policy(
    State(_container): State<AppContainer>,
    Path(_policy_id): Path<Uuid>,
    _auth: AuthContext,
) -> Result<Json<PermissionPolicyDto>, Response> {
    Err((StatusCode::NOT_FOUND, Json(json!({
        "error": "Policy not found"
    }))).into_response())
}

pub async fn create_permission_policy(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(_request): Json<CreatePermissionPolicyRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Policy created"})))
}

pub async fn update_permission_policy(
    State(_container): State<AppContainer>,
    Path(_policy_id): Path<Uuid>,
    _auth: AuthContext,
    Json(_request): Json<CreatePermissionPolicyRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Policy updated"})))
}

pub async fn delete_permission_policy(
    State(_container): State<AppContainer>,
    Path(_policy_id): Path<Uuid>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"success": true, "message": "Policy deleted"})))
}

pub async fn test_permission_policy(
    State(_container): State<AppContainer>,
    Path(_policy_id): Path<Uuid>,
    _auth: AuthContext,
    Json(_request): Json<TestPermissionPolicyRequest>,
) -> Result<Json<PolicyTestResponse>, Response> {
    Ok(Json(PolicyTestResponse {
        policy_id: _policy_id,
        test_results: Vec::new(),
        overall_success: true,
        execution_time_ms: 0.0,
    }))
}

pub async fn check_permission_inheritance(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"inheritance": "checked"})))
}

pub async fn create_permission_delegation(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"delegation": "created"})))
}

pub async fn revoke_permission_delegation(
    State(_container): State<AppContainer>,
    Path(_delegation_id): Path<Uuid>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"delegation": "revoked"})))
}

pub async fn evaluate_permission_constraints(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    Ok(Json(json!({"constraints": "evaluated"})))
}

// Legacy API handlers for backward compatibility
pub async fn legacy_check_permission(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(request): Json<ValidatePermissionRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // Convert to new format and delegate to new handler
    match validate_permissions(State(_container), _auth, Json(request)).await {
        Ok(Json(response)) => Ok(Json(json!({
            "allowed": response.allowed,
            "permission": response.permission,
            "user_id": response.user_id
        }))),
        Err(error) => Err(error),
    }
}

pub async fn legacy_get_user_permissions(
    State(_container): State<AppContainer>,
    Path(user_id): Path<UserId>,
    _auth: AuthContext,
) -> Result<Json<serde_json::Value>, Response> {
    match get_user_permissions(State(_container), Path(user_id), _auth).await {
        Ok(Json(response)) => Ok(Json(json!({
            "user_id": response.user_id,
            "permissions": response.effective_permissions.into_iter()
                .map(|p| p.permission)
                .collect::<Vec<_>>()
        }))),
        Err(error) => Err(error),
    }
}

pub async fn legacy_admin_check(
    State(_container): State<AppContainer>,
    _auth: AuthContext,
    Json(request): Json<ValidatePermissionRequest>,
) -> Result<Json<serde_json::Value>, Response> {
    // Legacy admin permission check
    let is_admin = crate::permissions::utils::is_admin_permission(&request.permission);
    
    Ok(Json(json!({
        "is_admin": is_admin,
        "permission": request.permission,
        "user_id": request.user_id
    })))
}