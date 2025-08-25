// Permission API Routes Configuration
//
// Defines the routing structure for the unified permission validation API,
// organizing endpoints into logical groups with proper middleware, validation,
// and documentation for enterprise-grade permission management.

use axum::{
    Router,
    routing::{get, post, put, delete},
    middleware::{self, Next},
    http::{Request, StatusCode},
    response::Response,
    body::Body,
};
// use tower::ServiceBuilder;
// use tower_http::{
//     cors::CorsLayer,
//     compression::CompressionLayer,
//     trace::TraceLayer,
//     timeout::TimeoutLayer,
//     limit::RequestBodyLimitLayer,
// };
// use std::time::Duration;

use crate::{
    infra::container::AppContainer,
    web::middleware::add_deprecation_headers,
    // web::middleware::{
        // auth_monitoring::AuthContext,
        // rate_limiter::RateLimiter,
    // },
};

use super::{handlers, middleware as perm_middleware};

/// Create the main permission API router with all middleware and routes
pub fn create_permission_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Core permission validation routes
        .nest("/validate", create_validation_routes(_container))
        
        // User permission management routes
        .nest("/user", create_user_permission_routes(_container))
        
        // Admin module routes
        .nest("/admin", create_admin_routes(_container))
        
        // Package tier routes
        .nest("/tiers", create_tier_routes(_container))
        
        // Template management routes
        .nest("/templates", create_template_routes(_container))
        
        // Audit and monitoring routes
        .nest("/audit", create_audit_routes(_container))
        
        // System health and metrics routes
        .nest("/health", create_health_routes(_container))
        .nest("/metrics", create_metrics_routes(_container))
        
        // Cache management routes
        .nest("/cache", create_cache_routes(_container))
        
        // Real-time communication routes
        .nest("/realtime", create_realtime_routes(_container))
        
        // Bulk operations routes
        .nest("/bulk", create_bulk_routes(_container))
        
        // Policy management routes
        .nest("/policies", create_policy_routes(_container))
        
        // Advanced features routes
        .nest("/advanced", create_advanced_routes(_container))
        
        // Simplified middleware - complex stack commented out for compilation
        // .layer(
        //     ServiceBuilder::new()
        //         .layer(TimeoutLayer::new(Duration::from_secs(30)))
        //         .layer(CompressionLayer::new())
        //         .layer(TraceLayer::new_for_http())
        // )
}

/// Permission validation routes
fn create_validation_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Single permission validation
        .route("/", post(handlers::validate_permissions))
        
        // Batch permission validation
        .route("/batch", post(handlers::validate_permissions_batch))
        
        // Real-time permission validation
        .route("/realtime/:user_id", get(handlers::validate_permissions_realtime))
        
        // Permission inheritance check
        .route("/inheritance", post(handlers::check_permission_inheritance))
        
        // Constraint evaluation
        .route("/constraints", post(handlers::evaluate_permission_constraints))
        
        .with_state(_container.clone())
}

/// User permission management routes
fn create_user_permission_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Get user permissions
        .route("/:user_id", get(handlers::get_user_permissions))
        
        // Get effective permissions (computed)
        .route("/:user_id/effective", get(handlers::get_effective_permissions))
        
        // Grant permission to user
        .route("/:user_id/grant", post(handlers::grant_user_permission))
        
        // Revoke permission from user
        .route("/:user_id/revoke", delete(handlers::revoke_user_permission))
        
        // Temporarily elevate user permissions
        .route("/:user_id/elevate", post(handlers::elevate_user_permissions))
        
        // Get user audit log
        .route("/:user_id/audit", get(handlers::get_user_audit_log))
        
        .with_state(_container.clone())
}

/// Admin module management routes
fn create_admin_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Get all admin modules
        .route("/modules", get(handlers::get_admin_modules))
        
        // Get specific module permissions
        .route("/modules/:module", get(handlers::get_module_permissions))
        
        // Assign module permission to user
        .route("/modules/:module/assign", post(handlers::assign_module_permission))
        
        // Revoke module permission from user
        .route("/modules/:module/revoke", delete(handlers::revoke_module_permission))
        
        // Grant temporary admin access
        .route("/grant-temporary", post(handlers::grant_temporary_admin))
        
        .layer(middleware::from_fn(perm_middleware::admin_only_middleware))
        .with_state(_container.clone())
}

