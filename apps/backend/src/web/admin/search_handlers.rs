// Admin search handlers with Diesel implementation

use axum::{extract::{Query, State}, response::Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{warn, info};
use std::sync::Arc;

use crate::app::ports::repositories::{UserRepository, UserPermissionRepository, UserSearchFilters};
use crate::infra::db::diesel::DbPool;
use crate::web::auth::routes::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSearchQuery {
    pub search: Option<String>,
    pub email: Option<String>,
    pub package_tier: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchUsersResponse {
    pub users: Vec<UserSearchResult>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSearchResult {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub firebase_uid: Option<String>,
    pub package_tier: Option<String>,
    pub subscription_tier: Option<String>,
    pub permissions: Vec<String>,
    pub created_at: String,
    pub email_verified: Option<bool>,
    pub status: String,
    pub is_active: bool,
}

pub async fn search_users_handler(
    State(state): State<AppState>,
    Query(query): Query<UserSearchQuery>
) -> Result<Json<SearchUsersResponse>, (StatusCode, String)> {
    use crate::infra::db::diesel::repos::{DieselUserRepository, DieselUserPermissionRepository};
    
    info!("Searching users with query: {:?}", query);
    
    // Create repository instances
    let user_repo = DieselUserRepository::new(state.db_pool.clone());
    let permission_repo = DieselUserPermissionRepository::new(state.db_pool.clone());
    
    // Set pagination parameters
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100); // Max 100 per page
    let offset = (page - 1) * per_page;
    
    // Create search filters
    let filters = UserSearchFilters {
        search: query.search.clone(),
        email: query.email.clone(),
        package_tier: query.package_tier.clone(),
        status: query.status.clone(),
        tier: None, // Alias for package_tier
        last_login_after: None,
        last_login_before: None,
        created_after: None,
        created_before: None,
        has_module: None,
        permission_profile: None,
        email_verified: None,
        two_factor_enabled: None,
        has_api_keys: None,
    };
    
    let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
    let sort_order = query.sort_order.as_deref().unwrap_or("desc");
    
    // Execute search
    let users_result = user_repo.search_users(&filters, offset, per_page, sort_by, sort_order).await;
    let total_result = user_repo.count_search_users(&filters).await;
    
    match (users_result, total_result) {
        (Ok(users), Ok(total)) => {
            let mut user_results: Vec<UserSearchResult> = Vec::new();
            
            // Process each user and fetch their permissions
            for user in users {
                let is_active = user.is_active();
                let status = if is_active { "active".to_string() } else { "inactive".to_string() };
                
                // Fetch user permissions
                let permissions = match permission_repo.get_user_permissions(user.id()).await {
                    Ok(perms) => perms.into_iter().map(|p| p.permission().to_string()).collect(),
                    Err(e) => {
                        warn!("Failed to fetch permissions for user {}: {}", user.id().0, e);
                        vec![] // Empty permissions on error
                    }
                };
                
                // Determine subscription tier based on permissions
                let subscription_tier = if permissions.iter().any(|p| p.starts_with("admin:")) {
                    Some("admin".to_string())
                } else if permissions.iter().any(|p| p.contains("premium") || p.contains("pro")) {
                    Some("premium".to_string())
                } else {
                    Some("basic".to_string())
                };
                
                user_results.push(UserSearchResult {
                    id: user.id().0.to_string(),
                    email: user.email().to_string(),
                    display_name: Some(user.email().to_string().split('@').next().unwrap_or("User").to_string()), // Extract name from email
                    firebase_uid: Some(user.firebase_uid().to_string()),
                    package_tier: Some("basic".to_string()), // Default tier
                    subscription_tier,
                    permissions,
                    created_at: user.created_at().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    email_verified: Some(true), // Default to true for simplicity
                    status,
                    is_active,
                });
            }
            
            info!("Found {} users (total: {})", user_results.len(), total);
            
            Ok(Json(SearchUsersResponse {
                users: user_results,
                total,
                page,
                per_page,
            }))
        }
        (Err(e), _) | (_, Err(e)) => {
            warn!("User search failed: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Search failed: {}", e)
            ))
        }
    }
}

// Additional search handlers for different admin functionalities
pub async fn search_audit_logs_handler(
    State(_pool): State<Arc<DbPool>>,
    Query(_query): Query<serde_json::Value>
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // TODO: Implement audit log search
    info!("Audit log search requested");
    Ok(Json(serde_json::json!({
        "logs": [],
        "total": 0,
        "message": "Audit log search not yet implemented"
    })))
}

pub async fn search_security_events_handler(
    State(_pool): State<Arc<DbPool>>,
    Query(_query): Query<serde_json::Value>
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // TODO: Implement security event search
    info!("Security event search requested");
    Ok(Json(serde_json::json!({
        "events": [],
        "total": 0,
        "message": "Security event search not yet implemented"
    })))
}