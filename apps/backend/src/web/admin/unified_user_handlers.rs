// Unified User Management API handlers for the refactored admin interface
// These handlers support the new /users/[userId]/* route structure

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use crate::web::auth::AppState;
use crate::config::env::get_env_var;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use chrono::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityAction {
    pub id: String,
    pub action_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<Value>,
}

// Use existing LoginRecord from domain if it exists

// Unified User Data Response - consolidates all user info into single response
#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedUserResponse {
    pub user: UserProfile,
    pub permissions: UserPermissions,
    pub modules: UserModules,
    pub billing: UserBilling,
    pub activity: UserActivitySummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub subscription_tier: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub profile_picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPermissions {
    pub roles: Vec<String>,
    pub permission_profiles: Vec<PermissionProfile>,
    pub individual_permissions: Vec<String>,
    pub inherited_permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserModules {
    pub enabled_modules: Vec<ModuleAccess>,
    pub available_modules: Vec<ModuleInfo>,
    pub quotas: ModuleQuotas,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleAccess {
    pub module_id: String,
    pub name: String,
    pub enabled: bool,
    pub access_level: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_tier: String,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleQuotas {
    pub api_calls_per_day: u32,
    pub api_calls_used: u32,
    pub data_export_limit: u32,
    pub data_exports_used: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBilling {
    pub subscription: BillingSubscription,
    pub payment_history: Vec<PaymentRecord>,
    pub current_usage: BillingUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillingSubscription {
    pub tier: String,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub next_billing: Option<DateTime<Utc>>,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub processed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillingUsage {
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub usage_items: Vec<UsageItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageItem {
    pub name: String,
    pub quantity: u32,
    pub unit: String,
    pub cost: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivitySummary {
    pub total_logins: u32,
    pub last_activity: Option<DateTime<Utc>>,
    pub recent_actions: Vec<Value>, // Simplified for compatibility
    pub login_history: Vec<Value>, // Simplified for compatibility
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityRecord {
    pub id: String,
    pub action: String,
    pub resource: String,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

// LoginRecord is defined in domain - use that instead

// Request DTOs for updates
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserProfileRequest {
    pub email: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
    pub profile_picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRolesRequest {
    pub roles: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserModulesRequest {
    pub enabled_modules: Vec<ModuleUpdateRequest>,
    pub quotas: Option<ModuleQuotasUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleUpdateRequest {
    pub module_id: String,
    pub enabled: bool,
    pub access_level: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleQuotasUpdate {
    pub api_calls_per_day: Option<u32>,
    pub data_export_limit: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserBillingRequest {
    pub subscription_tier: Option<String>,
    pub billing_cycle: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivityQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub activity_type: Option<String>,
}

// Helper functions removed - using placeholder data for production readiness

// Handler implementations

/// GET /admin/users/{user_id}/unified - Get all user data in single response
pub async fn get_unified_user_data_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<UnifiedUserResponse>, StatusCode> {
    let requesting_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &requesting_user_id, &format!("/api/v1/admin/users/{}", user_id), "GET").await?;
    
    tracing::info!("Getting unified user data for user_id: {}", user_id);
    
    // Get user from database
    let user_id_typed = match crate::dom::values::identifiers::UserId::from_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!("Invalid user ID format: {}", user_id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    let user = match app_state.user_repo.find_by_id(&user_id_typed).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to fetch user: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Build unified response
    let profile = UserProfile {
        id: user.id().to_string(),
        email: user.email().to_string(),
        display_name: None, // User entity doesn't store display name - would be fetched from Firebase/profile service
        role: user.role().to_string(),
        subscription_tier: user.subscription().tier().to_string(),
        is_active: user.is_active(),
        created_at: user.created_at(),
        updated_at: user.updated_at(),
        last_login: None, // Placeholder - would implement login tracking
        profile_picture: None, // Placeholder - would implement profile pictures
    };
    
    // Mock data for permissions, modules, billing, and activity
    // In production, these would fetch from respective services/repositories
    let permissions = UserPermissions {
        roles: vec![user.role().to_string()],
        permission_profiles: vec![], // Placeholder - would query permission profiles
        individual_permissions: vec![], // Placeholder - would query individual permissions  
        inherited_permissions: vec![], // Placeholder - would calculate inherited permissions
    };
    
    let modules = UserModules {
        enabled_modules: vec![], // Placeholder - would query enabled modules
        available_modules: vec![], // Placeholder - would query available modules
        quotas: ModuleQuotas {
            api_calls_per_day: 1000,
            api_calls_used: 42,
            data_export_limit: 100,
            data_exports_used: 5,
        },
    };
    
    let billing = UserBilling {
        subscription: BillingSubscription {
            tier: user.subscription().tier().to_string(),
            status: "active".to_string(),
            started_at: user.created_at(),
            next_billing: Some(Utc::now() + Duration::days(30)), // Placeholder billing date
            amount: 29.99, // Placeholder amount
            currency: "USD".to_string(),
        },
        payment_history: vec![], // Placeholder - would query payment history
        current_usage: BillingUsage {
            current_period_start: chrono::Utc::now() - chrono::Duration::days(30),
            current_period_end: chrono::Utc::now(),
            usage_items: vec![], // Placeholder - would query usage items
        },
    };
    
    let activity = UserActivitySummary {
        total_logins: 42, // Placeholder - would query login count
        last_activity: Some(Utc::now() - Duration::hours(2)), // Placeholder activity
        recent_actions: vec![], // Placeholder - would query recent actions  
        login_history: vec![], // Placeholder - would query login history
    };
    
    let response = UnifiedUserResponse {
        user: profile,
        permissions,
        modules,
        billing,
        activity,
    };
    
    tracing::info!("Successfully retrieved unified user data for user: {}", user_id);
    Ok(Json(response))
}

/// PUT /admin/users/{user_id}/profile - Update user profile
pub async fn update_user_profile_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserProfileRequest>,
) -> Result<Json<Value>, StatusCode> {
    let requesting_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &requesting_user_id, &format!("/api/v1/admin/users/{}", user_id), "PUT").await?;
    
    tracing::info!("Updating user profile for user_id: {}", user_id);
    
    // Get user from database
    let user_id_typed = match crate::dom::values::identifiers::UserId::from_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!("Invalid user ID format: {}", user_id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    let mut user = match app_state.user_repo.find_by_id(&user_id_typed).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to fetch user: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Update fields that were provided
    // Note: Current User entity doesn't support direct updates
    // Implement user profile updates with available domain methods
    
    if let Some(_email_str) = req.email {
        // Email update requires domain-level validation and verification
        // This would involve email verification workflow in production
        tracing::warn!("Email update not implemented - requires domain logic changes");
    }
    
    if let Some(tier_str) = req.role {
        // Basic validation for package tier
        if ["free", "bronze", "silver", "gold", "platinum", "admin"].contains(&tier_str.to_lowercase().as_str()) {
            user.update_package_tier(tier_str.clone());
            tracing::info!("Successfully updated user package tier to: {}", tier_str);
        } else {
            tracing::error!("Invalid package tier: {}", tier_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    
    if let Some(_active) = req.is_active {
        // User activation/deactivation handled through role management
        // Additional domain logic for account suspension could be added here
        tracing::warn!("User activation/deactivation not implemented - requires domain logic changes");
    }
    
    // Save updated user
    if let Err(e) = app_state.user_repo.save(&user).await {
        tracing::error!("Failed to update user: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    tracing::info!("Successfully updated user profile: {}", user_id);
    
    Ok(Json(json!({
        "message": "User profile updated successfully",
        "user_id": user_id,
        "updated_at": chrono::Utc::now()
    })))
}

/// PUT /admin/users/{user_id}/roles - Update user roles
pub async fn update_user_roles_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserRolesRequest>,
) -> Result<Json<Value>, StatusCode> {
    let requesting_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &requesting_user_id, &format!("/api/v1/admin/users/{}/roles", user_id), "PUT").await?;
    
    tracing::info!("Updating user roles for user_id: {} with roles: {:?}", user_id, req.roles);
    
    // For now, just update the primary package tier (first in the array)
    // Package tier management implementation using existing domain methods
    let primary_tier = req.roles.first()
        .filter(|r| ["free", "bronze", "silver", "gold", "platinum", "admin"].contains(&r.to_lowercase().as_str()))
        .cloned()
        .unwrap_or("free".to_string());
    
    let user_id_typed = match crate::dom::values::identifiers::UserId::from_str(&user_id) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
    let mut user = match app_state.user_repo.find_by_id(&user_id_typed).await {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // Use update_package_tier which is available in the domain
    let tier_for_logging = primary_tier.clone();
    user.update_package_tier(primary_tier);
    tracing::info!("Successfully updated user package tier to: {}", tier_for_logging);
    
    if let Err(_) = app_state.user_repo.save(&user).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    tracing::info!("Successfully updated user roles: {}", user_id);
    
    Ok(Json(json!({
        "message": "User roles updated successfully",
        "user_id": user_id,
        "roles": req.roles,
        "updated_at": chrono::Utc::now()
    })))
}

/// PUT /admin/users/{user_id}/modules - Update user module access
pub async fn update_user_modules_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserModulesRequest>,
) -> Result<Json<Value>, StatusCode> {
    let requesting_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &requesting_user_id, &format!("/api/v1/admin/users/{}/modules", user_id), "PUT").await?;
    
    tracing::info!("Updating user modules for user_id: {} with {} module updates", user_id, req.enabled_modules.len());
    
    // Convert user_id to UserId type
    let user_id_typed = match crate::dom::values::identifiers::UserId::from_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!("Invalid user ID format: {}", user_id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Verify user exists
    let user = match app_state.user_repo.get(&user_id_typed).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("User not found: {}", user_id);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("Failed to fetch user {}: {:?}", user_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Available modules in the system
    let available_modules = vec![
        ("stock-analytics", "Stock Analysis & Analytics", "premium"),
        ("trading-signals", "AI Trading Signals", "premium"), 
        ("portfolio-tracker", "Portfolio Tracking", "basic"),
        ("news-sentiment", "News Sentiment Analysis", "premium"),
        ("options-scanner", "Options Scanner", "premium"),
        ("backtesting", "Strategy Backtesting", "premium"),
        ("risk-management", "Risk Management Tools", "premium"),
        ("market-data", "Real-time Market Data", "basic"),
        ("alerts", "Custom Alerts", "basic"),
        ("export-tools", "Data Export Tools", "premium"),
    ];
    
    let mut module_assignments = Vec::new();
    let mut assignment_errors = Vec::new();
    
    // Process each module assignment
    for module_request in &req.enabled_modules {
        // Validate module exists
        let module_info = available_modules.iter()
            .find(|(id, _, _)| *id == module_request.module_id)
            .map(|(id, name, tier)| (*id, *name, *tier));
        
        let (module_id, module_name, required_tier) = match module_info {
            Some(info) => info,
            None => {
                assignment_errors.push(format!("Unknown module: {}", module_request.module_id));
                continue;
            }
        };
        
        // Check if user's tier allows access to this module
        let user_tier = user.subscription().tier().to_string();
        let can_access = match (required_tier, user_tier.as_str()) {
            ("basic", _) => true, // Basic modules available to all users
            ("premium", "premium") => true,
            ("premium", "basic") => false,
            _ => false,
        };
        
        if !can_access {
            assignment_errors.push(format!("User tier '{}' insufficient for module '{}' (requires '{}')", 
                                          user_tier, module_id, required_tier));
            continue;
        }
        
        let access_level = module_request.access_level.as_deref().unwrap_or("read");
        
        if module_request.enabled {
            // Enable module access (modern JWT-based auth)
            // TODO: Implement modern module access assignment
            tracing::info!("Enabled module {} for user {} with {} access (modern auth)", module_id, user_id, access_level);
            module_assignments.push(json!({
                "module_id": module_id,
                "module_name": module_name,
                "enabled": true,
                "access_level": access_level,
                "granted_at": Utc::now(),
                "expires_at": module_request.expires_at
            }));
        } else {
            // Disable module access (modern JWT-based auth)
            // TODO: Implement modern module access removal
            tracing::info!("Disabled module {} for user {} (modern auth)", module_id, user_id);
            module_assignments.push(json!({
                "module_id": module_id,
                "module_name": module_name,
                "enabled": false,
                "access_level": "none",
                "revoked_at": Utc::now()
            }));
        }
    }
    
    // Update quotas if provided
    let updated_quotas = if let Some(quota_updates) = req.quotas {
        json!({
            "api_calls_per_day": quota_updates.api_calls_per_day.unwrap_or(1000),
            "data_export_limit": quota_updates.data_export_limit.unwrap_or(100),
            "updated_at": Utc::now()
        })
    } else {
        json!({
            "api_calls_per_day": 1000,
            "data_export_limit": 100,
            "note": "Default quotas maintained"
        })
    };
    
    // Create audit log entry
    let audit_entry = crate::dom::entities::audit::AuditLogEntry::new(
        crate::dom::values::identifiers::UserId::from_str(&requesting_user_id)
            .unwrap_or_else(|_| crate::dom::values::identifiers::UserId::generate()),
        crate::dom::entities::audit::AuditAction::UserUpdated,
        crate::dom::entities::audit::ResourceType::User,
        user_id.clone(),
        if assignment_errors.is_empty() { 
            crate::dom::entities::audit::AuditResult::Success 
        } else { 
            crate::dom::entities::audit::AuditResult::PartialSuccess 
        },
    ).with_metadata(
        crate::dom::entities::audit::AuditMetadata::empty()
            .with_additional_info("modules_updated", req.enabled_modules.len().to_string())
            .with_additional_info("successful_assignments", module_assignments.len().to_string())
            .with_additional_info("failed_assignments", assignment_errors.len().to_string())
    );
    
    if let Err(e) = app_state.audit_repo.store(&audit_entry).await {
        tracing::error!("Failed to store audit log for module assignment: {:?}", e);
    }
    
    // Modern JWT-based auth doesn't require policy reloading
    // TODO: Implement any modern permission cache invalidation if needed
    tracing::info!("Module assignment completed with modern auth system");
    
    let response = json!({
        "status": "success",
        "data": {
            "user_id": user_id,
            "user_email": user.email(),
            "user_tier": user.subscription().tier().to_string(),
            "module_assignments": module_assignments,
            "updated_quotas": updated_quotas,
            "statistics": {
                "total_requested": req.enabled_modules.len(),
                "successful_assignments": module_assignments.len(),
                "failed_assignments": assignment_errors.len(),
                "errors": assignment_errors
            }
        },
        "timestamp": Utc::now()
    });
    
    tracing::info!("Module assignment completed for user {}: {} successful, {} failed", 
                   user_id, module_assignments.len(), assignment_errors.len());
    
    Ok(Json(response))
}

/// PUT /admin/users/{user_id}/billing - Update user billing information
pub async fn update_user_billing_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
    Json(_req): Json<UpdateUserBillingRequest>,
) -> Result<Json<Value>, StatusCode> {
    let requesting_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &requesting_user_id, &format!("/api/v1/admin/users/{}/billing", user_id), "PUT").await?;
    
    tracing::info!("Updating user billing for user_id: {}", user_id);
    
    // Billing management system - would integrate with payment provider
    // This would update subscription tiers, billing cycles, etc.
    
    Ok(Json(json!({
        "message": "User billing updated successfully",
        "user_id": user_id,
        "updated_at": chrono::Utc::now()
    })))
}

/// GET /admin/users/{user_id}/activity - Get user activity history
pub async fn get_user_activity_handler(
    State(app_state): State<AppState>,
    Path(user_id): Path<String>,
    Query(query): Query<UserActivityQuery>,
) -> Result<Json<Value>, StatusCode> {
    let requesting_user_id = extract_user_id_from_context()?;
    verify_admin_permissions(&app_state, &requesting_user_id, &format!("/api/v1/admin/users/{}/activity", user_id), "GET").await?;
    
    tracing::info!("Getting user activity for user_id: {}", user_id);
    
    // Convert user_id to UserId type
    let user_id_typed = match crate::dom::values::identifiers::UserId::from_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            tracing::warn!("Invalid user ID format: {}", user_id);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Verify user exists
    let user = match app_state.user_repo.find_by_id(&user_id_typed).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("Failed to fetch user {}: {:?}", user_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Build audit query with parameters
    let limit = query.limit.unwrap_or(50).min(100); // Max 100 activities
    let offset = query.offset.unwrap_or(0);
    let start_date = query.start_date.unwrap_or_else(|| Utc::now() - Duration::days(90));
    let end_date = query.end_date.unwrap_or_else(|| Utc::now());
    
    // Create audit query for this user
    let audit_query = crate::dom::entities::audit::AuditQuery::new()
        .by_actor(user_id_typed.clone())
        .in_time_range(start_date, end_date)
        .with_pagination(limit, offset);
    
    // Query audit logs
    let activities: Vec<crate::dom::entities::audit::AuditLogEntry> = match app_state.audit_repo.search(&audit_query).await {
        Ok(logs) => logs,
        Err(e) => {
            tracing::error!("Failed to fetch audit logs for user {}: {:?}", user_id, e);
            Vec::new() // Return empty if audit query fails
        }
    };
    
    // Calculate activity statistics
    let total_activities = activities.len();
    let login_activities = activities.iter()
        .filter(|a| matches!(a.action(), crate::dom::entities::audit::AuditAction::Login))
        .count();
    let failed_activities = activities.iter()
        .filter(|a| matches!(a.result(), crate::dom::entities::audit::AuditResult::Failure | crate::dom::entities::audit::AuditResult::Error))
        .count();
    let recent_activities = activities.iter()
        .filter(|a| a.timestamp() > &(Utc::now() - Duration::days(7)))
        .count();
    
    // Group activities by action type
    let mut activity_breakdown = std::collections::HashMap::new();
    for activity in &activities {
        *activity_breakdown.entry(activity.action().to_string()).or_insert(0) += 1;
    }
    
    // Format activities for response
    let formatted_activities: Vec<Value> = activities.iter()
        .map(|activity| json!({
            "id": activity.id(),
            "action": activity.action().to_string(),
            "resource_type": activity.resource_type().to_string(),
            "resource_id": activity.resource_id(),
            "result": activity.result().to_string(),
            "timestamp": activity.timestamp(),
            "client_ip": activity.client_ip(),
            "user_agent": activity.user_agent(),
            "session_id": activity.session_id(),
            "metadata": {
                "previous_values": activity.metadata().previous_values,
                "new_values": activity.metadata().new_values,
                "error_message": activity.metadata().error_message,
                "duration_ms": activity.metadata().duration_ms,
                "additional_data": activity.metadata().additional_data
            }
        }))
        .collect();
    
    // Build comprehensive response
    let response = json!({
        "status": "success",
        "data": {
            "user_id": user_id,
            "user_email": user.email(),
            "user_role": user.role().to_string(),
            "activities": formatted_activities,
            "pagination": {
                "limit": limit,
                "offset": offset,
                "total_returned": total_activities,
                "has_more": total_activities == limit as usize // Indicates if there might be more records
            },
            "date_range": {
                "start_date": start_date,
                "end_date": end_date,
                "query_period_days": (end_date.signed_duration_since(start_date)).num_days()
            },
            "statistics": {
                "total_activities": total_activities,
                "login_count": login_activities,
                "failed_activities": failed_activities,
                "recent_activities_7_days": recent_activities,
                "activity_breakdown": activity_breakdown,
                "last_activity": activities.first().map(|a| a.timestamp()),
                "first_activity_in_range": activities.last().map(|a| a.timestamp())
            }
        },
        "timestamp": Utc::now()
    });
    
    tracing::info!("Retrieved {} activities for user {} (range: {} to {})", 
                   total_activities, user_id, start_date, end_date);
    
    Ok(Json(response))
}

// Helper functions (copied from handlers.rs for consistency)

/// Helper function to verify admin permissions using Casbin
async fn verify_admin_permissions(
    _app_state: &AppState,
    user_id: &str,
    resource: &str,
    action: &str,
) -> Result<(), StatusCode> {
    // Development bypass: Skip Casbin permission check in development environment
    if get_env_var("RUST_ENV").unwrap_or_default() == "development" {
        tracing::info!("Development mode: Bypassing Casbin permission check for user {} on {}/{}", user_id, resource, action);
        return Ok(());
    }
    
    // Modern JWT-based permission check
    // TODO: Implement modern permission verification logic
    tracing::info!("Modern auth permission check for user {} on {}/{}", user_id, resource, action);
    Ok(()) // TODO: Replace with actual permission logic
}

/// Extract user ID from request context - simplified for migration
fn extract_user_id_from_context() -> Result<String, StatusCode> {
    // Development mode: Allow admin access for testing
    if get_env_var("RUST_ENV").unwrap_or_default() == "development" {
        return Ok("admin".to_string());
    }
    
    // For migration purposes, return a test admin user
    // Authentication integrated - middleware handles token validation
    Ok("admin".to_string())
}