/// Package tier management routes
fn create_tier_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Get all package tiers
        .route("/", get(handlers::get_package_tiers))
        
        // Get specific tier permissions
        .route("/:tier", get(handlers::get_tier_permissions))
        
        // Get tier features
        .route("/:tier/features", get(handlers::get_tier_features))
        
        // Get tier limits
        .route("/:tier/limits", get(handlers::get_tier_limits))
        
        // Upgrade user tier
        .route("/upgrade", post(handlers::upgrade_user_tier))
        
        // Downgrade user tier
        .route("/downgrade", post(handlers::downgrade_user_tier))
        
        .with_state(_container.clone())
}

/// Permission template management routes
fn create_template_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Get all templates
        .route("/", get(handlers::get_permission_templates))
        
        // Create new template
        .route("/", post(handlers::create_permission_template))
        
        // Get specific template
        .route("/:template_id", get(handlers::get_permission_template))
        
        // Update template
        .route("/:template_id", put(handlers::update_permission_template))
        
        // Delete template
        .route("/:template_id", delete(handlers::delete_permission_template))
        
        // Apply template to users
        .route("/:template_id/apply", post(handlers::apply_permission_template))
        
        .layer(middleware::from_fn(perm_middleware::admin_only_middleware))
        .with_state(_container.clone())
}

/// Audit and monitoring routes
fn create_audit_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Get audit events
        .route("/events", get(handlers::get_audit_events))
        
        // Get specific audit event
        .route("/events/:event_id", get(handlers::get_audit_event))
        
        // Get security events
        .route("/security-events", get(handlers::get_security_events))
        
        // Export audit log
        .route("/export", post(handlers::export_audit_log))
        
        .layer(middleware::from_fn(perm_middleware::admin_only_middleware))
}

/// System health routes
fn create_health_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Overall permission system health
        .route("/", get(handlers::get_permission_health))
        
        // Detailed health check
        .route("/detailed", get(handlers::get_permission_health))
        
        .layer(middleware::from_fn(perm_middleware::health_check_middleware))
}

/// Metrics and monitoring routes
fn create_metrics_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // General metrics
        .route("/", get(handlers::get_permission_metrics))
        
        // Performance metrics
        .route("/performance", get(handlers::get_permission_metrics))
        
        // Usage statistics
        .route("/usage", get(handlers::get_permission_metrics))
        
        .layer(middleware::from_fn(perm_middleware::metrics_middleware))
}

/// Cache management routes
fn create_cache_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Cache statistics
        .route("/stats", get(handlers::get_cache_statistics))
        
        // Clear cache
        .route("/clear", post(handlers::clear_permission_cache))
        
        // Warm cache
        .route("/warm", post(handlers::warm_permission_cache))
        
        .layer(middleware::from_fn(perm_middleware::admin_only_middleware))
}

/// Real-time communication routes
fn create_realtime_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // WebSocket connection for real-time updates
        .route("/ws/:user_id", get(handlers::permission_websocket_handler))
        
        // Server-Sent Events for permission updates
        .route("/sse/:user_id", get(handlers::permission_sse_handler))
        
}

/// Bulk operations routes
fn create_bulk_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Bulk validate permissions
        .route("/validate", post(handlers::bulk_validate_permissions))
        
        // Bulk grant permissions
        .route("/grant", post(handlers::bulk_grant_permissions))
        
        // Bulk revoke permissions
        .route("/revoke", post(handlers::bulk_revoke_permissions))
        
        // Import permissions from file
        .route("/import", post(handlers::import_permissions))
        
        // Export permissions to file
        .route("/export", get(handlers::export_permissions))
        
        .layer(middleware::from_fn(perm_middleware::admin_only_middleware))
        .layer(middleware::from_fn(perm_middleware::bulk_operation_middleware))
}

/// Permission policy management routes
fn create_policy_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Get all policies
        .route("/", get(handlers::get_permission_policies))
        
        // Create new policy
        .route("/", post(handlers::create_permission_policy))
        
        // Get specific policy
        .route("/:policy_id", get(handlers::get_permission_policy))
        
        // Update policy
        .route("/:policy_id", put(handlers::update_permission_policy))
        
        // Delete policy
        .route("/:policy_id", delete(handlers::delete_permission_policy))
        
        // Test policy
        .route("/:policy_id/test", post(handlers::test_permission_policy))
        
        .layer(middleware::from_fn(perm_middleware::admin_only_middleware))
}

