// Admin search handlers with Diesel implementation

use axum::{extract::{Query, State}, response::Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{warn, info};
use std::sync::Arc;

use crate::domain::shared_kernel::AggregateRoot;
use sqlx::PgPool as DbPool;
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
    
    
    info!("Searching users with query: {:?}", query);
    
    // Use repository instances from AppState (properly typed with trait methods)
    let user_repo = &state.user_repo;
    
    // Set pagination parameters
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100); // Max 100 per page
    let offset = (page - 1) * per_page;
    
    // Create search criteria using DDD approach
    let criteria = crate::domain::user_management::repository_ports::user_repository_port::UserSearchCriteria {
        search_term: query.search.clone(),
        email_pattern: query.email.clone(),
        is_active: query.status.as_ref().map(|s| s == "active"),
        email_verified: None,
        has_permissions: Vec::new(), // TODO: Convert package_tier to permissions if needed
        created_after: None,
        created_before: None,
        last_login_after: None,
        firebase_uid_pattern: None,
        custom_filters: std::collections::HashMap::new(),
    };
    
    // Execute search using DDD repository methods
    let search_result = user_repo.find_by_criteria(&criteria, per_page, offset).await;
    
    match search_result {
        Ok(result) => {
            let users = result.users;
            let total = result.total_count;
            let mut user_results: Vec<UserSearchResult> = Vec::new();
            
            // Process each user and fetch their permissions
            for user in users {
                let is_active = user.is_active();
                let status = if is_active { "active".to_string() } else { "inactive".to_string() };
                
                // Get user permissions from User aggregate (direct access)
                let permissions = user.active_permissions();
                
                // Determine subscription tier based on permissions
                let subscription_tier = if permissions.iter().any(|p| p.starts_with("admin:")) {
                    Some("admin".to_string())
                } else if permissions.iter().any(|p| p.contains("premium") || p.contains("pro")) {
                    Some("premium".to_string())
                } else {
                    Some("basic".to_string())
                };
                
                user_results.push(UserSearchResult {
                    id: user.id().to_string(),
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
        Err(e) => {
            warn!("User search failed: {:?}", e);
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