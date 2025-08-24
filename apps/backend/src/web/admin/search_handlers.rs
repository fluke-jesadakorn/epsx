// Admin search handlers - Stubbed for Diesel migration
// TODO: Implement with Diesel

use axum::{extract::Query, response::Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSearchQuery {
    pub search: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchUsersResponse {
    pub users: Vec<UserSearchResult>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSearchResult {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
}

pub async fn search_users_handler(
    Query(_query): Query<UserSearchQuery>
) -> Result<Json<SearchUsersResponse>, (StatusCode, &'static str)> {
    warn!("Search users handler stubbed - implement with Diesel");
    
    Ok(Json(SearchUsersResponse {
        users: vec![],
        total: 0,
        page: 1,
        per_page: 20,
    }))
}

// Additional stub handlers
pub fn stub_function() {
    warn!("Admin search handlers stubbed - implement with Diesel");
}