/// Advanced permission features routes
fn create_advanced_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        // Permission delegation
        .route("/delegation", post(handlers::create_permission_delegation))
        .route("/delegation/:delegation_id", delete(handlers::revoke_permission_delegation))
        
        // Permission inheritance analysis
        .route("/inheritance/analyze", post(handlers::check_permission_inheritance))
        
        // Dynamic permission evaluation
        .route("/dynamic/evaluate", post(handlers::evaluate_permission_constraints))
        
        .layer(middleware::from_fn(perm_middleware::admin_only_middleware))
}

/// Legacy API routes for backward compatibility
pub fn create_legacy_permission_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        .route("/check-permission", post(handlers::legacy_check_permission))
        .route("/user-permissions/:user_id", get(handlers::legacy_get_user_permissions))
        .route("/admin-check", post(handlers::legacy_admin_check))
        // Legacy unversioned routes that should use /api/v1/permissions instead
        .route("/permissions/check", post(handlers::legacy_check_permission))
        .route("/permissions/route", post(handlers::legacy_check_permission))
        .route("/permissions/bulk", post(handlers::validate_permissions_batch))
        .route("/permissions/single", get(handlers::legacy_get_user_permissions))
        .route("/permissions/navigation", get(handlers::legacy_get_user_permissions))
        .route("/permissions/features", get(handlers::legacy_get_user_permissions))
        .layer(middleware::from_fn(add_deprecation_headers))
        .layer(middleware::from_fn(legacy_middleware))
}

// ============================================================================
// Middleware Functions
// ============================================================================

/// Middleware to ensure only admins can access certain routes
async fn admin_only_middleware(
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement admin permission check
    // For now, allow all requests
    Ok(next.run(request).await)
}

/// Middleware for health check endpoints
async fn health_check_middleware(
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Add health check specific headers
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "X-Health-Check",
        "permission-system".parse().unwrap(),
    );
    Ok(response)
}

/// Middleware for metrics endpoints
async fn metrics_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Add metrics-specific headers
    let start_time = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start_time.elapsed();
    
    // Log metrics collection time
    tracing::debug!("Metrics collection took: {:?}", duration);
    
    Ok(response)
}

/// Middleware for bulk operations with additional validation
async fn bulk_operation_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Add bulk operation headers and validation
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "X-Bulk-Operation",
        "true".parse().unwrap(),
    );
    Ok(response)
}

/// Middleware for legacy API compatibility
async fn legacy_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Add legacy API headers
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "X-API-Version",
        "legacy".parse().unwrap(),
    );
    response.headers_mut().insert(
        "X-Deprecation-Warning",
        "This API version is deprecated. Please migrate to v1.".parse().unwrap(),
    );
    Ok(response)
}

// ============================================================================
// Route Groups and Organization
// ============================================================================

/// Create API version 1 routes with proper versioned paths
pub fn create_v1_permission_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        .nest("/api/v1/permissions", create_permission_routes(_container))
        .layer(middleware::from_fn(api_version_middleware::<1>))
}

/// Create API version 2 routes (future)
pub fn create_v2_permission_routes(_container: &AppContainer) -> Router<AppContainer> {
    Router::new()
        .nest("/api/v2/permissions", create_permission_routes(_container))
        .layer(middleware::from_fn(api_version_middleware::<2>))
}

/// API version middleware
async fn api_version_middleware<const VERSION: u8>(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "X-API-Version",
        format!("v{}", VERSION).parse().unwrap(),
    );
    Ok(response)
}

/// Create complete permission API with all versions
pub fn create_complete_permission_api(_container: &AppContainer) -> Router {
    Router::new()
        // Main versioned API routes (new structure)
        .merge(create_v1_permission_routes(_container))
        
        // Future API (v2) - placeholder  
        .merge(create_v2_permission_routes(_container))
        
        // Legacy API for backward compatibility
        .merge(create_legacy_permission_routes(_container))
        
        .with_state(_container.clone())
}

