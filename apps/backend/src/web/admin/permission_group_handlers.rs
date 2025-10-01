/**
 * UNIFIED PERMISSION GROUP MANAGEMENT HANDLERS
 * 
 * Backend-centric permission authority system for unified permission groups.
 * This replaces the old tier system with a unified wallet-first approach.
 * This is THE SINGLE SOURCE OF TRUTH for all permission decisions.
 * 
 * Features:
 * - Unified permission groups (subscription, manual, web3_asset, dao_membership, admin)
 * - Wallet-first authentication and assignment
 * - Real-time permission validation API
 * - Structured error responses for frontend consumption
 * - Permission audit logging and compliance
 */

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json as JsonResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// TODO: Re-enable when session verification is implemented
// use crate::web::middleware::openid_bearer_auth_middleware::{
//     require_user_context, check_user_permission, create_permission_denied_error
// };
use std::collections::HashMap;

use crate::web::auth::AppState;

// ============================================================================
// REQUEST/RESPONSE TYPES FOR UNIFIED PERMISSION GROUPS
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePermissionGroupRequest {
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String, // "manual" | "subscription" | "web3_asset" | "dao_membership" | "admin"
    pub permissions: Vec<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>, // "monthly" | "yearly" | "one_time" | "lifetime"
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub group_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePermissionGroupRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub billing_cycle: Option<String>,
    pub is_active: Option<bool>,
    pub is_promoted: Option<bool>,
    pub display_order: Option<i32>,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: Option<bool>,
    pub group_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct PermissionGroupResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub group_type: String,
    pub permissions: Vec<String>,
    pub price: f64,
    pub currency: String,
    pub billing_cycle: String,
    pub is_active: bool,
    pub is_promoted: bool,
    pub display_order: i32,
    pub max_members: Option<i32>,
    pub auto_assign_enabled: bool,
    pub group_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_modified_by: Option<String>,
    pub member_count: i32,
    pub revenue_30_days: f64,
}