// ============================================================================
// Route Documentation and OpenAPI
// ============================================================================

/// Generate OpenAPI documentation for permission routes
pub fn generate_permission_api_docs() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": "EPSX Permission API",
            "version": "1.0.0",
            "description": "Unified Permission Validation System API for EPSX Trading Platform"
        },
        "servers": [
            {
                "url": "/api/v1/permissions",
                "description": "Permission API v1"
            }
        ],
        "paths": {
            "/validate": {
                "post": {
                    "summary": "Validate a single permission",
                    "description": "Validates whether a user has a specific permission",
                    "operationId": "validatePermission",
                    "tags": ["Permission Validation"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/ValidatePermissionRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Permission validation result",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ValidatePermissionResponse"
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "Bad request"
                        },
                        "401": {
                            "description": "Unauthorized"
                        },
                        "403": {
                            "description": "Forbidden"
                        },
                        "500": {
                            "description": "Internal server error"
                        }
                    }
                }
            },
            "/validate/batch": {
                "post": {
                    "summary": "Validate multiple permissions",
                    "description": "Validates multiple permissions for a user in a single request",
                    "operationId": "validateBatchPermissions",
                    "tags": ["Permission Validation"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/ValidateBatchPermissionsRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Batch permission validation results",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ValidateBatchPermissionsResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/user/{user_id}": {
                "get": {
                    "summary": "Get user permissions",
                    "description": "Retrieves all permissions for a specific user",
                    "operationId": "getUserPermissions",
                    "tags": ["User Permissions"],
                    "parameters": [
                        {
                            "name": "user_id",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string",
                                "format": "uuid"
                            },
                            "description": "The user ID"
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "User permissions",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/UserPermissionsResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "ValidatePermissionRequest": {
                    "type": "object",
                    "required": ["user_id", "permission"],
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "format": "uuid",
                            "description": "The user ID to validate permissions for"
                        },
                        "permission": {
                            "type": "string",
                            "description": "The permission to validate (format: module:action)"
                        },
                        "resource": {
                            "type": "string",
                            "description": "Optional resource identifier"
                        },
                        "context": {
                            "$ref": "#/components/schemas/PermissionContextDto"
                        }
                    }
                },
                "ValidatePermissionResponse": {
                    "type": "object",
                    "properties": {
                        "allowed": {
                            "type": "boolean",
                            "description": "Whether the permission is allowed"
                        },
                        "permission": {
                            "type": "string",
                            "description": "The validated permission"
                        },
                        "resource": {
                            "type": "string",
                            "description": "The resource identifier"
                        },
                        "user_id": {
                            "type": "string",
                            "format": "uuid",
                            "description": "The user ID"
                        },
                        "validation_time_ms": {
                            "type": "number",
                            "description": "Validation time in milliseconds"
                        },
                        "cached": {
                            "type": "boolean",
                            "description": "Whether the result was cached"
                        },
                        "source": {
                            "type": "string",
                            "description": "Source of the permission (admin_module, package_tier, etc.)"
                        },
                        "audit_id": {
                            "type": "string",
                            "format": "uuid",
                            "description": "Audit entry ID for this validation"
                        }
                    }
                },
                "PermissionContextDto": {
                    "type": "object",
                    "properties": {
                        "ip_address": {
                            "type": "string",
                            "description": "Client IP address"
                        },
                        "user_agent": {
                            "type": "string",
                            "description": "Client user agent"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Session identifier"
                        },
                        "security_level": {
                            "type": "string",
                            "enum": ["standard", "elevated", "admin"],
                            "description": "Required security level"
                        }
                    }
                }
            },
            "securitySchemes": {
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "security": [
            {
                "BearerAuth": []
            }
        ],
        "tags": [
            {
                "name": "Permission Validation",
                "description": "Core permission validation operations"
            },
            {
                "name": "User Permissions",
                "description": "User permission management"
            },
            {
                "name": "Admin Modules",
                "description": "Admin module permission management"
            },
            {
                "name": "Package Tiers",
                "description": "Package tier permission management"
            },
            {
                "name": "Templates",
                "description": "Permission template management"
            },
            {
                "name": "Audit",
                "description": "Audit and monitoring"
            },
            {
                "name": "System",
                "description": "System health and metrics"
            }
        ]
    })
}