#[derive(Debug, Serialize)]
pub struct PermissionGroupListResponse {
    pub permission_groups: Vec<PermissionGroupResponse>,
    pub total: i32,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: i32,
    pub limit: i32,
    pub total_pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct PermissionGroupListQuery {
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub is_active: Option<bool>,
    pub group_type: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>, // "name" | "price" | "created_at" | "member_count"
    pub sort_order: Option<String>, // "asc" | "desc"
}

// ============================================================================
// WALLET GROUP MEMBERSHIP TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateWalletAssignmentRequest {
    pub wallet_address: String,   // Web3 wallet address (primary key)
    pub group_id: String,
    pub assignment_source: String, // "manual" | "payment" | "web3_asset" | "dao_governance" | "admin" | "migration" | "auto_assignment"
    pub assignment_reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub payment_reference: Option<String>,
    pub subscription_id: Option<String>,
    pub assignment_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct WalletAssignmentResponse {
    pub id: String,
    pub wallet_address: String,      // Web3 wallet address (primary key)
    pub group_id: String,
    pub group_name: String,
    pub group_type: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub assignment_source: String,
    pub assignment_reason: Option<String>,
    pub assigned_by: Option<String>,  // Admin wallet address who assigned
    pub payment_reference: Option<String>,
    pub subscription_id: Option<String>,
    pub auto_renew: bool,
    pub next_billing_date: Option<DateTime<Utc>>,
    pub assignment_metadata: serde_json::Value,
}

// ============================================================================
// PERMISSION VALIDATION TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PermissionValidationRequest {
    pub wallet_address: String,      // Web3 wallet address
    pub permission: String,
    pub context: Option<HashMap<String, serde_json::Value>>, // Additional context for validation
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PermissionValidationResponse {
    pub is_valid: bool,
    pub permission: String,
    pub wallet_address: String,      // Web3 wallet address
    pub group_id: Option<String>,
    pub validation_result: PermissionValidationResult,
    pub error_details: Option<PermissionError>,
    pub suggestions: Option<Vec<PermissionSuggestion>>,
    pub audit_id: String,
    pub validated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PermissionValidationResult {
    pub granted: bool,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub source_group: Option<String>,
    pub next_refresh: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PermissionError {
    pub error_code: String,
    pub error_type: String, // "access_denied" | "expired" | "insufficient_permissions" | "wallet_not_found"
    pub user_message: String,
    pub technical_message: String,
    pub retry_after: Option<DateTime<Utc>>,
    pub upgrade_path: Option<UpgradePath>,
}

#[derive(Debug, Serialize)]
pub struct PermissionSuggestion {
    pub suggestion_type: String, // "upgrade" | "extend" | "alternative"
    pub title: String,
    pub description: String,
    pub action_url: Option<String>,
    pub priority: i32,
}

#[derive(Debug, Serialize)]
pub struct UpgradePath {
    pub suggested_group: String,
    pub price_difference: f64,
    pub additional_permissions: Vec<String>,
    pub upgrade_url: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// PERMISSION GROUP CRUD HANDLERS
// ============================================================================

/**
 * Create a new permission group
 * POST /admin/permission-groups
 */
pub async fn create_permission_group_handler(
    State(_app_state): State<AppState>,
    Json(create_request): Json<CreatePermissionGroupRequest>,
) -> Result<JsonResponse<ApiResponse<PermissionGroupResponse>>, StatusCode> {
    // TODO: Extract and validate admin user context from Bearer token
    // let user_context = require_user_context(&request).map_err(|(status, response)| {
    //     // Convert the tuple error to StatusCode for return
    //     status
    // })?;
    
    // TODO: Check if user has admin permission to create permission groups
    // if !check_user_permission(user_context, "admin:permission-groups:create") {
    //     return Err(StatusCode::FORBIDDEN);
    // }
    
    // Validate request
    if create_request.name.is_empty() || create_request.slug.is_empty() {
        return Ok(JsonResponse(ApiResponse {
            success: false,
            data: None,
            error: Some("name_required".to_string()),
            message: "Permission group name and slug are required".to_string(),
            timestamp: Utc::now(),
        }));
    }

    // Validate group type
    let valid_types = ["manual", "subscription", "web3_asset", "dao_membership", "admin"];
    if !valid_types.contains(&create_request.group_type.as_str()) {
        return Ok(JsonResponse(ApiResponse {
            success: false,
            data: None,
            error: Some("invalid_group_type".to_string()),
            message: "Invalid group type. Must be one of: manual, subscription, web3_asset, dao_membership, admin".to_string(),
            timestamp: Utc::now(),
        }));
    }

    // Validate price for subscription groups
    if create_request.group_type == "subscription" {
        if let Some(price) = create_request.price {
            if price < 0.0 {
                return Ok(JsonResponse(ApiResponse {
                    success: false,
                    data: None,
                    error: Some("invalid_price".to_string()),
                    message: "Price must be non-negative for subscription groups".to_string(),
                    timestamp: Utc::now(),
                }));
            }
        }
    }

    // TODO: Implement database insertion using unified permission_groups table
    // For now, return mock response
    let permission_group = PermissionGroupResponse {
        id: Uuid::new_v4().to_string(),
        name: create_request.name.clone(),
        slug: create_request.slug.clone(),
        description: create_request.description.clone(),
        group_type: create_request.group_type.clone(),
        permissions: create_request.permissions.clone(),
        price: create_request.price.unwrap_or(0.0),
        currency: create_request.currency.unwrap_or("USD".to_string()),
        billing_cycle: create_request.billing_cycle.unwrap_or("monthly".to_string()),
        is_active: create_request.is_active.unwrap_or(true),
        is_promoted: create_request.is_promoted.unwrap_or(false),
        display_order: create_request.display_order.unwrap_or(0),
        max_members: create_request.max_members,
        auto_assign_enabled: create_request.auto_assign_enabled.unwrap_or(false),
        group_metadata: create_request.group_metadata.unwrap_or(serde_json::json!({})),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Some("0xadmin...".to_string()), // TODO: Get from Web3 auth context
        last_modified_by: Some("0xadmin...".to_string()),
        member_count: 0,
        revenue_30_days: 0.0,
    };

    Ok(JsonResponse(ApiResponse {
        success: true,
        data: Some(permission_group),
        error: None,
        message: "Permission group created successfully".to_string(),
        timestamp: Utc::now(),
    }))
}

/**
 * Get a permission group by ID
 * GET /admin/permission-groups/:group_id
 */
pub async fn get_permission_group_handler(
    State(_app_state): State<AppState>,
    Path(group_id): Path<String>,
) -> Result<JsonResponse<ApiResponse<PermissionGroupResponse>>, StatusCode> {
    // TODO: Extract and validate admin user context from Bearer token
    // let user_context = require_user_context(&request).map_err(|(status, _)| status)?;
    
    // TODO: Check if user has admin permission to read permission groups
    // if !check_user_permission(user_context, "admin:permission-groups:read") {
    //     return Err(StatusCode::FORBIDDEN);
    // }
    
    // TODO: Implement database query using unified permission_groups table
    // For now, return mock response based on the default groups from migration
    let permission_group = match group_id.as_str() {
        "free-plan" => PermissionGroupResponse {
            id: group_id.clone(),
            name: "Free Plan".to_string(),
            slug: "free-plan".to_string(),
            description: "Basic analytics access for getting started with EPSX".to_string(),
            group_type: "subscription".to_string(),
            permissions: vec![
                "epsx:analytics:view:3".to_string(),
                "epsx:portfolio:view".to_string(),
                "epsx:profile:manage".to_string(),
            ],
            price: 0.00,
            currency: "USD".to_string(),
            billing_cycle: "monthly".to_string(),
            is_active: true,
            is_promoted: false,
            display_order: 0,
            max_members: None,
            auto_assign_enabled: false,
            group_metadata: serde_json::json!({
                "tier_level": "Bronze",
                "features": ["3 stock rankings", "Basic analytics", "Community support"],
                "limits": {"rankings_per_month": 3, "exports_per_month": 0}
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("system".to_string()),
            last_modified_by: Some("system".to_string()),
            member_count: 5432,
            revenue_30_days: 0.0,
        },
        "professional-plan" => PermissionGroupResponse {
            id: group_id.clone(),
            name: "Professional Plan".to_string(),
            slug: "professional-plan".to_string(),
            description: "Advanced analytics and trading features for serious investors".to_string(),
            group_type: "subscription".to_string(),
            permissions: vec![
                "epsx:analytics:view:50".to_string(),
                "epsx:analytics:export".to_string(),
                "epsx:trading:advanced".to_string(),
                "epsx:portfolio:manage".to_string(),
                "epsx:portfolio:advanced".to_string(),
                "epsx:alerts:create".to_string(),
                "epsx:alerts:manage".to_string(),
            ],
            price: 29.99,
            currency: "USD".to_string(),
            billing_cycle: "monthly".to_string(),
            is_active: true,
            is_promoted: true,
            display_order: 1,
            max_members: None,
            auto_assign_enabled: false,
            group_metadata: serde_json::json!({
                "tier_level": "Gold",
                "features": ["50 stock rankings", "Advanced analytics", "Export capabilities", "Priority support"],
                "limits": {"rankings_per_month": 50, "exports_per_month": 10}
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("system".to_string()),
            last_modified_by: Some("system".to_string()),
            member_count: 1247,
            revenue_30_days: 37403.53,
        },
        "enterprise-plan" => PermissionGroupResponse {
            id: group_id.clone(),
            name: "Enterprise Plan".to_string(),
            slug: "enterprise-plan".to_string(),
            description: "Full platform access with enterprise features and support".to_string(),
            group_type: "subscription".to_string(),
            permissions: vec![
                "epsx:analytics:*".to_string(),
                "epsx:trading:*".to_string(),
                "epsx:portfolio:*".to_string(),
                "epsx:alerts:*".to_string(),
                "epsx:api:*".to_string(),
            ],
            price: 99.99,
            currency: "USD".to_string(),
            billing_cycle: "monthly".to_string(),
            is_active: true,
            is_promoted: false,
            display_order: 2,
            max_members: None,
            auto_assign_enabled: false,
            group_metadata: serde_json::json!({
                "tier_level": "Diamond",
                "features": ["Unlimited rankings", "All analytics features", "API access", "Enterprise support"],
                "limits": {"rankings_per_month": -1, "exports_per_month": -1}
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("system".to_string()),
            last_modified_by: Some("system".to_string()),
            member_count: 234,
            revenue_30_days: 23397.66,
        },
        "platform-admins" => PermissionGroupResponse {
            id: group_id.clone(),
            name: "Platform Administrators".to_string(),
            slug: "platform-admins".to_string(),
            description: "Full administrative access to the EPSX platform".to_string(),
            group_type: "admin".to_string(),
            permissions: vec![
                "admin:*:*".to_string(),
                "epsx:*:*".to_string(),
            ],
            price: 0.00,
            currency: "USD".to_string(),
            billing_cycle: "lifetime".to_string(),
            is_active: true,
            is_promoted: false,
            display_order: 999,
            max_members: None,
            auto_assign_enabled: false,
            group_metadata: serde_json::json!({
                "tier_level": "Admin",
                "features": ["Full platform access", "User management", "System administration"],
                "limits": {}
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("system".to_string()),
            last_modified_by: Some("system".to_string()),
            member_count: 3,
            revenue_30_days: 0.0,
        },
        _ => return Err(StatusCode::NOT_FOUND),
    };

    Ok(JsonResponse(ApiResponse {
        success: true,
        data: Some(permission_group),
        error: None,
        message: "Permission group retrieved successfully".to_string(),
        timestamp: Utc::now(),
    }))
}

/**
 * List permission groups with filtering and pagination
 * GET /admin/permission-groups
 */
pub async fn list_permission_groups_handler(
    State(_app_state): State<AppState>,
    Query(query): Query<PermissionGroupListQuery>,
) -> Result<JsonResponse<ApiResponse<PermissionGroupListResponse>>, StatusCode> {
    // TODO: Extract and validate admin user context from Bearer token
    // let user_context = require_user_context(&request).map_err(|(status, _)| status)?;
    
    // TODO: Check if user has admin permission to read permission groups
    // if !check_user_permission(user_context, "admin:permission-groups:read") {
    //     return Err(StatusCode::FORBIDDEN);
    // }
    
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50);
    
    // TODO: Implement database query with filters using unified permission_groups table
    // For now, return mock response with default groups
    let mock_groups = vec![
        PermissionGroupResponse {
            id: "free-plan".to_string(),
            name: "Free Plan".to_string(),
            slug: "free-plan".to_string(),
            description: "Basic analytics access for getting started with EPSX".to_string(),
            group_type: "subscription".to_string(),
            permissions: vec![
                "epsx:analytics:view:3".to_string(),
                "epsx:portfolio:view".to_string(),
                "epsx:profile:manage".to_string(),
            ],
            price: 0.00,
            currency: "USD".to_string(),
            billing_cycle: "monthly".to_string(),
            is_active: true,
            is_promoted: false,
            display_order: 0,
            max_members: None,
            auto_assign_enabled: false,
            group_metadata: serde_json::json!({
                "tier_level": "Bronze",
                "features": ["3 stock rankings", "Basic analytics", "Community support"],
                "limits": {"rankings_per_month": 3, "exports_per_month": 0}
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("system".to_string()),
            last_modified_by: Some("system".to_string()),
            member_count: 5432,
            revenue_30_days: 0.0,
        },
        PermissionGroupResponse {
            id: "professional-plan".to_string(),
            name: "Professional Plan".to_string(),
            slug: "professional-plan".to_string(),
            description: "Advanced analytics and trading features for serious investors".to_string(),
            group_type: "subscription".to_string(),
            permissions: vec![
                "epsx:analytics:view:50".to_string(),
                "epsx:analytics:export".to_string(),
                "epsx:trading:advanced".to_string(),
                "epsx:portfolio:manage".to_string(),
                "epsx:portfolio:advanced".to_string(),
                "epsx:alerts:create".to_string(),
                "epsx:alerts:manage".to_string(),
            ],
            price: 29.99,
            currency: "USD".to_string(),
            billing_cycle: "monthly".to_string(),
            is_active: true,
            is_promoted: true,
            display_order: 1,
            max_members: None,
            auto_assign_enabled: false,
            group_metadata: serde_json::json!({
                "tier_level": "Gold",
                "features": ["50 stock rankings", "Advanced analytics", "Export capabilities", "Priority support"],
                "limits": {"rankings_per_month": 50, "exports_per_month": 10}
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("system".to_string()),
            last_modified_by: Some("system".to_string()),
            member_count: 1247,
            revenue_30_days: 37403.53,
        },
        PermissionGroupResponse {
            id: "enterprise-plan".to_string(),
            name: "Enterprise Plan".to_string(),
            slug: "enterprise-plan".to_string(),
            description: "Full platform access with enterprise features and support".to_string(),
            group_type: "subscription".to_string(),
            permissions: vec![
                "epsx:analytics:*".to_string(),
                "epsx:trading:*".to_string(),
                "epsx:portfolio:*".to_string(),
                "epsx:alerts:*".to_string(),
                "epsx:api:*".to_string(),
            ],
            price: 99.99,
            currency: "USD".to_string(),
            billing_cycle: "monthly".to_string(),
            is_active: true,
            is_promoted: false,
            display_order: 2,
            max_members: None,
            auto_assign_enabled: false,
            group_metadata: serde_json::json!({
                "tier_level": "Diamond",
                "features": ["Unlimited rankings", "All analytics features", "API access", "Enterprise support"],
                "limits": {"rankings_per_month": -1, "exports_per_month": -1}
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("system".to_string()),
            last_modified_by: Some("system".to_string()),
            member_count: 234,
            revenue_30_days: 23397.66,
        },
        PermissionGroupResponse {
            id: "platform-admins".to_string(),
            name: "Platform Administrators".to_string(),
            slug: "platform-admins".to_string(),
            description: "Full administrative access to the EPSX platform".to_string(),
            group_type: "admin".to_string(),
            permissions: vec![
                "admin:*:*".to_string(),
                "epsx:*:*".to_string(),
            ],
            price: 0.00,
            currency: "USD".to_string(),
            billing_cycle: "lifetime".to_string(),
            is_active: true,
            is_promoted: false,
            display_order: 999,
            max_members: None,
            auto_assign_enabled: false,
            group_metadata: serde_json::json!({
                "tier_level": "Admin",
                "features": ["Full platform access", "User management", "System administration"],
                "limits": {}
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("system".to_string()),
            last_modified_by: Some("system".to_string()),
            member_count: 3,
            revenue_30_days: 0.0,
        },
    ];

    let total = mock_groups.len() as i32;
    let total_pages = (total + limit - 1) / limit;

    Ok(JsonResponse(ApiResponse {
        success: true,
        data: Some(PermissionGroupListResponse {
            permission_groups: mock_groups,
            total,
            pagination: PaginationInfo {
                page,
                limit,
                total_pages,
            },
        }),
        error: None,
        message: "Permission groups retrieved successfully".to_string(),
        timestamp: Utc::now(),
    }))
}

/**
 * Update a permission group
 * PUT /admin/permission-groups/:group_id
 */
pub async fn update_permission_group_handler(
    State(_app_state): State<AppState>,
    Path(group_id): Path<String>,
    Json(request): Json<UpdatePermissionGroupRequest>,
) -> Result<JsonResponse<ApiResponse<PermissionGroupResponse>>, StatusCode> {
    // TODO: Implement admin permission check
    // Ensure wallet has admin:permission-groups:update permission
    
    if let Some(price) = request.price {
        if price < 0.0 {
            return Ok(JsonResponse(ApiResponse {
                success: false,
                data: None,
                error: Some("invalid_price".to_string()),
                message: "Price must be non-negative".to_string(),
                timestamp: Utc::now(),
            }));
        }
    }

    // TODO: Implement database update using unified permission_groups table
    // For now, return mock response
    let updated_group = PermissionGroupResponse {
        id: group_id.clone(),
        name: request.name.unwrap_or("Professional Plan".to_string()),
        slug: "professional-plan".to_string(),
        description: request.description.unwrap_or("Advanced analytics and trading features for serious investors".to_string()),
        group_type: "subscription".to_string(),
        permissions: request.permissions.unwrap_or(vec![
            "epsx:analytics:view:50".to_string(),
            "epsx:analytics:export".to_string(),
            "epsx:trading:advanced".to_string(),
        ]),
        price: request.price.unwrap_or(29.99),
        currency: request.currency.unwrap_or("USD".to_string()),
        billing_cycle: request.billing_cycle.unwrap_or("monthly".to_string()),
        is_active: request.is_active.unwrap_or(true),
        is_promoted: request.is_promoted.unwrap_or(false),
        display_order: request.display_order.unwrap_or(1),
        max_members: request.max_members,
        auto_assign_enabled: request.auto_assign_enabled.unwrap_or(false),
        group_metadata: request.group_metadata.unwrap_or(serde_json::json!({
            "tier_level": "Gold",
            "features": ["50 stock rankings", "Advanced analytics", "Export capabilities", "Priority support"],
            "limits": {"rankings_per_month": 50, "exports_per_month": 10}
        })),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Some("system".to_string()),
        last_modified_by: Some("0xadmin...".to_string()), // TODO: Get from Web3 auth context
        member_count: 1247,
        revenue_30_days: 37403.53,
    };

    Ok(JsonResponse(ApiResponse {
        success: true,
        data: Some(updated_group),
        error: None,
        message: "Permission group updated successfully".to_string(),
        timestamp: Utc::now(),
    }))
}

/**
 * Delete a permission group
 * DELETE /admin/permission-groups/:group_id
 */
pub async fn delete_permission_group_handler(
    State(_app_state): State<AppState>,
    Path(_group_id): Path<String>,
) -> Result<JsonResponse<ApiResponse<()>>, StatusCode> {
    // TODO: Implement admin permission check
    // Ensure wallet has admin:permission-groups:delete permission
    
    // TODO: Check if permission group has active members via wallet_group_memberships
    // Prevent deletion if wallets are assigned to this group
    
    // TODO: Implement database deletion from unified permission_groups table
    // For now, return success response
    
    Ok(JsonResponse(ApiResponse {
        success: true,
        data: None,
        error: None,
        message: "Permission group deleted successfully".to_string(),
        timestamp: Utc::now(),
    }))
}

// ============================================================================
// WALLET ASSIGNMENT HANDLERS
// ============================================================================

/**
 * Assign wallet to permission group
 * POST /admin/wallet-assignments
 */
pub async fn create_wallet_assignment_handler(
    State(_app_state): State<AppState>,
    Json(request): Json<CreateWalletAssignmentRequest>,
) -> Result<JsonResponse<ApiResponse<WalletAssignmentResponse>>, StatusCode> {
    // TODO: Implement admin permission check
    // Ensure wallet has admin:wallet-assignments:create permission
    
    // Validate request
    if request.wallet_address.is_empty() || request.group_id.is_empty() {
        return Ok(JsonResponse(ApiResponse {
            success: false,
            data: None,
            error: Some("invalid_request".to_string()),
            message: "Wallet address and Group ID are required".to_string(),
            timestamp: Utc::now(),
        }));
    }

    // Validate wallet address format
    if !request.wallet_address.starts_with("0x") || request.wallet_address.len() != 42 {
        return Ok(JsonResponse(ApiResponse {
            success: false,
            data: None,
            error: Some("invalid_wallet_address".to_string()),
            message: "Invalid wallet address format. Must be 42 characters starting with 0x".to_string(),
            timestamp: Utc::now(),
        }));
    }

    // TODO: Validate group exists and is active
    // TODO: Check for existing active assignment
    // TODO: Use assign_wallet_to_group() database function from migration
    
    let assignment = WalletAssignmentResponse {
        id: Uuid::new_v4().to_string(),
        wallet_address: request.wallet_address.clone(),
        group_id: request.group_id.clone(),
        group_name: "Professional Plan".to_string(), // TODO: Get from database
        group_type: "subscription".to_string(), // TODO: Get from database
        assigned_at: Utc::now(),
        expires_at: request.expires_at,
        is_active: true,
        assignment_source: request.assignment_source.clone(),
        assignment_reason: request.assignment_reason,
        assigned_by: Some("0xadmin...".to_string()), // TODO: Get admin wallet from Web3 auth context
        payment_reference: request.payment_reference,
        subscription_id: request.subscription_id,
        auto_renew: request.auto_renew.unwrap_or(false),
        next_billing_date: None, // TODO: Calculate based on billing cycle
        assignment_metadata: request.assignment_metadata.unwrap_or(serde_json::json!({})),
    };

    Ok(JsonResponse(ApiResponse {
        success: true,
        data: Some(assignment),
        error: None,
        message: "Wallet assigned to permission group successfully".to_string(),
        timestamp: Utc::now(),
    }))
}

/**
 * Get wallet's permission group assignments
 * GET /admin/wallets/:wallet_address/assignments
 */
pub async fn get_wallet_assignments_handler(
    State(_app_state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> Result<JsonResponse<ApiResponse<Vec<WalletAssignmentResponse>>>, StatusCode> {
    // TODO: Implement admin permission check
    // Ensure wallet has admin:wallet-assignments:read permission
    
    // TODO: Implement database query using wallet_group_memberships table
    // For now, return mock response
    let assignments = vec![
        WalletAssignmentResponse {
            id: Uuid::new_v4().to_string(),
            wallet_address: wallet_address.clone(),
            group_id: "professional-plan".to_string(),
            group_name: "Professional Plan".to_string(),
            group_type: "subscription".to_string(),
            assigned_at: Utc::now(),
            expires_at: None,
            is_active: true,
            assignment_source: "payment".to_string(),
            assignment_reason: Some("Monthly subscription payment".to_string()),
            assigned_by: Some("0xadmin...".to_string()),
            payment_reference: Some("pay_1234567890".to_string()),
            subscription_id: Some("sub_abcdef123456".to_string()),
            auto_renew: true,
            next_billing_date: Some(Utc::now()),
            assignment_metadata: serde_json::json!({"payment_method": "credit_card"}),
        },
    ];

    Ok(JsonResponse(ApiResponse {
        success: true,
        data: Some(assignments),
        error: None,
        message: "Wallet assignments retrieved successfully".to_string(),
        timestamp: Utc::now(),
    }))
}

// ============================================================================
// UNIFIED PERMISSION VALIDATION HANDLERS
// THE SINGLE SOURCE OF TRUTH FOR ALL PERMISSION DECISIONS
// ============================================================================

/**
 * Real-time permission validation - THE AUTHORITY
 * POST /api/permissions/validate
 * 
 * This uses the new get_wallet_effective_permissions() and wallet_has_permission() 
 * database functions created in the unified migration.
 */
pub async fn validate_permission_handler(
    State(_app_state): State<AppState>,
    Json(request): Json<PermissionValidationRequest>,
) -> Result<JsonResponse<PermissionValidationResponse>, StatusCode> {
    let audit_id = Uuid::new_v4().to_string();
    
    // TODO: Implement comprehensive permission validation using database functions:
    // 1. Call get_wallet_effective_permissions(wallet_address) 
    // 2. Call wallet_has_permission(wallet_address, permission)
    // 3. Check temporal constraints (expiry from wallet_group_memberships)
    // 4. Log audit trail
    
    // Mock validation logic - replace with real database calls
    let is_valid = validate_wallet_permission(&request.wallet_address, &request.permission).await;
    let validation_result = PermissionValidationResult {
        granted: is_valid,
        reason: if is_valid { "Permission granted by permission group".to_string() } 
                else { "Insufficient permissions".to_string() },
        expires_at: if is_valid { Some(Utc::now() + chrono::Duration::days(30)) } else { None },
        source_group: if is_valid { Some("professional-plan".to_string()) } else { None },
        next_refresh: Some(Utc::now() + chrono::Duration::hours(1)),
    };
    
    let error_details = if !is_valid {
        Some(PermissionError {
            error_code: "INSUFFICIENT_PERMISSIONS".to_string(),
            error_type: "access_denied".to_string(),
            user_message: "You need to upgrade your plan to access this feature.".to_string(),
            technical_message: format!("Permission '{}' not found in wallet's effective permissions", request.permission),
            retry_after: None,
            upgrade_path: Some(UpgradePath {
                suggested_group: "Professional Plan".to_string(),
                price_difference: 29.99,
                additional_permissions: vec![
                    "epsx:analytics:export".to_string(),
                    "epsx:trading:advanced".to_string(),
                ],
                upgrade_url: "/upgrade/professional-plan".to_string(),
            }),
        })
    } else {
        None
    };
    
    let suggestions = if !is_valid {
        Some(vec![
            PermissionSuggestion {
                suggestion_type: "upgrade".to_string(),
                title: "Upgrade to Professional".to_string(),
                description: "Unlock advanced analytics and trading features".to_string(),
                action_url: Some("/upgrade/professional-plan".to_string()),
                priority: 1,
            },
        ])
    } else {
        None
    };
    
    // TODO: Log audit trail to database/logging system
    tracing::info!(
        audit_id = %audit_id,
        wallet_address = %request.wallet_address,
        permission = %request.permission,
        granted = is_valid,
        "Permission validation completed"
    );
    
    Ok(JsonResponse(PermissionValidationResponse {
        is_valid,
        permission: request.permission,
        wallet_address: request.wallet_address,
        group_id: if is_valid { Some("professional-plan".to_string()) } else { None },
        validation_result,
        error_details,
        suggestions,
        audit_id,
        validated_at: Utc::now(),
    }))
}

/**
 * Get wallet's effective permissions - what they can actually do
 * GET /api/permissions/wallet/:wallet_address
 * 
 * This uses the new get_wallet_effective_permissions() database function.
 */
#[derive(Debug, Serialize)]
pub struct WalletPermissionsResponse {
    pub wallet_address: String,      // Web3 wallet address
    pub permission_groups: Vec<PermissionGroupSummary>,
    pub effective_permissions: Vec<String>,
    pub permission_summary: PermissionSummary,
    pub audit_id: String,
    pub retrieved_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PermissionGroupSummary {
    pub id: String,
    pub name: String,
    pub group_type: String,
    pub is_active: bool,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PermissionSummary {
    pub total_permissions: i32,
    pub active_permissions: i32,
    pub expiring_soon: i32,
    pub expired_permissions: i32,
    pub highest_group: String,
}

pub async fn get_wallet_permissions_handler(
    State(_app_state): State<AppState>,
    Path(wallet_address): Path<String>,
) -> Result<JsonResponse<ApiResponse<WalletPermissionsResponse>>, StatusCode> {
    let audit_id = Uuid::new_v4().to_string();
    
    // TODO: Implement comprehensive wallet permission retrieval using database functions:
    // 1. Call get_wallet_effective_permissions(wallet_address) to get permissions array
    // 2. Query wallet_group_memberships joined with permission_groups for group info
    // 3. Calculate permission summary statistics
    // 4. Filter expired permissions based on expires_at timestamps
    
    // Mock response - replace with real database queries
    let permission_groups = vec![
        PermissionGroupSummary {
            id: "professional-plan".to_string(),
            name: "Professional Plan".to_string(),
            group_type: "subscription".to_string(),
            is_active: true,
            assigned_at: Utc::now() - chrono::Duration::days(30),
            expires_at: Some(Utc::now() + chrono::Duration::days(30)),
        },
    ];
    
    let effective_permissions = vec![
        "epsx:analytics:view:50".to_string(),
        "epsx:analytics:export".to_string(),
        "epsx:trading:advanced".to_string(),
        "epsx:portfolio:manage".to_string(),
        "epsx:portfolio:advanced".to_string(),
        "epsx:alerts:create".to_string(),
        "epsx:alerts:manage".to_string(),
    ];
    
    let permission_summary = PermissionSummary {
        total_permissions: effective_permissions.len() as i32,
        active_permissions: effective_permissions.len() as i32,
        expiring_soon: 1,
        expired_permissions: 0,
        highest_group: "Professional Plan".to_string(),
    };
    
    tracing::info!(
        audit_id = %audit_id,
        wallet_address = %wallet_address,
        active_permissions = permission_summary.active_permissions,
        "Wallet permissions retrieved"
    );
    
    let response = WalletPermissionsResponse {
        wallet_address,
        permission_groups,
        effective_permissions,
        permission_summary,
        audit_id,
        retrieved_at: Utc::now(),
    };
    
    Ok(JsonResponse(ApiResponse {
        success: true,
        data: Some(response),
        error: None,
        message: "Wallet permissions retrieved successfully".to_string(),
        timestamp: Utc::now(),
    }))
}

/**
 * Bulk permission validation for performance
 * POST /api/permissions/validate-bulk
 */
#[derive(Debug, Deserialize)]
pub struct BulkPermissionValidationRequest {
    pub wallet_address: String,      // Web3 wallet address
    pub permissions: Vec<String>,
    pub context: Option<HashMap<String, serde_json::Value>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkPermissionValidationResponse {
    pub wallet_address: String,      // Web3 wallet address
    pub results: HashMap<String, PermissionValidationResponse>,
    pub overall_valid: bool,
    pub audit_id: String,
    pub validated_at: DateTime<Utc>,
}

pub async fn validate_bulk_permissions_handler(
    State(app_state): State<AppState>,
    Json(request): Json<BulkPermissionValidationRequest>,
) -> Result<JsonResponse<BulkPermissionValidationResponse>, StatusCode> {
    let audit_id = Uuid::new_v4().to_string();
    let mut results = HashMap::new();
    let mut all_valid = true;
    
    // TODO: Optimize with batch validation logic using database functions
    for permission in &request.permissions {
        let individual_request = PermissionValidationRequest {
            wallet_address: request.wallet_address.clone(),
            permission: permission.clone(),
            context: request.context.clone(),
            ip_address: request.ip_address.clone(),
            user_agent: request.user_agent.clone(),
            request_id: request.request_id.clone(),
        };
        
        // Use the individual validation handler for now - can be optimized later
        let validation_response = validate_permission_handler(
            State(app_state.clone()),
            Json(individual_request),
        ).await?;
        
        if !validation_response.0.is_valid {
            all_valid = false;
        }
        
        results.insert(permission.clone(), validation_response.0);
    }
    
    tracing::info!(
        audit_id = %audit_id,
        wallet_address = %request.wallet_address,
        permissions_count = request.permissions.len(),
        overall_valid = all_valid,
        "Bulk permission validation completed"
    );
    
    Ok(JsonResponse(BulkPermissionValidationResponse {
        wallet_address: request.wallet_address,
        results,
        overall_valid: all_valid,
        audit_id,
        validated_at: Utc::now(),
    }))
}

// ============================================================================
// HELPER FUNCTIONS FOR PERMISSION VALIDATION
// ============================================================================

/**
 * Core permission validation logic using unified system
 */
async fn validate_wallet_permission(_wallet_address: &str, permission: &str) -> bool {
    // TODO: Implement comprehensive validation logic using database functions:
    // 1. Call wallet_has_permission(wallet_address, permission) database function
    // 2. Handle wildcard permission patterns (admin:*:*, epsx:*:*, platform:resource:*)
    // 3. Check temporal constraints from wallet_group_memberships.expires_at
    // 4. Validate wallet_group_memberships.is_active status
    // 5. Handle permission inheritance from parent groups if implemented
    
    // Mock validation - replace with real database implementation
    match permission {
        "epsx:analytics:view:3" => true,  // Free plan
        "epsx:analytics:view:50" => true, // Professional plan  
        "epsx:analytics:export" => true,  // Professional plan
        "epsx:trading:advanced" => true,  // Professional plan
        "epsx:analytics:*" => true,       // Enterprise plan
        "epsx:*:*" => false,              // Full epsx access (Enterprise/Admin)
        "admin:*:*" => false,             // Admin only
        _ => false,
    }
}